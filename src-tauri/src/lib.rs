//! Argus — Rust backend entrypoint.
//!
//! All stateful work lives in Rust. The Svelte frontend talks to us only via
//! Tauri commands (`commands::*`) and events (`events::*`).

pub mod commands;
pub mod db;
pub mod error;
pub mod events;
pub mod llm;
pub mod mock;
pub mod paths;
pub mod screenpipe;
pub mod session;
pub mod settings;
pub mod state;
pub mod synthesis;
pub mod tray;
pub mod vault;

pub use error::{ArgusError, ArgusResult};

use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "argus=info,warn".into()),
        )
        .init();

    let context = tauri::generate_context!();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let app_state = tauri::async_runtime::block_on(async move {
                state::AppState::initialize(handle).await
            })?;
            app.manage(Arc::new(app_state));

            tray::install(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::session_start,
            commands::session_stop,
            commands::session_pause,
            commands::session_resume,
            commands::session_current,
            commands::session_list,
            commands::session_get,
            commands::session_rename,
            commands::dev_seed_mock_session,
            commands::screenpipe_status,
            commands::screenpipe_download,
            commands::settings_get_all,
            commands::settings_set,
            commands::vault_choose,
            commands::vault_reindex,
            commands::vault_index_status,
            commands::ollama_test,
            commands::ollama_models,
            commands::exclusion_add,
            commands::exclusion_remove,
            commands::open_dashboard,
            commands::open_in_obsidian,
        ])
        .build(context)
        .expect("failed to build Argus")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Reopen { .. } = event {
                if let Some(w) = app_handle.get_webview_window("dashboard") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        });
}
