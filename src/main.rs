use std::process::Child;

use clap::Parser;
use notify::RecommendedWatcher;
use tabled::Table;
use tracing::info;

use vdpm::cli::{self, args::Cli};
use vdpm::config_loader::{self, RuntimeSettings};
use vdpm::error::Result;
use vdpm::fs::operations::create_visidata_rc;
use vdpm::interactive;
use vdpm::logger;
use vdpm::utils::{get_home_dir, get_vd_version};

#[tokio::main]
async fn main() -> Result<()> {
    let vd_version = get_vd_version()?;

    config_loader::init(RuntimeSettings { vd_version });
    let config = config_loader::load_or_create()?;

    let _logger_guard = logger::init(&config.settings.logs_dir);
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
