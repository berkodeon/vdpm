use std::path::PathBuf;

use crate::config_loader;
use crate::config_loader::AppConfig;
use crate::core::plugin::Plugin;
use crate::error::{PluginOperationError, Result, VDPMError};
use crate::utils::get_home_dir;

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use tabled::Table;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, instrument, warn};

#[instrument(level = "info", skip_all, fields(plugin = %name))]
pub async fn execute(name: &str) -> Result<Table> {
    let config: AppConfig = config_loader::load_or_create()?;
    debug!(
        plugin_folder = %config.settings.plugin_folder,
        "Loaded application configuration"
    );

    let plugin_folder = get_home_dir().join(&config.settings.plugin_folder);
    let plugin_file_path = plugin_folder.join(format!("{}.py", name));

    debug!(
        plugin_file_path = %plugin_file_path.display(),
        "Resolved plugin file path"
    );

    let mut file: File = create_plugin_file(name, &plugin_file_path).await?;

    let plugin_content: String = download_plugin(name).await?;
    debug!(
        content_bytes = plugin_content.len(),
        "Plugin downloaded successfully"
    );

    file.write_all(plugin_content.as_bytes())
        .await
        .map_err(|e| {
            VDPMError::PluginError(
                "Failed to write plugin content to disk".into(),
                PluginOperationError::from(e),
            )
        })?;

    info!("Plugin installed successfully");

    Ok(Table::new(vec![Plugin {
        name: name.to_string(),
        enabled: false,
        installed: true,
    }]))
}

#[instrument(level = "debug", skip_all, fields(plugin = %name, path = %path.display()))]
async fn create_plugin_file(name: &str, path: &PathBuf) -> Result<File> {
    if let Some(parent_path) = path.parent() {
        debug!(
            parent_path = %parent_path.display(),
            "Ensuring plugin directory exists"
        );

        tokio::fs::create_dir_all(parent_path).await.map_err(|e| {
            VDPMError::PluginError(
                format!("Failed to create plugin directory for plugin({})", name).into(),
                PluginOperationError::from(e),
            )
        })?;
    } else {
        warn!("Plugin path has no parent directory");
    }

    debug!("Creating plugin file");
    File::create(path).await.map_err(|e| {
        VDPMError::PluginError(
            "Failed to create plugin file".into(),
            PluginOperationError::from(e),
        )
    })
}

#[instrument(level = "info", skip_all, fields(plugin = %name))]
async fn download_plugin(name: &str) -> Result<String> {
    let visidata_version = "v3.1.1";
    let repo_url = format!(
        "https://raw.githubusercontent.com/saulpw/visidata/{}/visidata/loaders",
        visidata_version
    );
    let plugin_download_url = format!("{}/{}.py", repo_url, name);

    info!(
        url = %plugin_download_url,
        visidata_version,
        "Downloading plugin"
    );

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("vdpm-client"));

    let client = reqwest::Client::new();
    let response = client
        .get(&plugin_download_url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| {
            VDPMError::PluginError(
                format!("HTTP request failed for plugin({})", name),
                PluginOperationError::from(e),
            )
        })?;

    let response = response.error_for_status().map_err(|e| {
        VDPMError::PluginError(
            format!("Non-success HTTP status while downloading plugin({})", name),
            PluginOperationError::from(e),
        )
    })?;

    let text = response.text().await.map_err(|e| {
        VDPMError::PluginError(
            format!("Failed to read response body for plugin({})", name),
            PluginOperationError::from(e),
        )
    })?;

    debug!(content_bytes = text.len(), "Plugin content retrieved");
    Ok(text)
}
