use crate::core::registry::Registry;
use crate::error::Result;
use tabled::Table;
use tracing::info;

pub async fn execute() -> Result<Table> {
    info!("Listing all installed plugins!");

    let registry: Registry = Registry::generate().await?;
    Ok(Table::new(registry.plugins.values()))
}
