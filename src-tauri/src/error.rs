use serde::{Serialize, Serializer};
use thiserror::Error;

pub type ArgusResult<T> = Result<T, ArgusError>;

#[derive(Debug, Error)]
pub enum ArgusError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("http: {0}")]
    Http(#[from] reqwest::Error),

    #[error("watcher: {0}")]
    Watcher(#[from] notify::Error),

    #[error("tauri: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("path not configured: {0}")]
    PathNotConfigured(&'static str),

    #[error("invalid state: {0}")]
    InvalidState(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Other(String),
}

impl Serialize for ArgusError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl From<anyhow::Error> for ArgusError {
    fn from(e: anyhow::Error) -> Self {
        ArgusError::Other(e.to_string())
    }
}
