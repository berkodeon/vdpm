use crate::config_loader::AppConfig;
use crate::error::Result;
use tracing::info;

pub async fn execute(config: &AppConfig) -> Result<()> {
    info!("Listing all installed plugins!");
    // read the plugins folder
    // read the visidatarc
    // create a list of classes
    // return
    Ok(())
}
