use crate::config_loader::AppConfig;
use crate::core::registry::{self, Registry};
use crate::error::Result;
use crate::fs::paths::get_registry_file_path;
use tabled::Table;
use tracing::info;

pub async fn execute(config: &AppConfig) -> Result<()> {
    info!("Listing all installed plugins!");

    let registry: Registry = Registry::generate().await?;
    println!("{}", Table::new(registry.plugins.values()));
    Ok(())
}
