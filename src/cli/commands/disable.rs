use crate::core::plugin::Plugin;
use crate::error::Result;
use tabled::Table;
use tracing::info;

pub async fn execute(name: &str) -> Result<Table> {
    info!("Disable plugin({})!", name);
    // TODO delete the corresponding line from .visidatarc
    Ok(Table::new(Vec::<Plugin>::new()))
}
