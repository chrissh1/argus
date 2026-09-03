//! All Tauri command handlers. Frontend calls these via `invoke('snake_name', ...)`.

use crate::{
    events,
    llm::{ollama::OllamaClient, LlmClient, ModelTag},
    mock,
    paths,
    screenpipe::{download_pinned_screenpipe, Screenpipe, ScreenpipeStatus},
    session::{self, SessionRecord, Status},
    settings::{self, ExclusionEntry, Settings},
    state::{AppState, CurrentSession},
    synthesis,
    vault::{index::IndexStatus, watcher::VaultWatcher},
    ArgusError, ArgusResult,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

// ---------------- Sessions ----------------

#[tauri::command]
pub async fn session_start(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> ArgusResult<SessionRecord> {
    state.require_idle()?;
    let settings = settings::get_all(&state.db)?;

    let host = settings
        .ollama_host
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            ArgusError::Other(
                "Cannot start session: No LLM host configured. Please configure your LLM in Settings.".into(),
            )
        })?;

    let model = settings
        .ollama_model
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            ArgusError::Other(
                "Cannot start session: No inference model selected. Please select an installed model in Settings.".into(),
            )
        })?;

    // Crucial pipeline requirement: ensure LLM host is reachable and configured model is installed
    let ollama = OllamaClient::new(host);
    let installed_models = ollama.ping().await.map_err(|e| {
        ArgusError::Other(format!(
            "Cannot start session: Unable to reach Ollama at {}. Error: {}",
            host, e
        ))
    })?;

    let is_installed = installed_models.iter().any(|m| model_matches(model, &m.name));
    if !is_installed {
        let names: Vec<&str> = installed_models.iter().map(|m| m.name.as_str()).collect();
        let available_str = if names.is_empty() {
            "none (no models downloaded in Ollama)".to_string()
        } else {
            names.join(", ")
        };
        return Err(ArgusError::Other(format!(
            "Cannot start session: Model '{}' is not installed in Ollama (Installed: {}). Run 'ollama pull {}' or choose an installed model in Settings.",
            model, available_str, model
        )));
    }

    let record = session::create(&state.db, settings.data_retention_days)?;

    let raw_db = paths::session_raw_db(&record.id)?;
    let binary = Screenpipe::resolve_binary(Some(&app))?;

    if let Err(e) = state.screenpipe.spawn(&binary, &raw_db) {
        session::set_status(&state.db, &record.id, Status::Interrupted)?;
        return Err(e);
    }

    *state.current.lock().unwrap() = Some(CurrentSession {
        record: record.clone(),
        started_mono: std::time::Instant::now(),
        paused_at: None,
        accumulated_paused: std::time::Duration::ZERO,
    });

    events::emit_session_state(&app, Some(&record.id), "active");
    Ok(record)
}

