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
    fn test_should_generate_registry_when_no_plugins() {
        let registry = Registry::generate_with(HashSet::new(), HashSet::new()).unwrap();

        assert_eq!(registry.plugins.len(), 0);
    }

    #[test]
    fn test_should_generate_registry_with_installed_and_enabled_plugins() {
        let installed_plugins: HashSet<String> = HashSet::from(["foo".into(), "bar".into()]);
        let enabled_plugins: HashSet<String> = HashSet::from(["bar".into()]);
        let registry = Registry::generate_with(installed_plugins, enabled_plugins).unwrap();

        assert_eq!(registry.plugins.len(), 2);
        assert_eq!(
            registry.plugins.get("foo").unwrap(),
            &Plugin {
                name: "foo".into(),
                enabled: false,
                installed: true,
            },
        );
        assert_eq!(
            registry.plugins.get("bar").unwrap(),
            &Plugin {
                name: "bar".into(),
                enabled: true,
                installed: true,
            }
        );
    }

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

    #[test]
    fn test_should_write_csv_headers_when_no_plugins() {
        let registry = Registry::generate_with(HashSet::new(), HashSet::new()).unwrap();
        let mut buffer: Vec<u8> = vec![];

        registry.write_plugins(&mut buffer).unwrap();

        let csv_string = String::from_utf8(buffer).unwrap();
        let csv_content: Vec<&str> = csv_string.split("\n").collect();

        assert_eq!(csv_content.len(), 2);
        assert!(csv_content.contains(&"name,enabled,installed"));
        assert!(csv_content.contains(&""));
    }
}

#[cfg(test)]
mod registry_integration_tests {
    use crate::core::plugin::Plugin;
    use crate::{
        config_loader::{self, SettingsOverrides},
        core::registry::Registry,
        fs::paths::get_registry_file_path,
    };
    use std::fs::{self, File, OpenOptions};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::{TempDir, tempdir};

    struct MockFileDirs {
        _home_dir: TempDir,
        rc_file: PathBuf,
        plugins_csv_file: PathBuf,
        plugins_folder: PathBuf,
    }

    fn add_mock_plugin(dirs: &MockFileDirs, plugin: &Plugin) {
        let mut plugins_csv = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&dirs.plugins_csv_file.as_path())
            .unwrap();

