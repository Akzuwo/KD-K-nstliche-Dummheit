use crate::models::Model;
use anyhow::{Context, Result, bail, ensure};
use futures_util::StreamExt;
use serde_json::{Value, json};
use std::{
    fs::File,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

pub const SYSTEM: &str = "Antworte auf Deutsch in zusammenhängenden Absätzen. Erläutere offene Fragen ausführlich in ungefähr 100 bis 250 Wörtern.";
#[derive(Debug, Clone)]
pub enum Event {
    Status(String),
    Piece(String),
    Finished(String),
    Error(String),
}

pub struct Runtime {
    child: Child,
    _job: Job,
    pub pid: u32,
    base: String,
    key: String,
    http: reqwest::Client,
}
impl Drop for Runtime {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(windows)]
struct Job(windows_sys::Win32::Foundation::HANDLE);
#[cfg(windows)]
impl Job {
    fn attach(child: &Child) -> Result<Self> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::*;
        unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            ensure!(!job.is_null(), "Windows-Job konnte nicht erstellt werden.");
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &info as *const _ as _,
                std::mem::size_of_val(&info) as u32,
            ) == 0
                || AssignProcessToJobObject(job, child.as_raw_handle()) == 0
            {
                windows_sys::Win32::Foundation::CloseHandle(job);
                bail!("Lokaler Modellprozess konnte nicht abgesichert werden.");
            }
            Ok(Self(job))
        }
    }
}
#[cfg(windows)]
impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}
#[cfg(not(windows))]
struct Job;
#[cfg(not(windows))]
impl Job {
    fn attach(_: &Child) -> Result<Self> {
        Ok(Self)
    }
}