#[tauri::command]
pub async fn session_pause(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> ArgusResult<SessionRecord> {
    let mut guard = state.current.lock().unwrap();
    let cur = guard
        .as_mut()
        .ok_or_else(|| ArgusError::InvalidState("no active session".into()))?;
    if cur.record.status == Status::Paused {
        return Ok(cur.record.clone());
    }
    state.screenpipe.pause()?;
    cur.paused_at = Some(std::time::Instant::now());
    cur.record.status = Status::Paused;
    session::set_status(&state.db, &cur.record.id, Status::Paused)?;
    events::emit_session_state(&app, Some(&cur.record.id), "paused");
    Ok(cur.record.clone())
}

#[tauri::command]
pub async fn session_resume(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> ArgusResult<SessionRecord> {
    let mut guard = state.current.lock().unwrap();
    let cur = guard
        .as_mut()
        .ok_or_else(|| ArgusError::InvalidState("no active session".into()))?;
    if let Some(pt) = cur.paused_at.take() {
        cur.accumulated_paused += pt.elapsed();
    }
    state.screenpipe.resume()?;
    cur.record.status = Status::Active;
    session::set_status(&state.db, &cur.record.id, Status::Active)?;
    events::emit_session_state(&app, Some(&cur.record.id), "active");
    Ok(cur.record.clone())
}

#[tauri::command]
pub async fn session_stop(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> ArgusResult<SessionRecord> {
    let (session_id, duration_secs, paused_secs, too_short) = {
        let mut guard = state.current.lock().unwrap();
        let cur = guard
            .take()
            .ok_or_else(|| ArgusError::InvalidState("no active session".into()))?;
        let dur = cur.duration_secs();
        let pause = cur.paused_secs();
        let min_secs =
            settings::get_string(&state.db, "min_session_seconds")?
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(60);
        (cur.record.id, dur, pause, dur < min_secs)
    };

    state.screenpipe.stop()?;

    if too_short {
        session::set_status(&state.db, &session_id, Status::Interrupted)?;
        events::emit_session_state(&app, None, "idle");
        return Err(ArgusError::InvalidState(format!(
            "session too short ({duration_secs}s); discarded"
        )));
    }

    session::mark_ended(&state.db, &session_id, duration_secs, paused_secs)?;
    events::emit_session_state(&app, Some(&session_id), "synthesizing");

    let app_clone = app.clone();
    let state_arc = Arc::clone(&*state);
    let id_clone = session_id.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = synthesis::synthesize(app_clone.clone(), state_arc, id_clone.clone()).await {
            tracing::error!(?e, session=%id_clone, "synthesis failed");
            events::emit_step(&app_clone, &id_clone, "error", &format!("{e}"), 0, 0);
        }
    });

    session::get(&state.db, &session_id)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentSessionSnapshot {
    pub record: SessionRecord,
    pub duration_secs: i64,
    pub paused_secs: i64,
}

#[tauri::command]
pub async fn session_current(
    state: State<'_, Arc<AppState>>,
) -> ArgusResult<Option<CurrentSessionSnapshot>> {
    let guard = state.current.lock().unwrap();
    Ok(guard.as_ref().map(|cur| CurrentSessionSnapshot {
        record: cur.record.clone(),
        duration_secs: cur.duration_secs(),
        paused_secs: cur.paused_secs(),
    }))
}

#[tauri::command]
pub async fn session_list(
    state: State<'_, Arc<AppState>>,
    limit: Option<u32>,
) -> ArgusResult<Vec<SessionRecord>> {
    session::list(&state.db, limit.unwrap_or(100))
}

#[tauri::command]
pub async fn session_get(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> ArgusResult<SessionRecord> {
    session::get(&state.db, &id)
}

#[tauri::command]
pub async fn session_rename(
    state: State<'_, Arc<AppState>>,
    id: String,
    name: String,
) -> ArgusResult<SessionRecord> {
    session::rename(&state.db, &id, &name)?;
    session::get(&state.db, &id)
}

/// Seed a mock session and immediately trigger background synthesis for development testing.
#[tauri::command]
pub async fn dev_seed_mock_session(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    duration_mins: Option<u32>,
) -> ArgusResult<SessionRecord> {
    let record = mock::create_mock_session(&state.db, duration_mins.unwrap_or(25))?;
    events::emit_session_state(&app, Some(&record.id), "synthesizing");

    let app_clone = app.clone();
    let state_arc = Arc::clone(&*state);
    let id_clone = record.id.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = synthesis::synthesize(app_clone.clone(), state_arc, id_clone.clone()).await {
            tracing::error!(?e, session=%id_clone, "mock synthesis failed");
            events::emit_step(&app_clone, &id_clone, "error", &format!("{e}"), 0, 0);
        }
    });

    Ok(record)
}

// ---------------- Screenpipe Binary Downloader & Status ----------------

#[tauri::command]
pub async fn screenpipe_status(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> ArgusResult<ScreenpipeStatus> {
    Ok(state.screenpipe.status(Some(&app)))
}

#[tauri::command]
pub async fn screenpipe_download(
    expected_sha256: Option<String>,
) -> ArgusResult<String> {
    let path = download_pinned_screenpipe(expected_sha256.as_deref()).await?;
    Ok(path.to_string_lossy().to_string())
}

// ---------------- Settings ----------------

#[tauri::command]
pub async fn settings_get_all(state: State<'_, Arc<AppState>>) -> ArgusResult<Settings> {
    settings::get_all(&state.db)
}

#[tauri::command]
pub async fn settings_set(
    state: State<'_, Arc<AppState>>,
    key: String,
    value: String,
) -> ArgusResult<()> {
    settings::set_string(&state.db, &key, &value)
}

// ---------------- Vault ----------------

#[tauri::command]
pub async fn vault_choose(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    path: String,
) -> ArgusResult<Settings> {
    let p = PathBuf::from(&path);
    if !p.is_dir() {
        return Err(ArgusError::Other(format!("not a directory: {path}")));
    }
    settings::set_string(&state.db, "vault_path", &path)?;
    let settings = settings::get_all(&state.db)?;

    if let (Some(host), Some(embed)) = (&settings.ollama_host, &settings.embed_model) {
        let ollama: Arc<dyn LlmClient> = Arc::new(OllamaClient::new(host));
        let watcher = VaultWatcher::spawn(
            p.clone(),
            state.vault_index.clone(),
            ollama,
            embed.clone(),
        )?;
        *state.vault_watcher.lock().unwrap() = Some(watcher);
    }

    // Kick off initial indexing in the background if LLM is configured.
    if settings.ollama_host.is_some() && settings.embed_model.is_some() {
        let app_clone = app.clone();
        let state_arc = Arc::clone(&*state);
        let settings_clone = settings.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = run_reindex(app_clone, state_arc, settings_clone).await {
                tracing::error!(?e, "initial vault index failed");
            }
        });
    }

    Ok(settings)
}

