use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct Plugin {
    pub name: String,
    pub enabled: bool,
    pub installed: bool,
}
