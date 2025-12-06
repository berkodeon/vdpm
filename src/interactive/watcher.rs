use crate::core::registry::Registry;
use crate::error::{Result, VDPMError};
use crate::interactive::registry_snapshot::RegistrySnapshot;
use crate::utils::hash;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher};
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::error;

pub async fn watch_file(
    file_path: &Path,
    tx: mpsc::Sender<RegistrySnapshot>,
) -> NotifyResult<RecommendedWatcher> {
    let file_path = file_path.to_path_buf();
    let file_path_clone = file_path.clone();

    let mut watcher = RecommendedWatcher::new(
        move |res| handle_file_change(res, file_path_clone.clone(), tx.clone()),
        Config::default()
            .with_poll_interval(Duration::from_secs(100))
            .with_compare_contents(true),
    )?;

    watcher.watch(file_path.as_ref(), RecursiveMode::NonRecursive)?;
    Ok(watcher)
}

fn handle_file_change(
    event_result: notify::Result<Event>,
    file_path: PathBuf,
    tx: mpsc::Sender<RegistrySnapshot>,
) {
    tokio::spawn(async move {
        if let Err(e) = process_file_change(event_result, file_path, tx).await {
            handle_error(e);
        }
    });
}

async fn process_file_change(
    event_result: notify::Result<Event>,
    file_path: PathBuf,
    tx: mpsc::Sender<RegistrySnapshot>,
) -> Result<()> {
    let current_registry = Registry::from_file(&file_path).await?;
    let current_registry_hash = hash(&current_registry);
    let current_registry_snapshot = RegistrySnapshot {
        registry: current_registry,
        hash: current_registry_hash,
    };
    tx.send(current_registry_snapshot).await.map_err(|e| {
        VDPMError::RegistryFileChangeHandlerError(
            "Failed send message to registry change queue".into(),
            e,
        )
    })?;
    Ok(())
}

fn handle_error(e: impl Display) {
    error!("Failed to process file change because of {}!", e);
    // TODO: @memedov our file changed process failed, shall we revert the file?
}
