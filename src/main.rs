use clap::Parser;
use tracing::info;
mod cli;
mod config_loader;
mod error;
mod logger;
mod utils;

use crate::config_loader::AppConfig;
use crate::error::Result;
use cli::args::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let config: AppConfig = config_loader::load_or_create()?;

    let _logger_guard = logger::init(&config.settings.logs_dir);
    tracing::info!("Starting VDPM!");
    tracing::debug!(
        "Config(from: {}) is loaded: {}",
        &config.settings.logs_dir,
        config
    );

    let cli = Cli::parse();

    match cli.command {
        cli::args::Commands::Interactive => {
            info!("Starting interactive VPDP!");
        }
        _ => {
            cli::run(cli, config).await?;
        }
    };

    info!("VDPM completed successfully!");
    Ok(())
}
