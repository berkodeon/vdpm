use serde_json;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum VDPMError {
    #[error("Configuration error: {0}: {1}")]
    ConfigError(String, toml::de::Error),

    #[error("Visidata RC error: {0}: {1}")]
    VisidataRCError(String, std::io::Error),

    #[error("Reading registery failed: {0}: {1}")]
    RegisteryReadError(String, RegistryError),
}

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, VDPMError>;
