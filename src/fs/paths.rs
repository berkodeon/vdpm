use crate::config_loader::{self, AppConfig};
use crate::error::Result;
use crate::utils::get_home_dir;
use std::{fs, path::PathBuf};

fn create_vdpm_config_directory(config_path: &str) -> PathBuf {
    let home = get_home_dir();
    let absolute_config_path = home.join(config_path);

    if !absolute_config_path.exists() {
        fs::create_dir_all(&absolute_config_path).expect("Failed to create plugin directory");
    }

    absolute_config_path
}

pub fn get_plugin_manager_file_path() -> Result<PathBuf> {
    let config: AppConfig = config_loader::load_or_create()?;

    let vdpm_config_directory =
        create_vdpm_config_directory(&config.settings.vdpm_config_folder_path);
    let vdpm_config_file_path = vdpm_config_directory.join(config.settings.plugin_manager_file);
    Ok(vdpm_config_file_path)
}
