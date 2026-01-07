use crate::cli::commands::disable;
use crate::config_loader::{self, AppConfig};
use crate::error::{RegistryError, Result, VDPMError};
use crate::utils::get_home_dir;
use tabled::Table;
use tracing::info;

pub async fn execute(name: &str) -> Result<Table> {
    info!("Uninstall plugin({})!", name);
    let config: AppConfig = config_loader::load_or_create()?;
    let plugin_folder = get_home_dir().join(&config.settings.plugin_folder);
    let plugin_file_path = plugin_folder.join(format!("{}.py", name));

    let disabled_plugin_result: Table = disable::execute(name).await?;

    tokio::fs::remove_file(plugin_file_path)
        .await
        .map_err(|e| {
            VDPMError::RegistryOperationError(
                format!("Failed to delete the plugin({})", name),
                RegistryError::from(e),
            )
        })?;

    Ok(disabled_plugin_result)
}
