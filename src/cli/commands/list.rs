use crate::config_loader::AppConfig;
use crate::error::Result;
use tracing::info;

pub async fn execute(config: &AppConfig) -> Result<()> {
    info!("Listing all installed plugins!");
    Ok(())
}