        writeln!(
            plugins_csv,
            "{},{},{}",
            &plugin.name, plugin.enabled, plugin.installed
        )
        .unwrap();
    }

    fn save_mock_plugin(dirs: &MockFileDirs, plugin: &Plugin) {
        // write to plugins.csv
        add_mock_plugin(dirs, plugin);

        // create {plugin_name}.py file
        File::create(&dirs.plugins_folder.join(format!("{}.py", &plugin.name))).unwrap();

        // add entry to .visidatarc
        let mut rc_file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&dirs.rc_file.as_path())
            .unwrap();
        writeln!(rc_file, "import plugins.{}", &plugin.name).unwrap();
    }

    fn bootstrap() -> MockFileDirs {
        config_loader::init(SettingsOverrides {
            vd_version: "1.0.0".into(),
        });

        let tmp_home_dir = tempdir().unwrap();
        unsafe {
            // override HOME with our tmp folder, which will be deleted after run
            std::env::set_var("HOME", tmp_home_dir.as_ref());
        }

        let config = config_loader::load_or_create().unwrap();
        let plugins_folder_path = tmp_home_dir.as_ref().join(&config.settings.plugin_folder);
        let rc_file_path = tmp_home_dir.as_ref();
        let config_loader_path = tmp_home_dir
            .as_ref()
            .join(&config.settings.vdpm_config_folder_path);
        let tmp_file_plugins_csv = config_loader_path
            .as_path()
            .join(&config.settings.plugin_manager_file);
        let rc_file = rc_file_path.join(&config.settings.rc_file);

        std::fs::create_dir_all(&plugins_folder_path).unwrap();
        std::fs::create_dir_all(&rc_file_path).unwrap();
        std::fs::create_dir_all(&config_loader_path).unwrap();
        File::create(&rc_file).unwrap();
        let mut plugins_csv_file = File::create(&tmp_file_plugins_csv).unwrap();

        writeln!(plugins_csv_file, "name,enabled,installed").unwrap();

        let mock_dirs = MockFileDirs {
            _home_dir: tmp_home_dir,
            rc_file: rc_file.to_path_buf(),
            plugins_csv_file: tmp_file_plugins_csv.to_path_buf(),
            plugins_folder: plugins_folder_path,
        };

        save_mock_plugin(
            &mock_dirs,
            &Plugin {
                name: "foo".into(),
                enabled: false,
                installed: true,
            },
        );

        save_mock_plugin(
            &mock_dirs,
            &Plugin {
                name: "bar".into(),
                enabled: true,
                installed: true,
            },
        );

        mock_dirs
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_should_generate_registry_from_file() {
        let _tmp_dirs = bootstrap();

        let path = get_registry_file_path().unwrap();
        let registry = Registry::from_file(&path).await.unwrap();

        assert_eq!(registry.plugins.len(), 2);
        assert_eq!(
            registry.plugins.get("foo").unwrap(),
            &Plugin {
                name: "foo".into(),
                enabled: false,
                installed: true,
            },
        );
        assert_eq!(
            registry.plugins.get("bar").unwrap(),
            &Plugin {
                name: "bar".into(),
                enabled: true,
                installed: true,
            },
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_should_persist_added_plugin_into_registry_file() {
        let tmp_dirs = bootstrap();

        let path = get_registry_file_path().unwrap();
        let mut registry = Registry::from_file(&path).await.unwrap();
        let baz_plugin = Plugin {
            name: "baz".into(),
            enabled: true,
            installed: true,
        };

        registry.plugins.insert("baz".into(), baz_plugin.clone());
        registry.to_file(&tmp_dirs.plugins_csv_file).await.unwrap();

        let plugins_csv_content = fs::read_to_string(&tmp_dirs.plugins_csv_file).unwrap();

        assert_eq!(plugins_csv_content.contains("foo,false,true\n"), true);
        assert_eq!(plugins_csv_content.contains("bar,true,true\n"), true);
        assert_eq!(plugins_csv_content.contains("baz,true,true\n"), true);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_should_persist_updated_plugin_into_registry_file() {
        let tmp_dirs = bootstrap();

        let path = get_registry_file_path().unwrap();
        let mut registry = Registry::from_file(&path).await.unwrap();

        // enable foo
        registry
            .plugins
            .insert(
                "foo".into(),
                Plugin {
                    name: "foo".into(),
                    enabled: true,
                    installed: true,
                },
            )
            .unwrap();

        registry.to_file(&tmp_dirs.plugins_csv_file).await.unwrap();

        let plugins_csv_content = fs::read_to_string(&tmp_dirs.plugins_csv_file).unwrap();

        assert_eq!(plugins_csv_content.contains("foo,true,true\n"), true);
        assert_eq!(plugins_csv_content.contains("bar,true,true\n"), true);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_should_persist_deleted_plugin_into_registry_file() {
        let tmp_dirs = bootstrap();

        let path = get_registry_file_path().unwrap();
        let mut registry = Registry::from_file(&path).await.unwrap();

        // remove foo
        registry.plugins.remove("bar");

        registry.to_file(&tmp_dirs.plugins_csv_file).await.unwrap();

        let plugins_csv_content = fs::read_to_string(&tmp_dirs.plugins_csv_file).unwrap();

        assert_eq!(plugins_csv_content.contains("bar"), false);
        assert_eq!(plugins_csv_content.contains("bar,true,true\n"), false);
        assert_eq!(plugins_csv_content.contains("foo,false,true\n"), true);
    }
}
