//! Typed Tauri event emitters. Frontend listens via `@tauri-apps/api/event`.

use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SynthesisStep {
    pub session_id: String,
    pub step: String,
    pub message: String,
    pub progress: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStateChanged {
    pub session_id: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexProgress {
    pub current: u32,
    pub total: u32,
}

pub fn emit_step(
    app: &AppHandle,
    session_id: &str,
    step: &str,
    message: &str,
    progress: u32,
    total: u32,
) {
    let _ = app.emit(
        "synthesis-progress",
        SynthesisStep {
            session_id: session_id.to_string(),
            step: step.to_string(),
            message: message.to_string(),
            progress,
            total,
        },
    );
}

pub fn emit_complete(app: &AppHandle, session_id: &str) {
    let _ = app.emit(
        "synthesis-complete",
        SynthesisStep {
            session_id: session_id.to_string(),
            step: "complete".into(),
            message: "Synthesis complete".into(),
            progress: 1,
            total: 1,
        },
    );
}

pub fn emit_session_state(app: &AppHandle, session_id: Option<&str>, status: &str) {
    let _ = app.emit(
        "session-state",
        SessionStateChanged {
            session_id: session_id.map(str::to_string),
            status: status.to_string(),
        },
    );
}

pub fn emit_index_progress(app: &AppHandle, current: u32, total: u32) {
    let _ = app.emit("vault-index-progress", IndexProgress { current, total });
}
