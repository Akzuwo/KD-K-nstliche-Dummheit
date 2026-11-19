use anyhow::{Context, Result, bail, ensure};
use reqwest::{Client, StatusCode, redirect::Policy};
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::windows::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};
use uuid::Uuid;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPOSITORY: &str = match option_env!("KD_UPDATE_REPOSITORY") {
    Some(repository) => repository,
    None => "Akzuwo/KD-K-nstliche-Dummheit",
};
const MAX_INSTALLER_BYTES: u64 = 750 * 1024 * 1024;

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    state: String,
    size: u64,
    digest: Option<String>,
}

struct Candidate {
    version: Version,
    asset: Asset,
    sha256: String,
}

struct Pending {
    installer: PathBuf,
    _verified_file: File,
}

struct State {
    message: String,
    pending: Option<Pending>,
}

#[derive(Clone)]
pub struct Updater {
    state: Arc<Mutex<State>>,
    install_dir: PathBuf,
    cache: PathBuf,
}

impl Updater {
    pub fn start(data: &Path) -> Option<Self> {
        let install_dir = std::env::current_exe().ok()?.parent()?.to_owned();
        if !install_dir.join("kd-install.ini").is_file() {
            return None;
        }

        let updater = Self {
            state: Arc::new(Mutex::new(State {
                message: "Suche nach Updates …".into(),
                pending: None,
            })),
            install_dir,
            cache: data.join("updates"),
        };
        let worker = updater.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            match runtime {
                Ok(runtime) => runtime.block_on(worker.watch()),
                Err(_) => {
                    worker.message("Automatische Updates sind vorübergehend nicht verfügbar.")
                }
            }
        });
        Some(updater)
    }

    pub fn status(&self) -> String {
        self.state.lock().unwrap().message.clone()
    }

    fn message(&self, message: &str) {
        self.state.lock().unwrap().message = message.into();
    }

    async fn watch(&self) {
        let Ok(client) = update_client() else {
            self.message("Automatische Updates sind vorübergehend nicht verfügbar.");
            return;
        };
        loop {
            match self.prepare(&client).await {
                Ok(Some((version, pending))) => {
                    let mut state = self.state.lock().unwrap();
                    state.message = format!("Version {version} wird beim Schliessen installiert.");
                    state.pending = Some(pending);
                    return;
                }
                Ok(None) => self.message("Keine neuere GitHub-Version verfügbar."),
                Err(_) => {
                    self.message("Updates momentan nicht verfügbar. Die Prüfung wird wiederholt.")
                }
            }
            tokio::time::sleep(Duration::from_secs(6 * 60 * 60)).await;
        }
    }

    async fn prepare(&self, client: &Client) -> Result<Option<(Version, Pending)>> {
        let release_url = format!("https://api.github.com/repos/{REPOSITORY}/releases/latest");
        let response = client
            .get(release_url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let release = response.error_for_status()?.json::<Release>().await?;
        let Some(candidate) = select_release(release, &Version::parse(APP_VERSION)?)? else {
            return Ok(None);
        };
        self.message("GitHub-Update wird heruntergeladen …");
        let pending = download(client, &self.cache, &candidate).await?;
        Ok(Some((candidate.version, pending)))
    }

    pub fn install_on_close(&self) {
        let Some(pending) = self.state.lock().unwrap().pending.take() else {
            return;
        };
        let log = self.cache.join("install.log");
        let result = Command::new(&pending.installer)
            .args([
                "/VERYSILENT",
                "/SUPPRESSMSGBOXES",
                "/SP-",
                "/NORESTART",
                "/NOCLOSEAPPLICATIONS",
            ])
            .arg(format!("/KDWaitPid={}", std::process::id()))
            .arg(path_argument("/DIR=", &self.install_dir))
            .arg(path_argument("/LOG=", &log))
            .spawn();
        if let Err(error) = result {
            let _ = fs::write(
                log,
                format!("Update-Installer konnte nicht starten: {error}\n"),
            );
        }
    }
}

fn path_argument(prefix: &str, path: &Path) -> OsString {
    let mut argument = OsString::from(prefix);
    argument.push(path);
    argument
}

fn update_client() -> Result<Client> {
    Ok(Client::builder()
        .user_agent(format!("KD-Desktop/{APP_VERSION}"))
        .https_only(true)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(15 * 60))
        .redirect(Policy::custom(|attempt| {
            let allowed = matches!(
                attempt.url().host_str(),
                Some(
                    "github.com"
                        | "release-assets.githubusercontent.com"
                        | "objects.githubusercontent.com"
                )
            );
            if allowed && attempt.url().scheme() == "https" && attempt.previous().len() < 5 {
                attempt.follow()
            } else {
                attempt.error("Unerwartete GitHub-Weiterleitung")
            }
        }))
        .build()?)
}

