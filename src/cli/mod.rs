pub mod args;
pub mod commands;
use crate::config_loader::AppConfig;
use crate::error::Result;

use args::{Cli, Commands};

pub async fn run(cli: Cli, config: AppConfig) -> Result<()> {
    match cli.command {
        Commands::List => commands::list::execute(&config).await?,
        Commands::Interactive => unreachable!("Interactive mode is handled in main!"),
    }
    Ok(())
}
