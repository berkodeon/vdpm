use crate::config_loader::{self, AppConfig};
use crate::core::plugin::{self, Plugin};
use crate::core::registry;
use crate::error::{RegistryError, Result, VDPMError};
use crate::fs::operations::list_files_by_extension;
use crate::utils::get_home_dir;
use csv::WriterBuilder;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tokio;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub plugins: HashMap<String, Plugin>,
}

impl Registry {
    pub async fn from_file(path: &Path) -> Result<Self> {
        let registry_json = tokio::fs::read_to_string(path).await.map_err(|e| {
            VDPMError::RegistryOperationError(
                "Failed to read registry file".into(),
                RegistryError::from(e),
            )
        })?;

        let registry = serde_json::from_str(&registry_json).map_err(|e| {
            VDPMError::RegistryOperationError(
                "Failed to parse registry file to Registry object".into(),
                RegistryError::from(e),
            )
        })?;

        Ok(registry)
    }

    pub async fn to_file(&self, path: &Path) -> Result<&Self> {
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(vec![]);
        for plugin in self.plugins.values() {
            wtr.serialize(plugin).map_err(|e| {
                VDPMError::RegistryOperationError(
                    "Failed to serialize plugin to CSV".into(),
                    RegistryError::from(e),
                )
            })?;
        }

        let data = wtr.into_inner().map_err(|e| {
            VDPMError::RegistryOperationError(
                "Failed to finalize CSV writer".into(),
                RegistryError::from(e),
            )
        })?;

        let mut file = File::create(path).await.map_err(|e| {
            VDPMError::RegistryOperationError(
                "Failed to create CSV file".into(),
                RegistryError::from(e),
            )
        })?;
        file.write_all(&data).await.map_err(|e| {
            VDPMError::RegistryOperationError(
                "Failed to write CSV file".into(),
                RegistryError::from(e),
            )
        })?;

        Ok(self)
    }

    pub async fn generate() -> Result<Self> {
        let installed_plugins: HashSet<String> = Registry::get_installed_plugins()?;
        let enabled_plugins: HashSet<String> = Registry::get_enabled_plugins().await?;

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

        Ok(Registry { plugins })
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
