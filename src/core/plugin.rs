use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize, Tabled, Hash, Default)]
pub struct Plugin {
    pub name: String,
    pub enabled: bool,
    pub installed: bool,
}
