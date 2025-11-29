use crate::config_loader::{self, AppConfig};
use crate::core::plugin::{self, Plugin};
use crate::core::registery;
use crate::error::{RegistryError, Result, VDPMError};
use crate::fs::operations::list_files_by_extension;
use crate::utils::get_home_dir;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tokio;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registery {
    pub plugins: HashMap<String, Plugin>,
}

impl Registery {
    pub async fn from_file(path: &Path) -> Result<Self> {
        let registery_json = tokio::fs::read_to_string(path).await.map_err(|e| {
            VDPMError::RegisteryReadError(
                "Failed to read registery file".into(),
                RegistryError::from(e),
            )
        })?;

        let registery = serde_json::from_str(&registery_json).map_err(|e| {
            VDPMError::RegisteryReadError(
                "Failed to parse registery file to Registery object".into(),
                RegistryError::from(e),
            )
        })?;

        Ok(registery)
    }

    pub async fn generate() -> Result<Registery> {
        let installed_plugins: HashSet<String> = Registery::get_installed_plugins()?;
        let enabled_plugins: HashSet<String> = Registery::get_enabled_plugins().await?;

        let plugins: HashMap<String, Plugin> = installed_plugins
            .into_iter()
            .map(|plugin| {
                let is_enabled: bool = enabled_plugins.contains(plugin.as_str());
                (
                    plugin.clone(),
                    Plugin {
                        name: plugin,
                        installed: true,
                        enabled: is_enabled,
                    },
                )
            })
            .collect();

        Ok(Registery { plugins })
    }

    fn get_installed_plugins() -> Result<HashSet<String>> {
        // TODO @memedov, let's make it async also!
        let config: AppConfig = config_loader::load_or_create()?;
        let installed_plugins: HashSet<String> = list_files_by_extension(
            &get_home_dir().join(&config.settings.plugin_folder),
            "py".to_string(),
        );
        Ok(installed_plugins)
    }

    async fn get_enabled_plugins() -> Result<HashSet<String>> {
        let config: AppConfig = config_loader::load_or_create()?;
        let visidata_rc_content =
            tokio::fs::read_to_string(&get_home_dir().join(&config.settings.rc_file))
                .await
                .map_err(|e| {
                    VDPMError::VisidataRCError("VisidataRC could not be read!".into(), e)
                })?;

        let enabled_plugins: HashSet<String> = visidata_rc_content
            .split("\n")
            .filter(|line| line.starts_with("import plugins."))
            .filter_map(|line| {
                line.strip_prefix("import plugins.")
                    .map(|enabled_plugin| enabled_plugin.to_string())
            })
            .collect();

        Ok(enabled_plugins)
    }
}
