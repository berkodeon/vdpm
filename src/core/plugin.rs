use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use tabled::Tabled;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PluginName(String);

impl FromStr for PluginName {
    type Err = anyhow::Error;

    fn from_str(name: &str) -> crate::error::Result<Self> {
        let valid = !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        if !valid {
            anyhow::bail!(
                "invalid plugin name \"{name}\": must contain only letters, digits, underscores, and hyphens"
            );
        }
        Ok(Self(name.to_string()))
    }
}

impl TryFrom<String> for PluginName {
    type Error = anyhow::Error;

    fn try_from(value: String) -> crate::error::Result<Self> {
        value.parse()
    }
}

impl From<PluginName> for String {
    fn from(value: PluginName) -> Self {
        value.0
    }
}

impl fmt::Display for PluginName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Tabled, Hash, PartialEq)]
pub struct Plugin {
    pub name: PluginName,
    pub enabled: bool,
    pub installed: bool,
    #[tabled(display = "display_source")]
    pub source: Option<String>,
}

fn display_source(source: &Option<String>) -> String {
    source.clone().unwrap_or_default()
}

impl Plugin {
    pub fn new(name: PluginName, enabled: bool, installed: bool, source: Option<String>) -> Self {
        Self {
            name,
            enabled,
            installed,
            source,
        }
    }
}
