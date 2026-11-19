use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;

const MAX_MODEL: u64 = 32 * 1024 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub version: String,
    pub file: String,
    pub sha256: String,
    pub bytes: u64,
    pub context: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub format: u32,
    pub name: String,
    pub version: String,
    pub file: String,
    pub sha256: String,
    #[serde(default = "default_context")]
    pub context: u32,
}
fn default_context() -> u32 {
    4096
}
pub fn model_path(root: &Path, m: &Model) -> PathBuf {
    root.join("models").join(&m.id).join("model.gguf")
}
pub fn list(root: &Path) -> Result<Vec<Model>> {
    let dir = root.join("models");
    fs::create_dir_all(&dir)?;
    let mut models = Vec::new();
    for e in fs::read_dir(dir)? {
        let e = e?;
        if !e.file_type()?.is_dir() || e.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        let manifest = e.path().join("installed.json");
        if manifest.exists() {
            let m: Model = serde_json::from_reader(File::open(manifest)?)?;
            ensure!(
                Uuid::parse_str(&m.id).is_ok() && e.file_name().to_string_lossy() == m.id,
                "Ungültiges installiertes Modell."
            );
            ensure!(
                model_path(root, &m).is_file(),
                "Modelldatei von {} fehlt.",
                m.name
            );
            models.push(m);
        }
    }
    models.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
    Ok(models)
}
fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('/')
        && !name.contains('\\')
        && !name.contains(':')
        && !name.contains('\0')
        && name.split('/').all(|s| s != ".." && s != ".")
}
pub fn validate_gguf(path: &Path) -> Result<()> {
    let mut f = File::open(path)?;
    let len = f.metadata()?.len();
    let mut head = [0u8; 24];
    f.read_exact(&mut head)
        .context("Die GGUF-Datei ist unvollständig.")?;
    ensure!(&head[..4] == b"GGUF", "Kein GGUF-Modell.");
    let version = u32::from_le_bytes(head[4..8].try_into()?);
    ensure!(
        (2..=3).contains(&version),
        "GGUF-Version {version} wird nicht unterstützt."
    );
    let tensors = u64::from_le_bytes(head[8..16].try_into()?);
    let metadata = u64::from_le_bytes(head[16..24].try_into()?);
    ensure!(
        tensors > 0
            && tensors < 1_000_000
            && metadata > 0
            && metadata < 1_000_000
            && len > 24 + tensors * 16,
        "GGUF-Header ist ungültig oder die Datei abgeschnitten."
    );
    Ok(())
}
pub fn hash(path: &Path) -> Result<String> {
    let mut h = Sha256::new();
    let mut f = File::open(path)?;
    let mut b = vec![0; 1024 * 1024];
    loop {
        let n = f.read(&mut b)?;
        if n == 0 {
            break;
        }
        h.update(&b[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}

/// Extract only data, never imported executable code. Legacy KD portable ZIPs
/// are accepted by locating their single GGUF; their runtime is not installed.
pub fn import(root: &Path, archive: &Path, mut progress: impl FnMut(String)) -> Result<Model> {
    fs::create_dir_all(root.join("models"))?;
    let mut zip = zip::ZipArchive::new(File::open(archive)?)?;
    ensure!(zip.len() <= 4096, "Das Archiv enthält zu viele Einträge.");
    let mut names = HashSet::new();
    let mut ggufs = Vec::new();
    let mut manifest_index = None;
    let mut total = 0u64;
    for i in 0..zip.len() {
        let e = zip.by_index(i)?;
        let name = e.name();
        ensure!(safe_name(name), "Unsicherer Archivpfad: {name}");
        ensure!(
            names.insert(name.to_lowercase()),
            "Doppelter Archivpfad: {name}"
        );
        ensure!(
            e.unix_mode().unwrap_or(0) & 0o170000 != 0o120000,
            "Symbolische Links sind nicht erlaubt."
        );
        total = total
            .checked_add(e.size())
            .context("Archivgrösse ungültig")?;
        ensure!(
            total <= MAX_MODEL + 2 * 1024 * 1024 * 1024,
            "Archiv ist zu gross (maximal 34 GiB entpackt)."
        );
        if name.to_ascii_lowercase().ends_with(".gguf") {
            ensure!(e.size() <= MAX_MODEL, "Modell ist grösser als 32 GiB.");
            ggufs.push((i, name.to_string()));
        }
        if name == "kd-model.json" {
            ensure!(e.size() <= 16384, "Modellmanifest ist zu gross.");
            manifest_index = Some(i);
        }
    }
    let manifest: Option<Manifest> = if let Some(i) = manifest_index {
        Some(serde_json::from_reader(zip.by_index(i)?)?)
    } else {
        None
    };
    let (index, _) = if let Some(m) = &manifest {
        ensure!(
            m.format == 1 && (1024..=32768).contains(&m.context),
            "Paketformat oder Kontextgrösse nicht unterstützt."
        );
        ensure!(
            !m.name.trim().is_empty()
                && m.name.chars().count() <= 100
                && m.version.chars().count() <= 60,
            "Modellname oder Version ungültig."
        );
        ensure!(
            m.sha256.len() == 64 && m.sha256.bytes().all(|c| c.is_ascii_hexdigit()),
            "SHA-256-Prüfsumme fehlt oder ist ungültig."
        );
        ggufs
            .iter()
            .find(|(_, n)| n == &m.file)
            .cloned()
            .context("Die im Manifest genannte GGUF-Datei fehlt.")?
    } else {
        ensure!(
            ggufs.len() == 1,
            "Bitte ein KD-Paket mit genau einer GGUF-Datei oder kd-model.json wählen."
        );
        ggufs.remove(0)
    };
    let id = Uuid::new_v4().to_string();
    let staging = root.join("models").join(format!(".import-{id}"));
    fs::create_dir(&staging)?;
    let result = (|| -> Result<Model> {
        progress("Modell wird entpackt und geprüft …".into());
        let target = staging.join("model.gguf");
        let mut input = zip.by_index(index)?;
        let mut output = File::create(&target)?;
        let copied = std::io::copy(&mut input, &mut output)?;
        output.sync_all()?;
        drop(output);
        ensure!(copied == input.size(), "Die Modelldatei ist unvollständig.");
        validate_gguf(&target)?;
        progress("SHA-256 wird geprüft …".into());
        let sha = hash(&target)?;
        if let Some(m) = &manifest {
            ensure!(
                sha.eq_ignore_ascii_case(&m.sha256),
                "Prüfsummenfehler: Das Modell ist beschädigt oder wurde verändert."
            );
        }
        for installed in list(root)? {
            ensure!(
                installed.sha256 != sha,
                "Dieses Modell ist bereits installiert: {} {}",
                installed.name,
                installed.version
            );
        }
        let fallback = archive
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let m = Model {
            id: id.clone(),
            name: manifest
                .as_ref()
                .map(|m| m.name.clone())
                .unwrap_or(fallback),
            version: manifest
                .as_ref()
                .map(|m| m.version.clone())
                .unwrap_or("Legacy-Paket".into()),
            file: "model.gguf".into(),
            sha256: sha,
            bytes: copied,
            context: manifest.as_ref().map(|m| m.context).unwrap_or(4096),
        };
        let mut f = File::create(staging.join("installed.json"))?;
        f.write_all(&serde_json::to_vec_pretty(&m)?)?;
        f.sync_all()?;
        drop(f);
        fs::rename(&staging, root.join("models").join(&id))?;
        Ok(m)
    })();
    // Only our own temporary directory; historical builds and input ZIP untouched.
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}
pub fn delete(root: &Path, id: &str) -> Result<()> {
    ensure!(Uuid::parse_str(id).is_ok(), "Ungültige Modell-ID.");
    let base = root.join("models").canonicalize()?;
    let target = base.join(id).canonicalize()?;
    ensure!(
        target.parent() == Some(base.as_path()),
        "Unsicherer Modellpfad."
    );
    fs::remove_dir_all(target)?;
    Ok(())
}
pub fn package(model: &Path, dest: &Path, name: &str, version: &str) -> Result<()> {
    validate_gguf(model)?;
    if dest.exists() {
        bail!("Ziel existiert bereits; bitte einen neuen Paketnamen wählen.")
    }
    let manifest = Manifest {
        format: 1,
        name: name.into(),
        version: version.into(),
        file: "model.gguf".into(),
        sha256: hash(model)?,
        context: 4096,
    };
    let mut zip = zip::ZipWriter::new(File::create(dest)?);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .large_file(true);
    zip.start_file("kd-model.json", options)?;
    zip.write_all(&serde_json::to_vec_pretty(&manifest)?)?;
    zip.start_file("model.gguf", options)?;
    std::io::copy(&mut File::open(model)?, &mut zip)?;
    zip.finish()?.sync_all()?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn archive(path: &Path, entries: &[(&str, &[u8])]) -> Result<()> {
        let mut z = zip::ZipWriter::new(File::create(path)?);
        for (n, b) in entries {
            z.start_file(*n, zip::write::SimpleFileOptions::default())?;
            z.write_all(b)?;
        }
        z.finish()?;
        Ok(())
    }
    #[test]
    fn rejects_traversal_duplicate_corrupt_and_ambiguous() -> Result<()> {
        let t = tempfile::tempdir()?;
        let z = t.path().join("test.zip");
        for entries in [
            vec![("../bad.gguf", &b"bad"[..])],
            vec![("a.gguf", &b"bad"[..]), ("A.gguf", &b"bad"[..])],
            vec![("a.gguf", &b"GGUF"[..])],
            vec![("a.gguf", &b"x"[..]), ("b.gguf", &b"y"[..])],
        ] {
            archive(&z, &entries)?;
            assert!(import(t.path(), &z, |_| {}).is_err());
        }
        assert!(list(t.path())?.is_empty());
        assert!(delete(t.path(), "../builds").is_err());
        Ok(())
    }
    #[test]
    fn package_import_duplicate_delete_and_checksum() -> Result<()> {
        let t = tempfile::tempdir()?;
        let file = t.path().join("source.gguf");
        let mut b = b"GGUF".to_vec();
        b.extend(3u32.to_le_bytes());
        b.extend(1u64.to_le_bytes());
        b.extend(1u64.to_le_bytes());
        b.extend([0; 100]);
        fs::write(&file, b)?;
        let z = t.path().join("model.zip");
        package(&file, &z, "KD Test", "1")?;
        let m = import(t.path(), &z, |_| {})?;
        assert_eq!(hash(&file)?, m.sha256);
        assert!(import(t.path(), &z, |_| {}).is_err());
        delete(t.path(), &m.id)?;
        assert!(file.exists() && z.exists());
        let bad = Manifest {
            format: 1,
            name: "KD".into(),
            version: "1".into(),
            file: "model.gguf".into(),
            sha256: "0".repeat(64),
            context: 4096,
        };
        let bytes = fs::read(&file)?;
        archive(
            &z,
            &[
                ("kd-model.json", &serde_json::to_vec(&bad)?),
                ("model.gguf", &bytes),
            ],
        )?;
        assert!(
            import(t.path(), &z, |_| {})
                .unwrap_err()
                .to_string()
                .contains("Prüfsummenfehler")
        );
        Ok(())
    }
}
