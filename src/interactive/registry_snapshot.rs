use crate::core::registry::Registry;
use std::fmt;

#[derive(Debug)]
pub struct RegistrySnapshot {
    pub registry: Registry,
    pub hash: u64,
    // TODO @memedov lets add created_at to process idempotent
}

impl fmt::Display for RegistrySnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RegistrySnapshot(hash: {}, registry: {:?})",
            self.hash, self.registry
        )
    }
}
