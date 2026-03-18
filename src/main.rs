use std::process::Child;
use std::sync::Arc;

use clap::Parser;
use notify::RecommendedWatcher;
use tabled::Table;
use tokio::sync::Mutex;
use tracing::info;
mod cli;
mod config_loader;
mod core;
mod error;
mod fs;
mod interactive;
mod logger;
mod utils;

use crate::config_loader::SettingsOverrides;
use crate::error::Result;
use crate::fs::operations::create_visidata_rc;
use crate::interactive::WatcherState;
use crate::utils::{get_home_dir, get_vd_version};
use cli::args::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let vd_version = get_vd_version()?;

    config_loader::init(SettingsOverrides {
        vd_version: vd_version.clone(),
    });
    let config = config_loader::load_or_create()?;

    let _logger_guard = logger::init(&config.settings.logs_dir);

    tracing::debug!("Loaded config: {:?}", config);
    tracing::info!("Starting VDPM!");
    tracing::debug!(
        "Config(from: {}) is loaded: {}",
        &config.settings.logs_dir,
        config
    );

    create_visidata_rc(&get_home_dir().join(&config.settings.rc_file)).await?;

    let cli = Cli::parse();

    match cli.command {
        cli::args::Commands::Interactive => {
            info!("Starting interactive VDPM!");
            let (mut interactive_process, _watcher): (Child, RecommendedWatcher) =
                interactive::launch().await?;
            interactive_process
                .wait()
                .expect("VisiData process failed!");
        }
        command => {
            let result: Table = cli::run(&command).await?;
            println!("{}", result);
        }
    };

    info!("VDPM completed successfully!");
    Ok(())
}
