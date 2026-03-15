use crate::config_loader::{self, AppConfig};
use crate::core::plugin::Plugin;
use crate::error::{RegistryError, Result, VDPMError};
use crate::fs::operations::list_files_by_extension;
use crate::utils::get_home_dir;
use csv::WriterBuilder;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

mod sealed {
    pub trait Writable: std::io::Write {}

    impl Writable for std::fs::File {}
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Registry {
    pub plugins: BTreeMap<String, Plugin>,
}

impl Registry {
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

        let plugins: BTreeMap<String, Plugin> = reader
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

    fn create_plugins_file(&self, path: &Path) -> Result<std::fs::File> {
        std::fs::File::create(path).map_err(|e| {
            VDPMError::RegistryOperationError(
                "Failed to create CSV file".into(),
                RegistryError::from(e),
            )
        })
    }

    fn write_plugins<W: sealed::Writable>(&self, wtr: &mut W) -> Result<&Self> {
        let mut writer = WriterBuilder::new().has_headers(false).from_writer(wtr);

        writer
            .write_record(vec!["name", "enabled", "installed"])
            .map_err(|e| {
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

        let _ = writer.flush();

        Ok(self)
    }

    pub async fn to_file(&self, path: &Path) -> Result<&Self> {
        let mut file: std::fs::File = self.create_plugins_file(path)?;

        self.write_plugins(&mut file)?;

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

    fn generate_with(
        installed_plugins: HashSet<String>,
        enabled_plugins: HashSet<String>,
    ) -> Result<Self> {
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

    pub async fn generate() -> Result<Self> {
        let installed_plugins: HashSet<String> = Registry::get_installed_plugins()?;
        let enabled_plugins: HashSet<String> = Registry::get_enabled_plugins().await?;

        Ok(Self::generate_with(installed_plugins, enabled_plugins)?)
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

#[cfg(test)]
mod registry_unit_tests {
    use super::*;

    impl sealed::Writable for Vec<u8> {}

    #[test]
    fn test_should_write_plugins() {
        let installed_plugins: HashSet<String> = HashSet::from(["foo".into(), "bar".into()]);
        let enabled_plugins: HashSet<String> = HashSet::from(["bar".into()]);
        let registry = Registry::generate_with(installed_plugins, enabled_plugins).unwrap();

        let mut buffer: Vec<u8> = vec![];
        registry.write_plugins(&mut buffer).unwrap();

        let csv_string = String::from_utf8(buffer).unwrap();
        let csv_content: Vec<&str> = csv_string.split("\n").collect();

        assert_eq!(csv_content.len(), 4);
        assert!(csv_content.contains(&"name,enabled,installed"));
        assert!(csv_content.contains(&"foo,false,true"));
        assert!(csv_content.contains(&"bar,true,true"));
        assert!(csv_content.contains(&""));
    }
}
