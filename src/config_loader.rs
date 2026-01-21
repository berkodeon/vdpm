use crate::error::Result;
use serde::Deserialize;
use std::fmt;
use std::sync::OnceLock;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub settings: Settings,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub vdpm_config_folder_path: String,
    pub plugin_manager_file: String,
    pub plugin_folder: String,
    pub rc_file: String,
    pub logs_dir: String,
    pub vd_version: String,
}

pub struct SettingsOverrides {
    pub vd_version: String,
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "plugin_dir: {}, plugin_file: {}, plugin_folder: {}, rc_file: {}, logs_dir: {}",
            self.vdpm_config_folder_path,
            self.plugin_manager_file,
            self.plugin_folder,
            self.rc_file,
            self.logs_dir,
        )
    }
}

impl fmt::Display for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.settings)
    }
}

pub fn load_or_create() -> Result<&'static AppConfig> {
    if CONFIG.get().is_none() {
        panic!("Attempt to access config before init!");
    }

    Ok(CONFIG.get().unwrap())
}

pub fn init(overrides: SettingsOverrides) {
    // TODO if the config file is not there, we should create a default one

    if CONFIG.get().is_some() {
        // Do nothing
        return;
    }

    CONFIG.get_or_init(|| {
        let config_str = include_str!("../config.toml");
        let mut app_config: AppConfig = toml::de::from_str(config_str).unwrap();

        app_config.settings.vd_version = overrides.vd_version;

        app_config
    });
}
