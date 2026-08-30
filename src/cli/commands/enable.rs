use crate::core::plugin::PluginName;
use crate::core::registry::Registry;
use crate::error::Result;
use anyhow::Context;
use tabled::Table;
use tracing::{info, instrument};

#[instrument(
    name = "plugin.enable",
    skip_all,
    fields(plugin = %name)
)]
pub async fn execute(name: &PluginName) -> Result<Table> {
    info!(
        plugin = %name,
        "Starting plugin enablement"
    );
    let mut registry = Registry::generate().await?;

    let mut plugin = registry
        .plugins
        .get(name)
        .cloned()
        .with_context(|| format!("plugin \"{name}\" is not installed"))?;
    plugin.enabled = true;
    registry.plugins.insert(plugin.name.clone(), plugin.clone());

    registry.persist().await?;

    info!(
        plugin = %name,
        "Plugin enabled successfully"
    );
    Ok(Table::new(vec![plugin]))
}
