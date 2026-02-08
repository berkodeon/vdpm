use crate::config_loader;
use crate::core::registry::Registry;
use crate::error::Result;
use crate::fs::paths::get_registry_file_path;
use crate::utils::{get_home_dir, hash};
use notify::RecommendedWatcher;
use registry_snapshot::RegistrySnapshot;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

mod event_dispatcher;
pub mod registry_snapshot;
mod watcher;

#[derive(Debug)]
pub struct WatcherState {
    registry_file_path: PathBuf,
    previous_snapshot: RegistrySnapshot,
}

pub async fn launch() -> Result<(Child, RecommendedWatcher)> {
    info!("Launchin interactive mode!");
    let config = config_loader::load_or_create()?;
    let plugin_folder: PathBuf = get_home_dir().join(&config.settings.plugin_folder);
    let visidata_dir = plugin_folder
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("~");

    let registry_file_path: PathBuf = get_registry_file_path()?;
    let registry = Registry::generate().await?;
    let last_processed_registry_snapshot = RegistrySnapshot {
        hash: hash(&registry),
        registry: registry.clone(),
    };

    registry.to_file(&registry_file_path).await?;

    let watcher_state = Arc::new(Mutex::new(WatcherState {
        registry_file_path: registry_file_path.clone(),
        previous_snapshot: last_processed_registry_snapshot,
    }));

    info!("Before starting watching!");
    let watcher: RecommendedWatcher = watcher::watch_file(watcher_state.clone()).await?;

    let child = Command::new("vd")
        .arg(&registry_file_path)
        .arg(format!("--visidata-dir={visidata_dir}"))
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to start VisiData");

    Ok((child, watcher))
}