fn select_release(release: Release, current: &Version) -> Result<Option<Candidate>> {
    if release.draft || release.prerelease {
        return Ok(None);
    }
    let Some(tag) = release.tag_name.strip_prefix('v') else {
        return Ok(None);
    };
    let Ok(version) = Version::parse(tag) else {
        return Ok(None);
    };
    if &version <= current || !version.pre.is_empty() || !version.build.is_empty() {
        return Ok(None);
    }
    let expected = format!("KD-Desktop-{version}-Setup.exe");
    let mut assets = release
        .assets
        .into_iter()
        .filter(|asset| asset.name == expected);
    let asset = assets.next().context("GitHub-Installer fehlt")?;
    ensure!(assets.next().is_none(), "Mehrere GitHub-Installer gefunden");
    ensure!(
        asset.state == "uploaded" && (1..=MAX_INSTALLER_BYTES).contains(&asset.size),
        "Ungültiges GitHub-Asset"
    );
    let hash = asset
        .digest
        .as_deref()
        .and_then(|digest| digest.strip_prefix("sha256:"))
        .filter(|hash| hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .context("GitHub-SHA-256 fehlt oder ist ungültig")?;
    let sha256 = hash.to_ascii_lowercase();
    Ok(Some(Candidate {
        version,
        asset,
        sha256,
    }))
}

async fn download(client: &Client, cache: &Path, candidate: &Candidate) -> Result<Pending> {
    fs::create_dir_all(cache)?;
    let installer = cache.join(&candidate.asset.name);
    if let Ok(file) = verified_file(&installer, candidate) {
        return Ok(Pending {
            installer,
            _verified_file: file,
        });
    }

    let url = format!(
        "https://github.com/{REPOSITORY}/releases/download/v{}/{}",
        candidate.version, candidate.asset.name
    );
    let mut response = client.get(url).send().await?.error_for_status()?;
    if let Some(size) = response.content_length() {
        ensure!(
            size == candidate.asset.size,
            "GitHub-Downloadgrösse stimmt nicht"
        );
    }
    let partial = cache.join(format!("{}.part", Uuid::new_v4()));
    let result = async {
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&partial)?;
        let mut hash = Sha256::new();
        let mut size = 0;
        while let Some(chunk) = response.chunk().await? {
            size += chunk.len() as u64;
            if size > candidate.asset.size {
                bail!("GitHub-Download ist grösser als angekündigt");
            }
            hash.update(&chunk);
            output.write_all(&chunk)?;
        }
        ensure!(
            size == candidate.asset.size && format!("{:x}", hash.finalize()) == candidate.sha256,
            "GitHub-Installer stimmt nicht mit der veröffentlichten Prüfsumme überein"
        );
        output.sync_all()?;
        drop(output);
        fs::rename(&partial, &installer)?;
        let file = verified_file(&installer, candidate)?;
        Ok(Pending {
            installer,
            _verified_file: file,
        })
    }
    .await;
    if result.is_err() {
        let _ = fs::remove_file(partial);
    }
    result
}

fn verified_file(path: &Path, candidate: &Candidate) -> Result<File> {
    // Keep the verified installer read-only through process creation.
    let mut file = OpenOptions::new().read(true).share_mode(1).open(path)?;
    ensure!(
        file.metadata()?.len() == candidate.asset.size,
        "Installergrösse stimmt nicht"
    );
    let mut hash = Sha256::new();
    io::copy(&mut file, &mut hash)?;
    ensure!(
        format!("{:x}", hash.finalize()) == candidate.sha256,
        "Installer-Prüfsumme stimmt nicht"
    );
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str, bytes: &[u8]) -> Release {
        Release {
            tag_name: tag.into(),
            draft: false,
            prerelease: false,
            assets: vec![Asset {
                name: format!("KD-Desktop-{}-Setup.exe", &tag[1..]),
                state: "uploaded".into(),
                size: bytes.len() as u64,
                digest: Some(format!("sha256:{:x}", Sha256::digest(bytes))),
            }],
        }
    }

    #[test]
    fn only_newer_stable_releases_are_candidates() {
        let current = Version::new(1, 9, 0);
        for tag in ["v1.8.0", "v1.9.0", "v2.0.0-rc.1", "v2.0.0+dev", "vbad"] {
            assert!(
                select_release(release(tag, b"fixture"), &current)
                    .unwrap()
                    .is_none()
            );
        }
        assert_eq!(
            select_release(release("v1.10.0", b"fixture"), &current)
                .unwrap()
                .unwrap()
                .version,
            Version::new(1, 10, 0)
        );
    }

    #[test]
    fn release_requires_exact_installer_and_valid_digest() {
        let mut wrong = release("v2.0.0", b"fixture");
        wrong.assets[0].name = "other.exe".into();
        assert!(select_release(wrong, &Version::new(1, 0, 0)).is_err());
        let mut wrong = release("v2.0.0", b"fixture");
        wrong.assets[0].digest = None;
        assert!(select_release(wrong, &Version::new(1, 0, 0)).is_err());
    }

    #[test]
    fn cached_installer_must_match_hash_even_at_equal_size() {
        let cache = tempfile::tempdir().unwrap();
        let path = cache.path().join("setup.exe");
        let candidate = select_release(release("v2.0.0", b"valid"), &Version::new(1, 0, 0))
            .unwrap()
            .unwrap();
        fs::write(&path, b"wrong").unwrap();
        assert!(verified_file(&path, &candidate).is_err());
        fs::write(&path, b"valid").unwrap();
        assert!(verified_file(&path, &candidate).is_ok());
    }
}
