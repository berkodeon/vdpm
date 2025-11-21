use serde::Deserialize;
use std::fs::{self};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub settings: Settings,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub plugin_dir: String,
    pub plugin_file: String,
    pub plugin_folder: String,
    pub rc_file: String,
    pub logs_dir: String,
}

pub fn load_or_create() -> Config {
    // TODO if the config file is not there, we should create a default one
    let config_path = Path::new("config.toml");
    let config_str = fs::read_to_string(config_path).expect("Failed to read config file");
    toml::de::from_str(&config_str).expect("Failed to parse config file")
}
