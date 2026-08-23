use std::path::PathBuf;

use crate::config_loader;

use crate::core::plugin::Plugin;
use crate::error::Result;
use crate::utils::get_home_dir;

use anyhow::Context;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use tabled::Table;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, instrument, warn};

#[instrument(level = "info", skip_all, fields(plugin = %name))]
pub async fn execute(name: &str) -> Result<Table> {
    let config = config_loader::load_or_create()?;
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

    let plugin_content: String = download_plugin(name).await?;
    debug!(
        content_bytes = plugin_content.len(),
        "Plugin downloaded successfully"
    );

    let mut file: File = create_plugin_file(name, &plugin_file_path).await?;

    file.write_all(plugin_content.as_bytes())
        .await
        .context("failed to write plugin file to disk")?;

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

        tokio::fs::create_dir_all(parent_path)
            .await
            .with_context(|| format!("failed to create plugin directory for \"{name}\""))?;
    } else {
        warn!("Plugin path has no parent directory");
    }

    debug!("Creating plugin file");
    File::create(path)
        .await
        .context("failed to create plugin file")
}

#[instrument(level = "info", skip_all, fields(plugin = %name))]
async fn download_plugin(name: &str) -> Result<String> {
    let config = config_loader::load_or_create()?;
    let visidata_version = format!("v{}", config.settings.vd_version);
    let repo_url = format!(
        "{}/saulpw/visidata/{}/visidata/loaders",
        crate::utils::get_github_raw_base_url(),
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
        .with_context(|| format!("failed to reach GitHub while downloading plugin \"{name}\""))?;

    let response = response
        .error_for_status()
        .with_context(|| format!("failed to download plugin \"{name}\""))?;

    let text = response
        .text()
        .await
        .with_context(|| format!("failed to read downloaded content for plugin \"{name}\""))?;

    debug!(content_bytes = text.len(), "Plugin content retrieved");
    Ok(text)
}
