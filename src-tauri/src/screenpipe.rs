//! Manage the Screenpipe binary as a per-session child process.
//! Supports running from ~/.argus/bin/screenpipe or bundled app resources,
//! as well as on-demand download of pinned versions.

use crate::{paths, ArgusError, ArgusResult};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

pub const PINNED_SCREENPIPE_VERSION: &str = "v0.1.72";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenpipeStatus {
    pub installed: bool,
    pub path: Option<String>,
    pub pinned_version: String,
    pub running: bool,
}

pub struct Screenpipe {
    child: Mutex<Option<Child>>,
}

impl Screenpipe {
    pub const fn new() -> Self {
        Screenpipe { child: Mutex::new(None) }
    }

    /// Resolve the Screenpipe binary path.
    /// Checks ~/.argus/bin/screenpipe first, then bundled app resources.
    pub fn resolve_binary(app_handle: Option<&AppHandle>) -> ArgusResult<PathBuf> {
        let user_bin = paths::argus_root()?.join("bin").join("screenpipe");
        if user_bin.exists() && is_executable(&user_bin) {
            return Ok(user_bin);
        }

        if let Some(app) = app_handle {
            if let Ok(bundled) = app.path().resolve("bin/screenpipe", tauri::path::BaseDirectory::Resource) {
                if bundled.exists() && is_executable(&bundled) {
                    return Ok(bundled);
                }
            }
        }

        // Return the expected user binary path for error messaging
        Ok(user_bin)
    }

    pub fn status(&self, app_handle: Option<&AppHandle>) -> ScreenpipeStatus {
        let path_res = Self::resolve_binary(app_handle);
        let (installed, path) = match path_res {
            Ok(p) if p.exists() && is_executable(&p) => (true, Some(p.to_string_lossy().to_string())),
            _ => (false, None),
        };
        ScreenpipeStatus {
            installed,
            path,
            pinned_version: PINNED_SCREENPIPE_VERSION.into(),
            running: self.is_running(),
        }
    }

    pub fn spawn(&self, binary: &Path, raw_db: &Path) -> ArgusResult<()> {
        let mut guard = self.child.lock().unwrap();
        if guard.is_some() {
            return Err(ArgusError::InvalidState(
                "screenpipe already running".into(),
            ));
        }
        if !binary.exists() {
            return Err(ArgusError::NotFound(format!(
                "Screenpipe binary not found at: {}. Please download it in Settings.",
                binary.display()
            )));
        }

        let parent = raw_db.parent().ok_or_else(|| {
            ArgusError::Other(format!("raw_db has no parent: {}", raw_db.display()))
        })?;
        std::fs::create_dir_all(parent)?;

        let child = Command::new(binary)
            .arg("--start-audio")
            .arg("--ocr")
            .arg("--db")
            .arg(raw_db)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        tracing::info!(pid = child.id(), "screenpipe spawned");
        *guard = Some(child);
        Ok(())
    }

    pub fn pause(&self) -> ArgusResult<()> {
        self.signal(Signal::SIGSTOP)
    }

    pub fn resume(&self) -> ArgusResult<()> {
        self.signal(Signal::SIGCONT)
    }

    fn signal(&self, sig: Signal) -> ArgusResult<()> {
        let guard = self.child.lock().unwrap();
        let child = guard
            .as_ref()
            .ok_or_else(|| ArgusError::InvalidState("screenpipe not running".into()))?;
        kill(Pid::from_raw(child.id() as i32), sig)
            .map_err(|e| ArgusError::Other(format!("kill({sig:?}): {e}")))?;
        Ok(())
    }

    pub fn stop(&self) -> ArgusResult<()> {
        let mut guard = self.child.lock().unwrap();
        let Some(mut child) = guard.take() else {
            return Ok(());
        };

        // SIGCONT first in case we were paused
        let pid = Pid::from_raw(child.id() as i32);
        let _ = kill(pid, Signal::SIGCONT);
        let _ = kill(pid, Signal::SIGTERM);

        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match child.try_wait()? {
                Some(_) => return Ok(()),
                None if Instant::now() >= deadline => break,
                None => std::thread::sleep(Duration::from_millis(100)),
            }
        }
        let _ = child.kill();
        let _ = child.wait();
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.child.lock().unwrap().is_some()
    }
}

fn is_executable(path: &Path) -> bool {
    if let Ok(meta) = path.metadata() {
        // Must be a file and have executable bit (0o111)
        meta.is_file() && (meta.permissions().mode() & 0o111 != 0)
    } else {
        false
    }
}

/// Download pinned Screenpipe binary to ~/.argus/bin/screenpipe
pub async fn download_pinned_screenpipe(
    expected_sha256: Option<&str>,
) -> ArgusResult<PathBuf> {
    let bin_dir = paths::argus_root()?.join("bin");
    std::fs::create_dir_all(&bin_dir)?;
    let target_bin = bin_dir.join("screenpipe");

    let arch = std::env::consts::ARCH;
    let download_url = format!(
        "https://github.com/mediar-ai/screenpipe/releases/download/{}/screenpipe-{}-apple-darwin",
        PINNED_SCREENPIPE_VERSION, arch
    );

    tracing::info!(url = %download_url, "downloading Screenpipe binary");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| ArgusError::Other(e.to_string()))?;

    let res = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| ArgusError::Other(format!("Failed to download screenpipe: {e}")))?
        .error_for_status()
        .map_err(|e| ArgusError::Other(format!("Download HTTP error: {e}")))?;

    let bytes = res
        .bytes()
        .await
        .map_err(|e| ArgusError::Other(format!("Failed reading response stream: {e}")))?;

    // Checksum verification
    if let Some(expected) = expected_sha256 {
        let hash = hex::encode(Sha256::digest(&bytes));
        if !hash.eq_ignore_ascii_case(expected) {
            return Err(ArgusError::Other(format!(
                "SHA-256 mismatch for downloaded binary. Expected {expected}, got {hash}"
            )));
        }
    }

    let temp_target = bin_dir.join(format!(".screenpipe-tmp-{}", uuid::Uuid::new_v4().simple()));
    std::fs::write(&temp_target, &bytes)?;

    // Set permissions to rwxr-xr-x (0o755)
    let mut perms = std::fs::metadata(&temp_target)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&temp_target, perms)?;

    std::fs::rename(&temp_target, &target_bin)?;

    // Clear macOS quarantine attribute if present
    let _ = Command::new("xattr")
        .args(["-d", "com.apple.quarantine"])
        .arg(&target_bin)
        .output();

    tracing::info!(path = %target_bin.display(), "Screenpipe installed successfully");
    Ok(target_bin)
}
