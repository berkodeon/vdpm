use directories::BaseDirs;
use notify::event::{DataChange, ModifyKind::Data};
use notify::{
    Event, EventKind::Modify, FsEventWatcher, RecommendedWatcher, RecursiveMode, Watcher,
};
use polars::prelude::*;
// use polars::prelude::SeriesUtf8;
use chrono::Local;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::{self, File, read_to_string};
use std::io::Cursor;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::sync::{Mutex, mpsc};
use toml;
use tracing::level_filters::LevelFilter;
use tracing::{debug, error};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling;
use tracing_subscriber::{EnvFilter, fmt};
use vdpm::cli_args::read_cli_args;

#[derive(Debug, Deserialize)]
struct Config {
    settings: Settings,
}

#[derive(Debug, Deserialize)]
struct Settings {
    plugin_dir: String,
    plugin_file: String,
    plugin_folder: String,
    logs_dir: String,
}

#[derive(Debug, Deserialize)]
enum PluginOperationType {
    Install,
    Uninstall,
}

#[derive(Debug)]
struct LineDiff {
    old_entry: String,
    new_entry: String,
}

#[derive(Debug)]
struct PluginOperation {
    operation: PluginOperationType,
    plugin_name: String,
}

#[derive(Debug)]
struct FileState {
    path: String,
    content: String,
    hash: String,
}

fn read_config() -> Config {
    let config_path = Path::new("config.toml");
    let config_str = fs::read_to_string(config_path).expect("Failed to read config file");
    toml::de::from_str(&config_str).expect("Failed to parse config file")
}

fn get_home_dir() -> PathBuf {
    if let Some(base_dirs) = BaseDirs::new() {
        base_dirs.home_dir().to_path_buf()
    } else {
        panic!("Unable to find home directory")
    }
}

fn create_plugin_directory(plugin_dir_str: &str) -> PathBuf {
    let home = get_home_dir();
    let plugin_dir = home.join(plugin_dir_str);

    if !plugin_dir.exists() {
        fs::create_dir_all(&plugin_dir).expect("Failed to create plugin directory");
    }

    plugin_dir
}

fn list_python_files(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap().filter_map(Result::ok) {
            let path = entry.path();
            if path.extension() == Some("py".as_ref()) {
                if let Some(file_name) = path.file_stem() {
                    if let Some(file_str) = file_name.to_str() {
                        files.push(file_str.to_string());
                    }
                }
            }
        }
    }
    files
}

fn calculate_sha256_from_str(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn diff_lines(old_csv: &str, new_csv: &str, key_column: &str) -> Vec<PluginOperation> {
    let mut all_operations: Vec<PluginOperation> = vec![];

    let old_df = CsvReader::new(Cursor::new(old_csv))
        .with_options(CsvReadOptions::default().with_has_header(true))
        .finish()
        .unwrap();

    let new_df = CsvReader::new(Cursor::new(new_csv))
        .with_options(CsvReadOptions::default().with_has_header(true))
        .finish()
        .unwrap();

    let added = new_df
        .clone()
        .as_single_chunk()
        .join(
            &old_df,
            [key_column],
            [key_column],
            JoinArgs::new(JoinType::Anti),
            None,
        )
        .unwrap();
    debug!("Added rows:\n{}", added);

    let removed = old_df
        .clone()
        .as_single_chunk()
        .join(
            &new_df,
            [key_column],
            [key_column],
            JoinArgs::new(JoinType::Anti),
            None,
        )
        .unwrap();
    debug!("Removed rows:\n{}", removed);

    let changed = new_df
        .clone()
        .as_single_chunk()
        .join(
            &old_df,
            [key_column],
            [key_column],
            JoinArgs::new(JoinType::Inner).with_suffix(Some("_old".into())),
            None,
        )
        .unwrap();

    if !added.is_empty() {
        let name_series = added.column("plugin_name").unwrap();
        let install_operations: Vec<PluginOperation> = name_series
            .str()
            .unwrap()
            .into_iter()
            .map(|name| PluginOperation {
                operation: PluginOperationType::Install,
                plugin_name: name.unwrap().to_string(),
            })
            .collect();
        all_operations.extend(install_operations)
    }

    if !removed.is_empty() {
        let name_series = removed.column("plugin_name").unwrap();
        let uninstall_operations: Vec<PluginOperation> = name_series
            .str()
            .unwrap()
            .into_iter()
            .map(|name| PluginOperation {
                operation: PluginOperationType::Uninstall,
                plugin_name: name.unwrap().to_string(),
            })
            .collect();

        all_operations.extend(uninstall_operations)
    }

    all_operations
}

fn write_to_file(file_path: &str, content: &Vec<String>) -> io::Result<()> {
    let mut file = File::create(file_path)?;
    writeln!(file, "plugin_name")?;

    for line in content {
        writeln!(file, "{}", line)?;
    }
    Ok(())
}

fn process_operations(operations: Vec<PluginOperation>) -> Result<(), ()> {
    // TODO fix the result type, learn Result and error types
    for operation in operations.iter() {
        match operation.operation {
            PluginOperationType::Install => {
                debug!("Installing plugin: {}", operation.plugin_name);
                // Dummy install logic
            }
            PluginOperationType::Uninstall => {
                debug!("Uninstalling plugin: {}", operation.plugin_name);
                // Dummy uninstall logic
            }
        }
    }
    Ok(())
}

fn init_logger(log_folder: &str) -> WorkerGuard {
    let log_dir = get_home_dir().join(log_folder);
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir).expect("Failed to create logs directory");
    }

    let timestamp = Local::now().format("%d-%m-%Y-%H:%M").to_string();
    let log_file_name = format!("vdpm_{}.log", timestamp);

    // init log file as async-safe
    let file_appender = rolling::never(&log_dir, &log_file_name);
    let (non_blocking, guard) = NonBlocking::new(file_appender);

    let filter = EnvFilter::from_default_env().add_directive(LevelFilter::DEBUG.into()); // if not RUST_LOG set, fallback to DEBUG

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(non_blocking)
        .with_timer(fmt::time::LocalTime::rfc_3339())
        .with_ansi(false)
        .try_init()
        .ok(); // ignore error if already initialized

    // return guard to keep non-blocking writer alive
    guard
}

