use thiserror::Error;

#[derive(Error, Debug)]
pub enum VDPMError {
    #[error("Configuration error: {0}: {1}")]
    ConfigError(String, toml::de::Error),
}

pub type Result<T> = std::result::Result<T, VDPMError>;
