use crate::cli::commands::disable;
use crate::config_loader;
use crate::core::plugin::{Plugin, PluginName};
use crate::error::Result;
use crate::utils::get_home_dir;
use anyhow::Context;
use tabled::Table;
use tracing::info;

pub async fn execute(name: &PluginName) -> Result<Table> {
    let plugin = Plugin::new(name.clone(), false, false, None);

    info!("Uninstall plugin({})!", plugin.name);
    let config = config_loader::load_or_create()?;
    let plugin_folder = get_home_dir().join(&config.settings.plugin_folder);
    let plugin_file_path = plugin_folder.join(format!("{}.py", plugin.name));

    let disabled_plugin_result: Table = disable::execute(&plugin.name).await?;

    tokio::fs::remove_file(plugin_file_path)
        .await
        .with_context(|| format!("failed to delete plugin file for \"{}\"", plugin.name))?;

    Ok(disabled_plugin_result)
}