fn watch_for_file_content_changes(
    plugins_file_path: &String,
    tx: mpsc::Sender<bool>,
) -> FsEventWatcher {
    let mut watcher = RecommendedWatcher::new(
        {
            move |res: Result<Event, notify::Error>| match res {
                Ok(event) => {
                    debug!("File event {:#?}", event);

                    if let Modify(Data(data_change_type)) = event.kind {
                        if DataChange::Content == data_change_type {
                            debug!("CONTENT CHANGED!!!");

                            if tx.blocking_send(true).is_err() {
                                error!("Failed to send content event to tx, {:?}", event);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Watch error: {:?}", e);
                }
            }
        },
        notify::Config::default()
            .with_poll_interval(Duration::from_secs(1))
            .with_compare_contents(true),
    )
    .expect("Failed to create watcher");

    watcher
        .watch(Path::new(&plugins_file_path), RecursiveMode::NonRecursive)
        .expect("WATCH FAILED");

    watcher
}

#[tokio::main]
async fn main() {
    let config = read_config();
    // let cli_args = read_cli_args();

    // let _ = LOG_GUARD.get_or_init(|| init_logger(&config.settings.logs_dir));
    let _log_file_guard = init_logger(&config.settings.logs_dir);

    let plugin_dir = create_plugin_directory(&config.settings.plugin_dir);

    let plugin_path = get_home_dir().join(&config.settings.plugin_folder);
    let python_files = list_python_files(&plugin_path);

    let plugins_file = plugin_dir.join(&config.settings.plugin_file);
    let plugins_file_path = plugins_file.to_str().unwrap().to_string();

    if let Err(e) = write_to_file(&plugins_file_path, &python_files) {
        error!("Error writing to file: {}", e);
        return;
    }

    let previous_content = fs::read_to_string(&plugins_file_path).unwrap_or_default();
    let file_state = Arc::new(Mutex::new(FileState {
        path: plugins_file_path.clone(),
        hash: calculate_sha256_from_str(&previous_content),
        content: previous_content,
    }));

    let (tx, mut rx) = mpsc::channel::<bool>(16);

    let _watcher = watch_for_file_content_changes(&plugins_file_path, tx.clone());

    let file_state_clone = file_state.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            debug!("Got a content change message: {}", &msg);

            if msg == true {
                let mut file_state = file_state_clone.lock().await;

                let new_content = read_to_string(&file_state.path).unwrap();
                let new_hash = calculate_sha256_from_str(&new_content);

                debug!("old hash: {}, new hash: {}", &file_state.hash, &new_hash);

                if new_hash != file_state.hash {
                    let all_operations: Vec<PluginOperation> =
                        diff_lines(&file_state.content, &new_content, "plugin_name");
                    let _ = process_operations(all_operations);

                    file_state.hash = new_hash;
                    file_state.content = new_content;
                }
            }
        }
    });

    let mut child = Command::new("vd")
        .arg(&plugins_file_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to start VisiData");

    debug!("Visidata started with plugin list!");
    child.wait().expect("VisiData process failed");
    debug!("Stopped VisiData");
}

// fn run_vd(plugins_file_path: &str) {
//     let child = Command::new("vd")
//         .arg(&plugins_file_path)
//         .stdin(Stdio::inherit())
//         .stdout(Stdio::inherit())
//         .stderr(Stdio::inherit())
//         .spawn()
//         .expect("failed to start VisiData");
// }