#[tauri::command]
pub async fn vault_reindex(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> ArgusResult<()> {
    let settings = settings::get_all(&state.db)?;
    let state_arc = Arc::clone(&*state);
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run_reindex(app, state_arc, settings).await {
            tracing::error!(?e, "vault reindex failed");
        }
    });
    Ok(())
}

async fn run_reindex(
    app: AppHandle,
    state: Arc<AppState>,
    settings: Settings,
) -> ArgusResult<()> {
    let vault_path = settings
        .vault_path
        .clone()
        .ok_or(ArgusError::PathNotConfigured("vault_path"))?;
    let host = settings
        .ollama_host
        .as_deref()
        .ok_or(ArgusError::PathNotConfigured("ollama_host"))?;
    let embed = settings
        .embed_model
        .as_deref()
        .ok_or(ArgusError::PathNotConfigured("embed_model"))?;

    {
        let mut p = state.indexing.lock().unwrap();
        p.active = true;
        p.current = 0;
        p.total = 0;
    }
    let ollama = OllamaClient::new(host);
    let app_for_cb = app.clone();
    let state_for_cb = state.clone();
    let res = state
        .vault_index
        .reindex(
            std::path::Path::new(&vault_path),
            &ollama,
            embed,
            move |cur, total| {
                events::emit_index_progress(&app_for_cb, cur, total);
                let mut p = state_for_cb.indexing.lock().unwrap();
                p.current = cur;
                p.total = total;
            },
        )
        .await;
    {
        let mut p = state.indexing.lock().unwrap();
        p.active = false;
    }
    res
}

#[tauri::command]
pub async fn vault_index_status(
    state: State<'_, Arc<AppState>>,
) -> ArgusResult<IndexStatus> {
    let vault_path = settings::get_string(&state.db, "vault_path")?;
    let mut status = state.vault_index.status(vault_path.as_deref())?;
    let p = state.indexing.lock().unwrap();
    status.indexing = p.active;
    status.progress_current = p.current;
    status.progress_total = p.total;
    Ok(status)
}

// ---------------- Ollama ----------------