pub fn backend_path(runtime: &Path, backend: &str) -> Result<PathBuf> {
    let directory = if backend == "cuda" {
        runtime.join("cuda")
    } else {
        runtime.join("cpu")
    };
    let exe = directory.join("llama-server.exe");
    ensure!(
        exe.is_file(),
        "{}-Laufzeit fehlt. Bitte die vollständige App installieren{}.",
        if backend == "cuda" { "CUDA" } else { "CPU" },
        if backend == "cuda" {
            " oder in den Einstellungen CPU wählen"
        } else {
            ""
        }
    );
    Ok(exe)
}
fn cancelled(flag: &AtomicBool) -> Result<()> {
    if flag.load(Ordering::SeqCst) {
        bail!("Abgebrochen")
    }
    Ok(())
}
async fn interruptible<T>(
    flag: &AtomicBool,
    f: impl std::future::Future<Output = Result<T>>,
) -> Result<T> {
    tokio::select! {result=f=>result,_=async{loop{if flag.load(Ordering::SeqCst){break}tokio::time::sleep(Duration::from_millis(35)).await;}}=>bail!("Abgebrochen")}
}
impl Runtime {
    pub async fn start(
        runtime: &Path,
        model_file: &Path,
        context: u32,
        backend: &str,
        logs: &Path,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        cancelled(cancel)?;
        let exe = backend_path(runtime, backend)?.canonicalize()?;
        let model_file = model_file.canonicalize()?;
        std::fs::create_dir_all(logs)?;
        let log = File::create(logs.join("inference-latest.log"))?;
        let socket = TcpListener::bind("127.0.0.1:0")?;
        let port = socket.local_addr()?.port();
        drop(socket);
        let key = uuid::Uuid::new_v4().to_string();
        let threads = std::thread::available_parallelism()
            .map(|n| n.get() / 2)
            .unwrap_or(2)
            .clamp(1, 4)
            .to_string();
        let mut cmd = Command::new(&exe);
        cmd.current_dir(exe.parent().unwrap())
            .args(["-m"])
            .arg(&model_file)
            .args([
                "-c",
                &context.to_string(),
                "-t",
                &threads,
                "-tb",
                &threads,
                "-ngl",
                if backend == "cuda" { "99" } else { "0" },
                "-np",
                "1",
                "--host",
                "127.0.0.1",
                "--port",
                &port.to_string(),
                "--api-key",
                &key,
                "--no-webui",
                "--offline",
                "--jinja",
                "--reasoning",
                "off",
                "--chat-template-kwargs",
                "{\"enable_thinking\":false}",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(log));
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        let mut child = cmd
            .spawn()
            .context("Die lokale Inferenz-Engine konnte nicht starten.")?;
        let job = match Job::attach(&child) {
            Ok(j) => j,
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e);
            }
        };
        let mut rt = Self {
            pid: child.id(),
            child,
            _job: job,
            base: format!("http://127.0.0.1:{port}"),
            key,
            http: reqwest::Client::builder()
                .no_proxy()
                .connect_timeout(Duration::from_secs(2))
                .build()?,
        };
        let start = Instant::now();
        loop {
            cancelled(cancel)?;
            if let Some(code) = rt.child.try_wait()? {
                bail!(
                    "Modell konnte nicht geladen werden ({code}). Details: {}",
                    logs.join("inference-latest.log").display()
                );
            }
            if start.elapsed() > Duration::from_secs(180) {
                bail!(
                    "Das Modell konnte innerhalb von 180 Sekunden nicht geladen werden. Bitte RAM und Modellkompatibilität prüfen."
                );
            }
            let health = interruptible(cancel, async {
                Ok(rt
                    .http
                    .get(format!("{}/health", rt.base))
                    .bearer_auth(&rt.key)
                    .timeout(Duration::from_secs(2))
                    .send()
                    .await)
            })
            .await?;
            if health.is_ok_and(|r| r.status().is_success()) {
                return Ok(rt);
            }
            tokio::time::sleep(Duration::from_millis(120)).await;
        }
    }
    async fn post(&self, path: &str, body: Value, cancel: &AtomicBool) -> Result<Value> {
        interruptible(cancel, async {
            let r = self
                .http
                .post(format!("{}{path}", self.base))
                .bearer_auth(&self.key)
                .json(&body)
                .timeout(Duration::from_secs(60))
                .send()
                .await?;
            ensure!(
                r.status().is_success(),
                "Die Inferenz-Engine meldet beim Prüfen des Kontexts: {}",
                r.status()
            );
            Ok(r.json().await?)
        })
        .await
    }
    pub async fn generate(
        &self,
        mut history: Vec<Value>,
        max_tokens: u32,
        context: u32,
        cancel: &AtomicBool,
        mut event: impl FnMut(Event),
    ) -> Result<String> {
        ensure!(
            history.last().and_then(|m| m["role"].as_str()) == Some("user"),
            "Eine Nutzernachricht fehlt."
        );
        loop {
            let mut messages = vec![json!({"role":"system","content":SYSTEM})];
            messages.extend(history.clone());
            let template=self.post("/apply-template",json!({"messages":messages,"add_generation_prompt":true,"chat_template_kwargs":{"enable_thinking":false}}),cancel).await?;
            let tokens = self
                .post(
                    "/tokenize",
                    json!({"content":template["prompt"],"add_special":false,"parse_special":true}),
                    cancel,
                )
                .await?;
            let n = tokens["tokens"]
                .as_array()
                .context("Tokenprüfung lieferte kein Ergebnis.")?
                .len();
            if n + max_tokens as usize + 16 <= context as usize {
                break;
            }
            ensure!(
                history.len() >= 3,
                "Diese Nachricht ist zu lang für das Modell. Bitte kürzen und erneut senden."
            );
            history.drain(..2);
            event(Event::Status(
                "Ältere Gesprächspaare werden für das Kontextlimit ausgelassen …".into(),
            ));
        }
        let mut messages = vec![json!({"role":"system","content":SYSTEM})];
        messages.extend(history);
        let response=interruptible(cancel,async{Ok(self.http.post(format!("{}/v1/chat/completions",self.base)).bearer_auth(&self.key).json(&json!({"messages":messages,"stream":true,"max_tokens":max_tokens,"temperature":0.5,"top_p":0.9,"top_k":40,"min_p":0.05,"repeat_penalty":1.05,"dry_multiplier":0.2,"dry_allowed_length":4,"repeat_last_n":256,"dry_penalty_last_n":512,"cache_prompt":false,"seed":50117})).send().await?)}).await?;
        ensure!(
            response.status().is_success(),
            "Die lokale Generierung meldet {}.",
            response.status()
        );
        let mut stream = response.bytes_stream();
        let mut decoder = Sse::default();
        let mut full = String::new();
        let mut finish = String::new();
        event(Event::Status("Antwort wird geschrieben …".into()));
        loop {
            let next = interruptible(cancel, async {
                tokio::time::timeout(Duration::from_secs(120), stream.next())
                    .await
                    .context("Die Inferenz-Engine antwortet nicht mehr.")
            })
            .await?;
            let Some(chunk) = next else { break };
            for line in decoder.push(&chunk?)? {
                if let Some(payload) = line.strip_prefix("data:") {
                    let payload = payload.trim();
                    if payload == "[DONE]" {
                        continue;
                    }
                    let v: Value =
                        serde_json::from_str(payload).context("Ungültige Streaming-Antwort")?;
                    if let Some(error) = v.get("error") {
                        bail!("Inferenzfehler: {error}")
                    }
                    if let Some(piece) = v["choices"][0]["delta"]["content"].as_str() {
                        full.push_str(piece);
                        event(Event::Piece(piece.into()));
                    }
                    if let Some(reason) = v["choices"][0]["finish_reason"].as_str() {
                        finish = reason.into();
                    }
                }
            }
        }
        cancelled(cancel)?;
        ensure!(
            !finish.is_empty(),
            "Die Verbindung endete vor dem Abschluss der Antwort."
        );
        ensure!(
            !full.trim().is_empty(),
            "Das Modell hat eine leere Antwort erzeugt."
        );
        Ok(finish)
    }
}
#[derive(Default)]
pub struct Sse {
    buffer: Vec<u8>,
}
impl Sse {
    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<String>> {
        self.buffer.extend_from_slice(bytes);
        ensure!(
            self.buffer.len() < 4 * 1024 * 1024,
            "Streaming-Zeile ist zu gross."
        );
        let mut out = Vec::new();
        while let Some(i) = self.buffer.iter().position(|b| *b == b'\n') {
            let b = self.buffer.drain(..=i).collect::<Vec<_>>();
            let line = std::str::from_utf8(&b)?
                .trim_end_matches(['\r', '\n'])
                .to_string();
            if !line.is_empty() {
                out.push(line);
            }
        }
        Ok(out)
    }
}

