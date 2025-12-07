use crate::config_loader::AppConfig;
use crate::core::plugin::Plugin;
use crate::core::registry::{self, Registry};
use crate::error::Result;
use crate::fs::paths::get_registry_file_path;
use tabled::Table;
use tracing::info;

pub async fn execute(name: &str) -> Result<Table> {
    info!("Install plugin({})!", name);
    // TODO copy the install logic from github
    Ok(Table::new(Vec::<Plugin>::new()))
}
