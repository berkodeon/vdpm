use std::path::Path;

use crate::config_loader::{self, AppConfig};
use crate::core::registry::Registry;
use crate::error::{PluginOperationError, Result, VDPMError};
use tabled::Table;
use tracing::{info, instrument};

#[instrument(
    name = "plugin.enable",
    skip_all,
    fields(plugin = %name)
)]
pub async fn execute(name: &str) -> Result<Table> {
    info!(
        plugin = %name,
        "Starting plugin enablement"
    );
    let config: AppConfig = config_loader::load_or_create()?;
    let rc_file = &Path::new(&config.settings.rc_file);
    let mut registry = Registry::generate().await?;

    let mut plugin = registry.plugins.get(name).cloned().ok_or_else(|| {
        VDPMError::PluginError(
            format!("Error while enabling the plugin ({})", name),
            PluginOperationError::NotExistError(format!("Could not find plugin ({})", name)),
        )
    })?;
    plugin.enabled = true;
    registry.plugins.insert(name.to_string(), plugin.clone());

    registry.to_visidatarc_file(rc_file).await?;

    info!(
        plugin = %name,
        rc_file = %rc_file.display(),
        "Plugin enabled successfully"
    );
    Ok(Table::new(vec![plugin]))
}