pub fn model_matches(configured: &str, installed: &str) -> bool {
    let conf = configured.trim();
    let inst = installed.trim();
    if conf.eq_ignore_ascii_case(inst) {
        return true;
    }
    if !conf.contains(':') {
        let with_latest = format!("{conf}:latest");
        if with_latest.eq_ignore_ascii_case(inst) {
            return true;
        }
    }
    if let Some((base, _)) = inst.split_once(':') {
        if conf.eq_ignore_ascii_case(base) {
            return true;
        }
    }
    false
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaTestResult {
    pub ok: bool,
    pub models: Vec<ModelTag>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OllamaTestPayload {
    pub host: String,
    pub model: Option<String>,
}

#[tauri::command]
pub async fn ollama_test(payload: OllamaTestPayload) -> ArgusResult<OllamaTestResult> {
    let host = payload.host.trim();
    if host.is_empty() {
        return Ok(OllamaTestResult {
            ok: false,
            models: vec![],
            error: Some("Please enter an Ollama host URL.".into()),
        });
    }
    let ollama = OllamaClient::new(host);
    let models = match ollama.ping().await {
        Ok(m) => m,
        Err(e) => {
            return Ok(OllamaTestResult {
                ok: false,
                models: vec![],
                error: Some(format!("Failed to connect to Ollama at {host}: {e}")),
            });
        }
    };

    if let Some(model) = payload.model.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        if !models.iter().any(|m| model_matches(model, &m.name)) {
            let names: Vec<&str> = models.iter().map(|m| m.name.as_str()).collect();
            let available_str = if names.is_empty() {
                "no models installed".to_string()
            } else {
                names.join(", ")
            };
            return Ok(OllamaTestResult {
                ok: false,
                models,
                error: Some(format!(
                    "Connected to Ollama, but model '{model}' is not installed (Available: {available_str}). Run 'ollama pull {model}' to install."
                )),
            });
        }
    }

    Ok(OllamaTestResult {
        ok: true,
        models,
        error: None,
    })
}

#[tauri::command]
pub async fn ollama_models(state: State<'_, Arc<AppState>>) -> ArgusResult<Vec<ModelTag>> {
    let settings = settings::get_all(&state.db)?;
    let Some(host) = settings.ollama_host.filter(|s| !s.trim().is_empty()) else {
        return Ok(vec![]);
    };
    let ollama = OllamaClient::new(host);
    ollama.ping().await
}

// ---------------- Exclusion list ----------------

#[tauri::command]
pub async fn exclusion_add(
    state: State<'_, Arc<AppState>>,
    entry: ExclusionEntry,
) -> ArgusResult<Vec<ExclusionEntry>> {
    let mut list = settings::get_exclusion_list(&state.db)?;
    if !list.iter().any(|e| e.bundle_id == entry.bundle_id) {
        list.push(entry);
    }
    settings::set_json(&state.db, "exclusion_list", &list)?;
    Ok(list)
}

#[tauri::command]
pub async fn exclusion_remove(
    state: State<'_, Arc<AppState>>,
    bundle_id: String,
) -> ArgusResult<Vec<ExclusionEntry>> {
    let mut list = settings::get_exclusion_list(&state.db)?;
    list.retain(|e| e.bundle_id != bundle_id);
    settings::set_json(&state.db, "exclusion_list", &list)?;
    Ok(list)
}

#[tauri::command]
pub async fn llm_status(state: State<'_, Arc<AppState>>) -> ArgusResult<OllamaTestResult> {
    let settings = settings::get_all(&state.db)?;
    let host = match settings.ollama_host.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(h) => h,
        None => {
            return Ok(OllamaTestResult {
                ok: false,
                models: vec![],
                error: Some("No LLM host configured. Please configure your LLM in Settings.".into()),
            });
        }
    };

    let ollama = OllamaClient::new(host);
    let models = match ollama.ping().await {
        Ok(m) => m,
        Err(e) => {
            return Ok(OllamaTestResult {
                ok: false,
                models: vec![],
                error: Some(format!("Cannot reach Ollama at {host}: {e}")),
            });
        }
    };

    let model = match settings.ollama_model.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(m) => m,
        None => {
            return Ok(OllamaTestResult {
                ok: false,
                models,
                error: Some("No inference model selected. Please select an installed model in Settings.".into()),
            });
        }
    };

    if !models.iter().any(|m| model_matches(model, &m.name)) {
        let names: Vec<&str> = models.iter().map(|m| m.name.as_str()).collect();
        let available_str = if names.is_empty() {
            "none (no models downloaded)".to_string()
        } else {
            names.join(", ")
        };
        return Ok(OllamaTestResult {
            ok: false,
            models,
            error: Some(format!(
                "Model '{model}' is not installed in Ollama (Available: {available_str}). Run 'ollama pull {model}' or select an installed model in Settings."
            )),
        });
    }

    Ok(OllamaTestResult {
        ok: true,
        models,
        error: None,
    })
}

// ---------------- Window helpers ----------------

#[tauri::command]
pub async fn open_dashboard(app: AppHandle) -> ArgusResult<()> {
    if let Some(w) = app.get_webview_window("dashboard") {
        let _ = w.show();
        let _ = w.set_focus();
    }
    Ok(())
}

#[tauri::command]
pub async fn hide_menubar(app: AppHandle) -> ArgusResult<()> {
    if let Some(w) = app.get_webview_window("menubar") {
        let _ = w.hide();
    }
    Ok(())
}

#[tauri::command]
pub async fn note_read(path: String) -> ArgusResult<String> {
    let p = std::path::Path::new(&path);
    if !p.is_file() {
        return Err(ArgusError::Other(format!("Note file not found: {path}")));
    }
    std::fs::read_to_string(p).map_err(|e| ArgusError::Other(format!("Failed to read note {path}: {e}")))
}

#[tauri::command]
pub async fn open_in_obsidian(
    state: State<'_, Arc<AppState>>,
    note_path: Option<String>,
) -> ArgusResult<()> {
    let vault = settings::require_vault_path(&state.db)?;
    let url = match note_path {
        Some(p) => {
            let file = std::path::Path::new(&p)
                .strip_prefix(&vault)
                .unwrap_or_else(|_| std::path::Path::new(&p))
                .to_string_lossy()
                .to_string();
            format!(
                "obsidian://open?vault={}&file={}",
                urlencode(vault_name(&vault)),
                urlencode(&file)
            )
        }
        None => format!("obsidian://open?vault={}", urlencode(vault_name(&vault))),
    };
    open::that(url).map_err(|e| ArgusError::Other(format!("open obsidian: {e}")))?;
    Ok(())
}

fn vault_name(p: &str) -> &str {
    std::path::Path::new(p)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(p)
}

fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}
