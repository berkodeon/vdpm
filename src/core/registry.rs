use crate::config_loader;
use crate::core::plugin::{Plugin, PluginName};
use crate::error::Result;
use crate::fs::operations::list_files_by_extension;
use crate::fs::paths::get_registry_file_path;
use crate::utils::get_home_dir;
use anyhow::Context;
use csv::WriterBuilder;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Registry {
    pub plugins: BTreeMap<PluginName, Plugin>,
}

impl Registry {
    pub async fn from_file(path: &Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path)
            .await
            .context("failed to read registry file")?;

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(content.as_bytes());

        let plugins: BTreeMap<PluginName, Plugin> = reader
            .deserialize::<Plugin>()
            .collect::<std::result::Result<Vec<Plugin>, _>>()
            .context("failed to parse registry file as csv")?
            .into_iter()
            .map(|plugin| (plugin.name.clone(), plugin))
            .collect();

        Ok(Registry { plugins })
    }

    fn create_plugins_file(&self, path: &Path) -> Result<std::fs::File> {
        std::fs::File::create(path).context("failed to create registry file")
    }

    fn write_plugins<W: std::io::Write>(&self, wtr: &mut W) -> Result<&Self> {
        let mut writer = WriterBuilder::new().from_writer(wtr);

        for plugin in self.plugins.values() {
            writer
                .serialize(plugin)
                .context("failed to write plugin row to registry")?;
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
        let content = self
            .plugins
            .values()
            .filter(|p| p.installed && p.enabled)
            .map(|p| format!("import plugins.{}", p.name))
            .collect::<Vec<_>>()
            .join("\n");
        tokio::fs::write(path, content)
            .await
            .context("failed to write .visidatarc file")?;

        Ok(self)
    }

    pub async fn persist(&self) -> Result<&Self> {
        let config = config_loader::load_or_create()?;
        let rc_file_path = get_home_dir().join(&config.settings.rc_file);
        let registry_path = get_registry_file_path()?;

        self.to_file(&registry_path).await?;
        self.to_visidatarc_file(&rc_file_path).await?;

        Ok(self)
    }

    fn generate_with(
        installed_plugins: HashSet<PluginName>,
        previously_enabled: &HashSet<PluginName>,
        persisted: Option<&Registry>,
    ) -> Self {
        let plugins = installed_plugins
            .into_iter()
            .map(|name| {
                let source = persisted
                    .and_then(|registry| registry.plugins.get(&name))
                    .and_then(|plugin| plugin.source.clone());
                let plugin = Plugin {
                    name: name.clone(),
                    enabled: previously_enabled.contains(&name),
                    installed: true,
                    source,
                };
                (name, plugin)
            })
            .collect();

        Registry { plugins }
    }

    pub async fn generate() -> Result<Self> {
        let installed_plugins = Self::get_installed_plugins()?;
        let persisted = Self::from_file(&get_registry_file_path()?).await.ok();

        let previously_enabled: HashSet<PluginName> = match &persisted {
            Some(registry) => registry
                .plugins
                .values()
                .filter(|plugin| plugin.enabled)
                .map(|plugin| plugin.name.clone())
                .collect(),
            None => Self::get_enabled_plugins_from_visidatarc()
                .await
                .unwrap_or_default(),
        };

        let registry = Self::generate_with(installed_plugins, &previously_enabled, persisted.as_ref());
        registry.persist().await?;

        Ok(registry)
    }

    fn get_installed_plugins() -> Result<HashSet<PluginName>> {
        let config = config_loader::load_or_create()?;
        let installed_plugins: HashSet<String> = list_files_by_extension(
            &get_home_dir().join(&config.settings.plugin_folder),
            "py".to_string(),
        );
        Ok(installed_plugins
            .into_iter()
            .filter_map(|name| match name.parse::<PluginName>() {
                Ok(name) => Some(name),
                Err(err) => {
                    warn!("skipping invalid plugin file name \"{name}\": {err}");
                    None
                }
            })
            .collect())
    }

    async fn get_enabled_plugins_from_visidatarc() -> Result<HashSet<PluginName>> {
        let config = config_loader::load_or_create()?;
        let visidata_rc_content =
            tokio::fs::read_to_string(&get_home_dir().join(&config.settings.rc_file))
                .await
                .context("failed to read .visidatarc file")?;

        let enabled_plugins: HashSet<PluginName> = visidata_rc_content
            .split("\n")
            .filter(|line| line.starts_with("import plugins."))
            .filter_map(|line| line.strip_prefix("import plugins."))
            .filter_map(|name| match name.parse::<PluginName>() {
                Ok(name) => Some(name),
                Err(err) => {
                    warn!("skipping invalid plugin name \"{name}\" in .visidatarc: {err}");
                    None
                }
            })
            .collect();

        Ok(enabled_plugins)
    }
}
