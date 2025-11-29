use crate::error::Result;
use crate::fs::paths::get_plugin_manager_file_path;
use crate::{config_loader::AppConfig, core::registery::Registery};
use tracing::info;

use std::process::{Child, Command, Stdio};

pub async fn launch(app_config: AppConfig) -> Result<Child> {
    info!("Launchin interactive mode!");
    let plugin_manager_file_path = get_plugin_manager_file_path()?;
    let child = Command::new("vd")
        .arg(&plugin_manager_file_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to start VisiData");

    info!("Visidata stopped!");
    Ok(child)
}
