use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use directories::BaseDirs;
use serde::Deserialize;
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

