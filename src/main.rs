use directories::BaseDirs;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use polars::prelude::*;
// use polars::prelude::SeriesUtf8;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::Cursor;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use toml;
use std::os::unix::fs::OpenOptionsExt; // for custom_flags on Unix
use libc; // brings libc::* constants into scope
use env_logger::{Builder, Target};
use log::LevelFilter;
use vdpm::cli_args::{ read_cli_args };

#[derive(Debug, Deserialize)]
struct Config {
    settings: Settings,
}

#[derive(Debug, Deserialize)]
struct Settings {
    plugin_dir: String,
    plugin_file: String,
    plugin_folder: String,
    debug_tty_prefix: String,
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
    log::debug!("Added rows:\n{}", added);

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
    log::debug!("Removed rows:\n{}", removed);

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

fn process_operations(operations: Vec<PluginOperation>)-> Result<(), ()> {
    // TODO fix the result type, learn Result and error types
    for operation in operations.iter() {
        match operation.operation {
            PluginOperationType::Install => {
                log::debug!("Installing plugin: {}", operation.plugin_name);
                // Dummy install logic
            }
            PluginOperationType::Uninstall => {
                log::debug!("Uninstalling plugin: {}", operation.plugin_name);
                // Dummy uninstall logic
            }
        }
    }
    Ok(())
}

fn init_logger(tty_path: &str) -> io::Result<()> {
    let target = match tty_path.is_empty() {
        true => Target::Stdout,
        false => {
            let tty = OpenOptions::new()
                .write(true)
                .custom_flags(libc::O_NOCTTY)
                .open(tty_path)?;
            Target::Pipe(Box::new(tty))
        }
    };

    Builder::from_default_env()
      .filter_level(LevelFilter::Debug) // this can be overriden by RUST_LOG env var
      .target(target)
      .format_timestamp_secs()
      .try_init()
      .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

fn main() {
    let config = read_config();
    let cli_args = read_cli_args();

    if let Some(debug_tty) = cli_args.debug_tty {
        let tty_path = format!(
            "{}{}",
            config.settings.debug_tty_prefix,
            debug_tty,
        );

        init_logger(&tty_path).unwrap_or_else(|e| {
            panic!("Failed to initialize logger to tty: {} {}", e, tty_path);
        });
    } else {
        init_logger("").unwrap_or_else(|e| {
            panic!("Failed to initialize logger to stdout: {}", e);
        });
    }
x
    let plugin_dir = create_plugin_directory(&config.settings.plugin_dir);

    let plugin_path = get_home_dir().join(&config.settings.plugin_folder);
    let python_files = list_python_files(&plugin_path);

    let plugins_file = plugin_dir.join(&config.settings.plugin_file);
    let plugins_file_path = plugins_file.to_str().unwrap().to_string();

    if let Err(e) = write_to_file(&plugins_file_path, &python_files) {
        log::error!("Error writing to file: {}", e);
        return;
    }

    let previous_content = fs::read_to_string(&plugins_file_path).unwrap_or_default();
    let mut previous_hash = calculate_sha256_from_str(&previous_content);

    let mut watcher = RecommendedWatcher::new(
        {
            let plugins_file_path = plugins_file_path.clone();
            move |res: Result<Event, notify::Error>| match res {
                Ok(event) => {
                    log::debug!("File event {:#?}", event);

                    match fs::read_to_string(&plugins_file_path) {
                        Ok(current_content) => {
                            let current_hash = calculate_sha256_from_str(&current_content);
                            if current_hash != previous_hash {
                                let all_operations: Vec<PluginOperation> =
                                    diff_lines(&previous_content, &current_content, "plugin_name");
                                let _ = process_operations(all_operations);

                                previous_hash = current_hash;
                            } else {
                                log::debug!("No actual content change (same hash).");
                            }
                        }
                        Err(err) => {
                            log::error!("Failed to read file during event: {:?}", err);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Watch error: {:?}", e);
                }
            }
        },
        notify::Config::default().with_poll_interval(Duration::from_secs(1)),
    )
    .expect("Failed to create watcher");

    watcher
        .watch(Path::new(&plugins_file_path), RecursiveMode::NonRecursive)
        .expect("WATCH FAILED");

    let mut child = Command::new("vd")
        .arg(&plugins_file_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to start VisiData");

    log::debug!("Visidata started with plugin list!");
    child.wait().expect("VisiData process failed");
    log::debug!("Stopped VisiData");
}
