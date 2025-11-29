use std::path::Path;
use std::process::Child;

use clap::Parser;
use tracing::info;
mod cli;
mod config_loader;
mod core;
mod error;
mod fs;
mod interactive;
mod logger;
mod utils;

use crate::config_loader::AppConfig;
use crate::error::Result;
use crate::fs::operations::create_visidata_rc;
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

    create_visidata_rc(&Path::new(&config.settings.rc_file)).await?;

    let cli = Cli::parse();

    match cli.command {
        cli::args::Commands::Interactive => {
            info!("Starting interactive VPDP!");
            let mut interactive_process: Child = interactive::launch(config).await?;
            interactive_process
                .wait()
                .expect("VisiData process failed!");
        }
        _ => {
            cli::run(cli, config).await?;
        }
    };

    info!("VDPM completed successfully!");
    Ok(())
}
