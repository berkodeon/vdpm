use directories::BaseDirs;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use polars::prelude::*;
// use polars::prelude::SeriesUtf8;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Cursor;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use toml;

#[derive(Debug, Deserialize)]
struct Config {
    settings: Settings,
}

#[derive(Debug, Deserialize)]
struct Settings {
    plugin_dir: String,
    plugin_file: String,
    plugin_folder: String,
}

#[derive(Debug)]
enum PluginOperationType {
    Install,
    Uninstall,
    Enable,
    Disable,
}

#[derive(Debug)]
struct LineDiff {
    old_entry: String,
    new_entry: String,
}

#[derive(Debug)]
struct PluginOperation {
    operation: PluginOperationType,
    plugin_name: String
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

fn get_changes_structured(changes: &Vec<(&Series, &Series)>) -> Vec<LineDiff> {
    let mut diffs = Vec::new();

    for &(old_series, new_series) in changes {
        let len = old_series.len();
        assert_eq!(len, new_series.len(), "Series length mismatch");

        let old_utf8 = old_series.utf8().expect("Expected Utf8 Series");
        let new_utf8 = new_series.utf8().expect("Expected Utf8 Series");

        for i in 0..len {
            let old_val = old_utf8.get(i).unwrap_or("").to_string();
            let new_val = new_utf8.get(i).unwrap_or("").to_string();

            diffs.push(LineDiff {
                old_entry: old_val,
                new_entry: new_val,
            });
        }
    }

    diffs
}

fn resolve_uninstalled_packages(vec: &Vec<LineDiff>) ->  {

}

fn diff_lines(old_csv: &str, new_csv: &str, key_column: &str) -> Vec<PluginOperation> {
    let old_df = CsvReader::new(Cursor::new(new_csv))
        .with_options(CsvReadOptions::default().with_has_header(true))
        .finish()?;

    let new_df = CsvReader::new(Cursor::new(old_csv))
        .with_options(CsvReadOptions::default().with_has_header(true))
        .finish()?;

    let added = new_df.clone().as_single_chunk().join(
        &old_df,
        [key_column],
        [key_column],
        JoinArgs::new(JoinType::Anti),None
    )?;
    println!("Added rows:\n{}", added);

    let removed = old_df.clone().as_single_chunk().join(
        &new_df,
        [key_column],
        [key_column],
        JoinArgs::new(JoinType::Anti),None
    )?;
    println!("Removed rows:\n{}", removed);

    let common_old = old_df.clone().as_single_chunk().join(
        &new_df,
        [key_column],
        [key_column],
        JoinArgs::new(JoinType::Inner),None
    )?;

    let common_new = new_df.clone().as_single_chunk().join(
        &old_df,
        [key_column],
        [key_column],
        JoinArgs::new(JoinType::Inner),None
    )?;

    let changed = common_old
        .iter()
        .zip(common_new.iter())
        .filter(|(a, b)| a != b)
        .collect::<Vec<_>>();

    if !added.is_empty() {
        let operations: Vec<PluginOperation> = added
            .get_rows()
            .iter()
            .map(|row| {
                PluginOperation {
                    operation: PluginOperationType::Install,
                    plugin_name: row["name"],
                }
            })
            .collect();



        let line_diffs: Vec<LineDiff> = get_changes_structured(&added);
        let packages_to_uninstall = resolve_uninstalled_packages(&line_diffs);
    }

    if !changed.is_empty() {
                let operations: Vec<PluginOperation> = changed
            .get_rows()
            .iter()
            .map(|row| {
                PluginOperation {
                    operation: PluginOperationType::Install,
                    plugin_name: row["name"],
                }
            })
            .collect();
    }

    vec![
        PluginOperation{
            operation: PluginOperationType::Install,
            plugin_name: String::from("odeonsmightyplugin")
        }
    ]
}

fn write_to_file(file_path: &str, content: &Vec<String>) -> io::Result<()> {
    let mut file = File::create(file_path)?;
    writeln!(file, "plugin_name")?;

    for line in content {
        writeln!(file, "{}", line)?;
    }
    Ok(())
}

fn main() {
    let config = read_config();

    let plugin_dir = create_plugin_directory(&config.settings.plugin_dir);

    let plugin_path = get_home_dir().join(&config.settings.plugin_folder);
    let python_files = list_python_files(&plugin_path);

    let plugins_file = plugin_dir.join(&config.settings.plugin_file);
    let plugins_file_path = plugins_file.to_str().unwrap().to_string();

    if let Err(e) = write_to_file(&plugins_file_path, &python_files) {
        eprintln!("Error writing to file: {}", e);
        return;
    }

    let previous_content = fs::read_to_string(&plugins_file_path).unwrap_or_default();
    let mut previous_hash = calculate_sha256_from_str(&previous_content);

    let mut watcher = RecommendedWatcher::new(
        {
            let plugins_file_path = plugins_file_path.clone();
            move |res: Result<Event, notify::Error>| match res {
                Ok(event) => {
                    println!("File event {:#?}", event.kind);

                    match fs::read_to_string(&plugins_file_path) {
                        Ok(current_content) => {
                            let current_hash = calculate_sha256_from_str(&current_content);
                            if current_hash != previous_hash {
                                diff_lines(&previous_content, &current_content, "name");

                                previous_hash = current_hash;
                            } else {
                                println!("No actual content change (same hash).");
                            }
                        }
                        Err(err) => {
                            eprintln!("Failed to read file during event: {:?}", err);
                        }
                    }
                }
                Err(e) => {
                    println!("Watch error: {:?}", e);
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

    println!("Visidata started with plugin list!");
    child.wait().expect("VisiData process failed");
    println!("Stopped VisiData");
}