pub fn run(
    root: PathBuf,
    runtime: PathBuf,
    model: Model,
    history: Vec<Value>,
    backend: String,
    cancel: Arc<AtomicBool>,
    send: impl Fn(Event) + Send + 'static,
) {
    std::thread::spawn(move || {
        let result = (|| -> Result<String> {
            let executor = tokio::runtime::Runtime::new()?;
            executor.block_on(async {
                send(Event::Status(format!("{} wird geladen …", model.name)));
                let rt = Runtime::start(
                    &runtime,
                    &crate::models::model_path(&root, &model),
                    model.context,
                    &backend,
                    &root.join("logs"),
                    &cancel,
                )
                .await?;
                rt.generate(history, 900, model.context, &cancel, &send)
                    .await
            })
        })();
        match result {
            Ok(reason) => send(Event::Finished(reason)),
            Err(e) => send(Event::Error(format!("{e:#}"))),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arbitrary_utf8_chunks() {
        let data = "data: {\"text\":\"Grüsse 🌍\"}\r\n\r\ndata: [DONE]\n".as_bytes();
        for size in 1..data.len() {
            let mut d = Sse::default();
            let mut out = Vec::new();
            for part in data.chunks(size) {
                out.extend(d.push(part).unwrap());
            }
            assert_eq!(out.len(), 2);
            assert!(out[0].contains("Grüsse 🌍"));
        }
    }
    #[tokio::test]
    async fn cancellation_interrupts_pending_request() {
        let c = AtomicBool::new(true);
        let r: Result<()> = interruptible(&c, std::future::pending()).await;
        assert!(r.is_err());
    }
}
