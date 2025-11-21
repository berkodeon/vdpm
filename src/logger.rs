use chrono::Local;
use std::fs;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling;
use tracing_subscriber::{EnvFilter, fmt};

pub fn init(log_folder: &str) -> WorkerGuard {
    let log_dir = crate::utils::get_home_dir().join(log_folder);
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir).expect("Failed to create logs directory");
    }

    let timestamp = Local::now().format("%d-%m-%Y-%H:%M").to_string();
    let log_file_name = format!("vdpm_{}.log", timestamp);

    let file_appender = rolling::never(&log_dir, &log_file_name);
    let (non_blocking, guard) = NonBlocking::new(file_appender);

    let filter = EnvFilter::from_default_env().add_directive(LevelFilter::DEBUG.into());

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(non_blocking)
        .with_timer(fmt::time::LocalTime::rfc_3339())
        .with_ansi(false)
        .try_init()
        .ok();

    guard
}
