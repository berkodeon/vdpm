use crate::core::registry::Registry;
use crate::error::Result;
use crate::interactive::registry_snapshot::RegistrySnapshot;
use crate::interactive::{WatcherState, event_dispatcher};
use crate::utils::hash;
use notify::event::ModifyKind;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fmt::Display;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::Mutex;
use tracing::error;

pub async fn watch_file(watcher_state: Arc<Mutex<WatcherState>>) -> Result<RecommendedWatcher> {
    let registry_file_path = watcher_state.lock().await.registry_file_path.clone();
    let runtime_handle: Handle = Handle::current();

    let mut watcher = notify::recommended_watcher(move |event_result| {
        let watcher_state = watcher_state.clone();

        runtime_handle.spawn(async move {
            if let Err(e) = process_file_change(event_result, watcher_state).await {
                handle_error(e);
            }
        });
    })?;

    watcher.watch(&registry_file_path, RecursiveMode::NonRecursive)?;

    Ok(watcher)
}

async fn process_file_change(
    event_result: notify::Result<Event>,
    watcher_state: Arc<Mutex<WatcherState>>,
) -> Result<()> {
    let event = event_result?;
    if let EventKind::Modify(ModifyKind::Data(_)) = event.kind {
        let mut watcher_state = watcher_state.lock().await;

        let current_registry = Registry::from_file(&watcher_state.registry_file_path).await?;
        let current_snapshot = RegistrySnapshot {
            hash: hash(&current_registry),
            registry: current_registry,
        };

        if current_snapshot.hash == watcher_state.previous_snapshot.hash {
            return Ok(());
        }

        event_dispatcher::process_diff(&watcher_state.previous_snapshot, &current_snapshot)
            .await?;

        watcher_state.previous_snapshot = current_snapshot;
    }
    Ok(())
}

fn handle_error(e: impl Display) {
    error!("Failed to process file change because of {}!", e);
    // TODO: @memedov our file changed process failed, shall we revert the file?
}
