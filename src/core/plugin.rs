use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize, Tabled, Hash, PartialEq)]
pub struct Plugin {
    pub name: String,
    pub enabled: bool,
    pub installed: bool,
    #[tabled(display = "display_source")]
    pub source: Option<String>,
}

fn display_source(source: &Option<String>) -> String {
    source.clone().unwrap_or_default()
}

impl Plugin {
    pub fn new(
        name: impl Into<String>,
        enabled: bool,
        installed: bool,
        source: Option<String>,
    ) -> crate::error::Result<Self> {
        let name = name.into();
        validate_plugin_name(&name)?;
        Ok(Self {
            name,
            enabled,
            installed,
            source,
        })
    }
}

fn validate_plugin_name(name: &str) -> crate::error::Result<()> {
    let valid = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !valid {
        anyhow::bail!(
            "invalid plugin name \"{name}\": must contain only letters, digits, underscores, and hyphens"
        );
    }
    Ok(())
}
