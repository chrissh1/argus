//! Shared application state held inside Tauri's managed map.

use crate::{
    db::Db,
    ollama::Ollama,
    screenpipe::Screenpipe,
    session::{SessionRecord, Status},
    settings,
    vault::{index::VaultIndex, watcher::VaultWatcher},
    ArgusError, ArgusResult,
};
use std::sync::Arc;
use std::sync::Mutex;
use tauri::AppHandle;

/// Embedding dimension for nomic-embed-text. If a user switches to a different
/// model, vault must be re-indexed; we keep one dimension per vault index file.
const DEFAULT_EMBED_DIM: usize = 768;

pub struct CurrentSession {
    pub record: SessionRecord,
    pub started_mono: std::time::Instant,
    pub paused_at: Option<std::time::Instant>,
    pub accumulated_paused: std::time::Duration,
}

impl CurrentSession {
    pub fn duration_secs(&self) -> i64 {
        let mut elapsed = self.started_mono.elapsed();
        if let Some(p) = self.paused_at {
            elapsed -= p.elapsed();
        }
        elapsed -= self.accumulated_paused;
        elapsed.as_secs() as i64
    }
    pub fn paused_secs(&self) -> i64 {
        let mut acc = self.accumulated_paused;
        if let Some(p) = self.paused_at {
            acc += p.elapsed();
        }
        acc.as_secs() as i64
    }
}

pub struct AppState {
    pub app: AppHandle,
    pub db: Db,
    pub screenpipe: Screenpipe,
    pub vault_index: Arc<VaultIndex>,
    pub current: Mutex<Option<CurrentSession>>,
    pub vault_watcher: Mutex<Option<VaultWatcher>>,
    pub indexing: Mutex<IndexingProgress>,
}

#[derive(Default, Clone)]
pub struct IndexingProgress {
    pub active: bool,
    pub current: u32,
    pub total: u32,
}

impl AppState {
    pub async fn initialize(app: AppHandle) -> ArgusResult<Self> {
        let db = Db::open()?;
        let _ = crate::session::vacuum_expired_raw_dbs(&db);

        let vault_index = Arc::new(VaultIndex::open(DEFAULT_EMBED_DIM)?);

        let state = AppState {
            app: app.clone(),
            db,
            screenpipe: Screenpipe::new(),
            vault_index: vault_index.clone(),
            current: Mutex::new(None),
            vault_watcher: Mutex::new(None),
            indexing: Mutex::new(IndexingProgress::default()),
        };

        // If a vault is already configured, attach a watcher so the index stays warm.
        if let Some(path) = settings::get_string(&state.db, "vault_path")? {
            let settings = settings::get_all(&state.db)?;
            let ollama = Arc::new(Ollama::new(&settings.ollama_host));
            let watcher = VaultWatcher::spawn(
                path.into(),
                vault_index,
                ollama,
                settings.embed_model.clone(),
            )?;
            *state.vault_watcher.lock().unwrap() = Some(watcher);
        }

        Ok(state)
    }

    pub fn require_idle(&self) -> ArgusResult<()> {
        if self.current.lock().unwrap().is_some() {
            return Err(ArgusError::InvalidState(
                "a session is already active".into(),
            ));
        }
        Ok(())
    }

    pub fn current_status(&self) -> Status {
        self.current
            .lock()
            .unwrap()
            .as_ref()
            .map(|s| s.record.status)
            .unwrap_or(Status::Idle)
    }
}
