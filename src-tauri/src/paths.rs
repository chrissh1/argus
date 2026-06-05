//! Canonical filesystem paths for Argus state. Everything lives under `~/.argus/`.

use crate::{ArgusError, ArgusResult};
use std::path::{Path, PathBuf};
use tauri::Manager;

pub fn argus_root() -> ArgusResult<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| ArgusError::Other("no home dir".into()))?;
    let root = home.join(".argus");
    std::fs::create_dir_all(&root)?;
    Ok(root)
}

pub fn app_db_path() -> ArgusResult<PathBuf> {
    Ok(argus_root()?.join("app.db"))
}

pub fn sessions_dir() -> ArgusResult<PathBuf> {
    let dir = argus_root()?.join("sessions");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn session_dir(session_id: &str) -> ArgusResult<PathBuf> {
    let dir = sessions_dir()?.join(session_id);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn session_raw_db(session_id: &str) -> ArgusResult<PathBuf> {
    Ok(session_dir(session_id)?.join("raw.db"))
}

pub fn vault_index_db() -> ArgusResult<PathBuf> {
    let dir = argus_root()?.join("vault-index");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("vectors.db"))
}

pub fn screenpipe_binary(app_handle: &tauri::AppHandle) -> ArgusResult<PathBuf> {
    let path = app_handle
        .path()
        .resolve("bin/screenpipe", tauri::path::BaseDirectory::Resource)
        .map_err(|e| ArgusError::Other(format!("resolve screenpipe: {e}")))?;
    Ok(path)
}

#[allow(dead_code)]
pub fn ensure_dir(p: &Path) -> ArgusResult<()> {
    std::fs::create_dir_all(p)?;
    Ok(())
}
