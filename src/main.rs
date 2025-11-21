mod config_loader;
mod logger;
mod utils;

#[tokio::main]
async fn main() {
    let config: config_loader::Config = config_loader::load_or_create();

    let _logger_guard = logger::init(&config.settings.logs_dir);
    tracing::info!("Starting VDPM!");
}
