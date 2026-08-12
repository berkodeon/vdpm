use std::path::Path;

use crate::config_loader;
use crate::core::registry::Registry;
use crate::error::Result;
use crate::utils::get_home_dir;
use anyhow::Context;
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
    let config = config_loader::load_or_create()?;
    let rc_file = get_home_dir().join(&config.settings.rc_file);
    let mut registry = Registry::generate().await?;

    let mut plugin = registry
        .plugins
        .get(name)
        .cloned()
        .with_context(|| format!("plugin \"{name}\" is not installed"))?;
    plugin.enabled = true;
    registry.plugins.insert(name.to_string(), plugin.clone());

    registry.to_visidatarc_file(&rc_file).await?;

    info!(
        plugin = %name,
        rc_file = %rc_file.display(),
        "Plugin enabled successfully"
    );
    Ok(Table::new(vec![plugin]))
}
