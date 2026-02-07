use crate::core::registry::Registry;
use crate::interactive::registry_snapshot::RegistrySnapshot;
use crate::interactive::{WatcherState, event_dispatcher};
use crate::utils::{get_runtime_handle, hash};
use notify::event::ModifyKind;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub async fn watch_file(
    watcher_state: Arc<Mutex<WatcherState>>,
) -> crate::error::Result<RecommendedWatcher> {
    let handle = get_runtime_handle();

    let watcher_state_clone = watcher_state.clone();

    let mut watcher = notify::recommended_watcher(move |res| {
        let watcher_state_clone = watcher_state_clone.clone();

        handle.spawn(async {
            tracing::debug!(
                "spawned from watcher event handler!!! state: {:#?}",
                &watcher_state_clone
            );

            let _ = process_file_change(res, watcher_state_clone).await;
        });
    })?;

    // Lock ONLY to read the path
    let registry_file_path = {
        let state = watcher_state.lock().await;
        state.registry_file_path.clone()
    };

    watcher.watch(&registry_file_path, RecursiveMode::NonRecursive)?;

    Ok(watcher)
}

async fn process_file_change(
    event_result: notify::Result<Event>,
    watcher_state: Arc<Mutex<WatcherState>>,
) -> crate::error::Result<()> {
    let event = event_result?;
    if let EventKind::Modify(ModifyKind::Data(_)) = event.kind {
        let mut watcher_state_ref = watcher_state.lock().await;

        let current_registry = Registry::from_file(&watcher_state_ref.registry_file_path).await?;
        info!("Processing file change started!");
        let current_registry_hash = hash(&current_registry);

        if current_registry_hash == watcher_state_ref.previous_snapshot.hash {
            return Ok(());
        }

        let current_registry_snapshot = RegistrySnapshot {
            registry: current_registry,
            hash: current_registry_hash,
        };

        tracing::debug!(
            "current reg snapshot from process_file_change: {:?}",
            &current_registry_snapshot
        );

        tracing::debug!(
            "previous reg snapshot from process_file_change: {:?}",
            &watcher_state_ref.previous_snapshot
        );

        event_dispatcher::on_registry_hash_change(
            &watcher_state_ref.previous_snapshot,
            &current_registry_snapshot,
        )
        .await?;

        watcher_state_ref.previous_snapshot = current_registry_snapshot;
    }
    Ok(())
}
