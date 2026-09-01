use crate::{llm::LlmClient, vault::index::VaultIndex, ArgusResult};
use notify::RecursiveMode;
use notify_debouncer_full::{new_debouncer, DebouncedEvent};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub struct VaultWatcher {
    _debouncer: notify_debouncer_full::Debouncer<
        notify::RecommendedWatcher,
        notify_debouncer_full::RecommendedCache,
    >,
}

impl VaultWatcher {
    pub fn spawn(
        vault_root: PathBuf,
        index: Arc<VaultIndex>,
        llm: Arc<dyn LlmClient>,
        embed_model: String,
    ) -> ArgusResult<Self> {
        let (tx, rx) = std::sync::mpsc::channel::<Vec<DebouncedEvent>>();

        let mut debouncer = new_debouncer(
            Duration::from_secs(10),
            None,
            move |res: Result<Vec<DebouncedEvent>, Vec<notify::Error>>| {
                if let Ok(events) = res {
                    let _ = tx.send(events);
                }
            },
        )?;
        debouncer.watch(&vault_root, RecursiveMode::Recursive)?;

        let index_clone = index.clone();
        let llm_clone = llm.clone();
        let model = embed_model.clone();
        tauri::async_runtime::spawn(async move {
            while let Ok(events) = rx.recv() {
                for e in events {
                    for path in &e.paths {
                        if path.extension().map(|x| x != "md").unwrap_or(true) {
                            continue;
                        }
                        if path.exists() {
                            if let Err(err) =
                                index_clone.reindex_file(path, llm_clone.as_ref(), &model).await
                            {
                                tracing::warn!(?path, ?err, "reindex_file failed");
                            }
                        } else {
                            let p = path.to_string_lossy().to_string();
                            let _ = index_clone.purge_file(&p);
                        }
                    }
                }
            }
        });

        Ok(VaultWatcher { _debouncer: debouncer })
    }
}
