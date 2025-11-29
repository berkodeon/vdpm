use crate::error::Result;
use crate::fs::paths::get_registry_file_path;
use crate::{config_loader::AppConfig, core::registry::Registry};
use tracing::info;

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

pub async fn launch(app_config: AppConfig) -> Result<Child> {
    info!("Launchin interactive mode!");
    let registry_file_path: PathBuf = get_registry_file_path()?;
    Registry::generate()
        .await?
        .to_file(&registry_file_path)
        .await?;
    let child = Command::new("vd")
        .arg(&registry_file_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to start VisiData");

    info!("Visidata stopped!");
    Ok(child)
}
