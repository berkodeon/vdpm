use crate::core::registry::Registry;
use crate::error::{Result, VDPMError};
use crate::interactive::registry_snapshot::RegistrySnapshot;
use crate::utils::hash;
use notify::event::ModifyKind;
use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher,
};
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info};

pub fn watch_file(
    file_path: &Path,
    tx_snapshots: mpsc::Sender<RegistrySnapshot>,
) -> Result<RecommendedWatcher> {
    let file_path = file_path.to_path_buf();
    let (tx_file_events, rx_file_events) = mpsc::channel::<notify::Result<Event>>(100);

    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx_file_events.try_send(res);
    })?;

    watcher.watch(&file_path, RecursiveMode::NonRecursive)?;

    tokio::spawn(async move {
        process_events_loop(rx_file_events, file_path, tx_snapshots).await;
    });

    Ok(watcher)
}

async fn process_events_loop(
    mut rx_events: mpsc::Receiver<notify::Result<Event>>,
    file_path: PathBuf,
    tx_snapshots: mpsc::Sender<RegistrySnapshot>,
) {
    while let Some(event_result) = rx_events.recv().await {
        if let Err(e) =
            process_file_change(event_result, file_path.clone(), tx_snapshots.clone()).await
        {
            handle_error(e);
        }
    }
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
    let event = event_result?;
    if let EventKind::Modify(ModifyKind::Data(_)) = event.kind {
        let current_registry = Registry::from_file(&file_path).await?;
        info!("Processing file change started!");
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
    }
    Ok(())
}

fn handle_error(e: impl Display) {
    error!("Failed to process file change because of {}!", e);
    // TODO: @memedov our file changed process failed, shall we revert the file?
}
