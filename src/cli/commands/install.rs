use std::path::PathBuf;

use crate::config_loader;

use crate::core::plugin::Plugin;
use crate::core::registry::Registry;
use crate::error::Result;
use crate::github::GithubClient;
use crate::utils::get_home_dir;

use anyhow::Context;
use tabled::Table;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, instrument, warn};

#[instrument(level = "info", skip_all, fields(plugin = %name))]
pub async fn execute(name: &str, source: Option<&str>) -> Result<Table> {
    let mut plugin = Plugin::new(name, false, false, None)?;

    let config = config_loader::load_or_create()?;
    debug!(
        plugin_folder = %config.settings.plugin_folder,
        "Loaded application configuration"
    );

    let plugin_folder = get_home_dir().join(&config.settings.plugin_folder);
    let plugin_file_path = plugin_folder.join(format!("{}.py", plugin.name));

    debug!(
        plugin_file_path = %plugin_file_path.display(),
        "Resolved plugin file path"
    );

    plugin.source = Some(match source {
        Some(source) => rewrite_github_blob_url(source),
        None => default_plugin_url(&plugin.name)?,
    });
    let resolved_source = plugin.source.clone().expect("source was just set above");

    let plugin_content: String = if resolved_source.starts_with("http://")
        || resolved_source.starts_with("https://")
    {
        fetch_from_url(&plugin.name, &resolved_source).await?
    } else {
        fetch_from_local_path(&plugin.name, &resolved_source).await?
    };
    debug!(
        content_bytes = plugin_content.len(),
        "Plugin downloaded successfully"
    );

    let mut file: File = create_plugin_file(&plugin.name, &plugin_file_path).await?;

    file.write_all(plugin_content.as_bytes())
        .await
        .context("failed to write plugin file to disk")?;

    let mut registry = Registry::generate().await?;
    if let Some(registered) = registry.plugins.get_mut(&plugin.name) {
        registered.source = plugin.source.clone();
    }
    registry.persist().await?;

    info!("Plugin installed successfully");

    let plugin = registry
        .plugins
        .get(&plugin.name)
        .cloned()
        .context("installed plugin missing from registry")?;
    Ok(Table::new(vec![plugin]))
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

fn default_plugin_url(name: &str) -> Result<String> {
    let config = config_loader::load_or_create()?;
    let visidata_version = format!("v{}", config.settings.vd_version);
    Ok(format!(
        "{}/saulpw/visidata/{}/visidata/loaders/{}.py",
        crate::utils::get_github_raw_base_url(),
        visidata_version,
        name
    ))
}

#[instrument(level = "info", skip_all, fields(plugin = %name))]
async fn fetch_from_url(name: &str, url: &str) -> Result<String> {
    let url = rewrite_github_blob_url(url);

    info!(url = %url, "Downloading plugin");

    let text = GithubClient::new()
        .download(&url)
        .await
        .with_context(|| format!("failed to download plugin \"{name}\""))?;

    debug!(content_bytes = text.len(), "Plugin content retrieved");
    Ok(text)
}

async fn fetch_from_local_path(name: &str, path: &str) -> Result<String> {
    tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("failed to read local plugin file \"{path}\" for plugin \"{name}\""))
}

pub fn rewrite_github_blob_url(url: &str) -> String {
    for prefix in [
        "https://github.com/",
        "http://github.com/",
        "https://www.github.com/",
        "http://www.github.com/",
    ] {
        if let Some(rest) = url.strip_prefix(prefix)
            && let Some((repo_part, file_part)) = rest.split_once("/blob/")
        {
            return format!("https://raw.githubusercontent.com/{repo_part}/{file_part}");
        }
    }
    url.to_string()
}
