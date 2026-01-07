use thiserror::Error;

use crate::interactive::registry_snapshot::RegistrySnapshot;
#[derive(Error, Debug)]
pub enum VDPMError {
    #[error("File watcher error")]
    FileWatcherError(#[from] notify::Error),

    #[error("Configuration error: {0}: {1}")]
    ConfigError(String, toml::de::Error),

    #[error("Visidata RC error: {0}: {1}")]
    VisidataRCError(String, std::io::Error),

    #[error("Plugin error: {0}: {1}")]
    PluginError(String, PluginOperationError),

    #[error("Visidata RC error: {0}: {1}")]
    RegistryFileChangeHandlerError(
        String,
        tokio::sync::mpsc::error::SendError<RegistrySnapshot>,
    ),

    #[error("Reading registry failed: {0}: {1}")]
    RegistryOperationError(String, RegistryError),
}

#[derive(Error, Debug)]
pub enum PluginOperationError {
    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),

    #[error("Plugin not exists error")]
    NotExistError(String),

    #[error("Fetch error: {0}")]
    FetchError(#[from] reqwest::Error),
}

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("CSV Error: {0}")]
    Csv(#[from] csv::Error),

    #[error("CSV Error: {0}")]
    CSVWriter(#[from] csv::IntoInnerError<csv::Writer<Vec<u8>>>),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, VDPMError>;
