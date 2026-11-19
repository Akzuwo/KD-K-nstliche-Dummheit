use anyhow::{Result, bail};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Chat {
    pub id: String,
    pub title: String,
    pub updated: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: String,
    pub role: String,
    pub content: String,
    pub status: String,
    pub model: String,
}
#[derive(Clone)]
pub struct Store {
    path: PathBuf,
}

impl Store {
    pub fn open(root: &Path) -> Result<Self> {
        std::fs::create_dir_all(root)?;
        let store = Self {
            path: root.join("chats.sqlite3"),
        };
        store.connection()?.execute_batch("PRAGMA journal_mode=WAL;
            CREATE TABLE IF NOT EXISTS chats(id TEXT PRIMARY KEY,title TEXT NOT NULL,updated INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS messages(id TEXT PRIMARY KEY,chat_id TEXT NOT NULL REFERENCES chats(id) ON DELETE CASCADE,role TEXT NOT NULL,content TEXT NOT NULL,status TEXT NOT NULL,model TEXT NOT NULL);
            CREATE INDEX IF NOT EXISTS messages_chat ON messages(chat_id);
            CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY,value TEXT NOT NULL);
            PRAGMA user_version=1;")?;
        // A hard stop never turns a partial response into completed model context.
        store.connection()?.execute(
            "UPDATE messages SET status='stopped' WHERE status='streaming'",
            [],
        )?;
        Ok(store)
    }
    fn connection(&self) -> Result<Connection> {
        let db = Connection::open(&self.path)?;
        db.busy_timeout(std::time::Duration::from_secs(5))?;
        db.execute_batch("PRAGMA foreign_keys=ON;")?;
        Ok(db)
    }
    pub fn chats(&self, search: &str) -> Result<Vec<Chat>> {
        let db = self.connection()?;
        let mut stmt = db.prepare("SELECT id,title,updated FROM chats WHERE instr(lower(title),lower(?1))>0 OR EXISTS(SELECT 1 FROM messages WHERE chat_id=chats.id AND instr(lower(content),lower(?1))>0) ORDER BY updated DESC,rowid DESC")?;
        Ok(stmt
            .query_map([search], |r| {
                Ok(Chat {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    updated: r.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?)
    }
    pub fn create_chat(&self, title: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.connection()?.execute(
            "INSERT INTO chats VALUES(?1,?2,unixepoch())",
            params![id, title],
        )?;
        Ok(id)
    }
    pub fn rename(&self, id: &str, title: &str) -> Result<()> {
        if title.trim().is_empty() || title.chars().count() > 160 {
            bail!("Bitte einen Namen mit 1–160 Zeichen eingeben.");
        }
        self.connection()?.execute(
            "UPDATE chats SET title=?2,updated=unixepoch() WHERE id=?1",
            params![id, title.trim()],
        )?;
        Ok(())
    }
    pub fn delete_chat(&self, id: &str) -> Result<()> {
        self.connection()?
            .execute("DELETE FROM chats WHERE id=?1", [id])?;
        Ok(())
    }
    pub fn messages(&self, id: &str) -> Result<Vec<Message>> {
        let db = self.connection()?;
        let mut stmt = db.prepare(
            "SELECT id,role,content,status,model FROM messages WHERE chat_id=?1 ORDER BY rowid",
        )?;
        Ok(stmt
            .query_map([id], |r| {
                Ok(Message {
                    id: r.get(0)?,
                    role: r.get(1)?,
                    content: r.get(2)?,
                    status: r.get(3)?,
                    model: r.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?)
    }
    pub fn add(
        &self,
        chat: &str,
        role: &str,
        content: &str,
        status: &str,
        model: &str,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let mut db = self.connection()?;
        let tx = db.transaction()?;
        tx.execute(
            "INSERT INTO messages(id,chat_id,role,content,status,model) VALUES(?1,?2,?3,?4,?5,?6)",
            params![id, chat, role, content, status, model],
        )?;
        tx.execute("UPDATE chats SET updated=unixepoch() WHERE id=?1", [chat])?;
        tx.commit()?;
        Ok(id)
    }
    pub fn update(&self, id: &str, content: &str, status: &str) -> Result<()> {
        self.connection()?.execute(
            "UPDATE messages SET content=?2,status=?3 WHERE id=?1",
            params![id, content, status],
        )?;
        Ok(())
    }
    pub fn setting(&self, key: &str) -> String {
        self.connection()
            .map(|db| {
                db.query_row("SELECT value FROM settings WHERE key=?1", [key], |r| {
                    r.get(0)
                })
                .unwrap_or_default()
            })
            .unwrap_or_default()
    }
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.connection()?.execute("INSERT INTO settings VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![key,value])?;
        Ok(())
    }
}

/// Only whole completed turns are context. An interrupted user's prompt is paired
/// with its interrupted assistant and must not become a dangling prior turn.
pub fn context(messages: &[Message]) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    let mut user = None;
    for m in messages {
        if m.role == "user" {
            user = Some(m);
        } else if let Some(u) = user.take()
            && m.status == "complete"
        {
            out.push(serde_json::json!({"role":"user","content":u.content}));
            out.push(serde_json::json!({"role":"assistant","content":m.content}));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persistence_search_rename_delete_recovery() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let s = Store::open(dir.path())?;
        let c = s.create_chat("Hegel")?;
        s.add(&c, "user", "Französischer Chefkoch?", "complete", "")?;
        let m = s.add(&c, "assistant", "Angefangen", "streaming", "KD")?;
        assert_eq!(s.chats("Chefkoch")?.len(), 1);
        s.rename(&c, "Küche")?;
        assert_eq!(Store::open(dir.path())?.messages(&c)?[1].status, "stopped");
        assert!(context(&s.messages(&c)?).is_empty());
        s.update(&m, "Antwort", "complete")?;
        assert_eq!(context(&s.messages(&c)?).len(), 2);
        s.delete_chat(&c)?;
        assert!(s.messages(&c)?.is_empty());
        Ok(())
    }
}
