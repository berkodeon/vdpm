use crate::error::Result;
use serde::Deserialize;
use std::fmt::{self, Display};
use std::fs::{self};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
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

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "plugin_dir: {}, plugin_file: {}, plugin_folder: {}, rc_file: {}, logs_dir: {}",
            self.plugin_dir, self.plugin_file, self.plugin_folder, self.rc_file, self.logs_dir,
        )
    }
}

impl fmt::Display for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.settings)
    }
}

pub fn load_or_create() -> Result<AppConfig> {
    // TODO if the config file is not there, we should create a default one
    let config_path = Path::new("config.toml");
    let config_str = fs::read_to_string(config_path).expect("Failed to read config file");
    Ok(toml::de::from_str(&config_str).expect("Failed to parse config file"))
}
