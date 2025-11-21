use thiserror::Error;

#[derive(Error, Debug)]
pub enum VDPMError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, VDPMError>;
