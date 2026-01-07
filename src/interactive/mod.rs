use crate::config_loader;
use crate::error::Result;
use crate::fs::paths::get_registry_file_path;
use crate::utils::{get_home_dir, hash};
use crate::{config_loader::AppConfig, core::registry::Registry};
use notify::RecommendedWatcher;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use tokio::sync::mpsc;
use tracing::info;
mod event_dispatcher;
pub mod registry_snapshot;
mod watcher;
use registry_snapshot::RegistrySnapshot;

pub async fn launch(app_config: AppConfig) -> Result<(Child, RecommendedWatcher)> {
    info!("Launchin interactive mode!");
    let config: AppConfig = config_loader::load_or_create()?;
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

    let (tx, mut rx) = mpsc::channel::<RegistrySnapshot>(1);
    info!("Before starting watching!");
    let watcher: RecommendedWatcher = watcher::watch_file(&registry_file_path, tx.clone())?;

    event_dispatcher::listen(rx, last_processed_registry_snapshot);

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
