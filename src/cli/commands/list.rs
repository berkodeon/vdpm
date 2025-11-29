use crate::config_loader::AppConfig;
use crate::core::registery::{self, Registery};
use crate::error::Result;
use crate::fs::paths::get_plugin_manager_file_path;
use tabled::Table;
use tracing::info;

pub async fn execute(config: &AppConfig) -> Result<()> {
    info!("Listing all installed plugins!");

    let registery: Registery = Registery::generate().await?;
    println!("{}", Table::new(registery.plugins.values()));
    Ok(())
}
