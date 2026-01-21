use crate::config_loader::{self, AppConfig};
use crate::core::plugin::Plugin;
use crate::error::{RegistryError, Result, VDPMError};
use crate::fs::operations::list_files_by_extension;
use crate::utils::get_home_dir;
use csv::WriterBuilder;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use tokio;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

// TODO: move into macros or utils
macro_rules! assert_has_fields {
    ($ty:ty, $( $field:ident ),+ $(,)?) => {
        const _: fn(&$ty) = |v| {
            $(
                let _ = &v.$field;
            )+
        };
    };
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Registry {
    pub plugins: BTreeMap<String, Plugin>,
}

impl Registry {
    fn get_csv_headers(&self) -> Vec<&'static str> {
        // dyanmic alternative: https://crates.io/crates/struct-field-names-as-array
        assert_has_fields!(Plugin, name, enabled, installed);

        vec!["name", "enabled", "installed"]
    }

    pub async fn from_file(path: &Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            VDPMError::RegistryOperationError(
                "Failed to read registry file".into(),
                RegistryError::from(e),
            )
        })?;

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(content.as_bytes());

        let plugins = reader
            .deserialize::<Plugin>()
            .collect::<std::result::Result<Vec<Plugin>, _>>()
            .map_err(|e| {
                VDPMError::RegistryOperationError(
                    "Failed to parse CSV registry file".into(),
                    RegistryError::from(e),
                )
            })?
            .into_iter()
            .map(|plugin| (plugin.name.clone(), plugin))
            .collect();

        Ok(Registry { plugins })
    }

    pub async fn to_file(&self, path: &Path) -> Result<&Self> {
        let headers = self.get_csv_headers();

        let mut file = File::create(path).await.map_err(|e| {
            VDPMError::RegistryOperationError(
                "Failed to create CSV file".into(),
                RegistryError::from(e),
            )
        })?;

        let mut writer = WriterBuilder::new().has_headers(false).from_writer(vec![]);

        // write headers
        writer.write_record(headers).map_err(|e| {
            VDPMError::RegistryOperationError(
                "Failed to write CSV headers".into(),
                RegistryError::from(e),
            )
        })?;

        for plugin in self.plugins.values() {
            writer.serialize(plugin).map_err(|e| {
                VDPMError::RegistryOperationError(
                    "Failed to serialize plugin to CSV".into(),
                    RegistryError::from(e),
                )
            })?;
        }

        let data = writer.into_inner().map_err(|e| {
            VDPMError::RegistryOperationError(
                "Failed to finalize CSV writer".into(),
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

    pub async fn to_visidatarc_file(&self, path: &PathBuf) -> Result<&Self> {
        let map_err = |e| {
            VDPMError::RegistryOperationError(
                "Failed to write to .visidatarc file".into(),
                RegistryError::from(e),
            )
        };

        let content = self
            .plugins
            .values()
            .filter(|p| p.installed && p.enabled)
            .map(|p| format!("import plugins.{}", p.name))
            .collect::<Vec<_>>()
            .join("\n");
        tokio::fs::write(path, content).await.map_err(map_err)?;

        Ok(self)
    }

    pub async fn generate() -> Result<Self> {
        let installed_plugins: HashSet<String> = Registry::get_installed_plugins()?;
        let enabled_plugins: HashSet<String> = Registry::get_enabled_plugins().await?;

        let plugins: BTreeMap<String, Plugin> = installed_plugins
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
        let config = config_loader::load_or_create()?;
        let installed_plugins: HashSet<String> = list_files_by_extension(
            &get_home_dir().join(&config.settings.plugin_folder),
            "py".to_string(),
        );
        Ok(installed_plugins)
    }

    async fn get_enabled_plugins() -> Result<HashSet<String>> {
        let config = config_loader::load_or_create()?;
        let visidata_rc_content =
            tokio::fs::read_to_string(&get_home_dir().join(&config.settings.rc_file))
                .await
                .map_err(|e| {
                    VDPMError::VisidataRCError("VisidataRC could not be read!".into(), e)
                })?;

        tracing::debug!("visidata_rc_content {:?}", &visidata_rc_content);

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
