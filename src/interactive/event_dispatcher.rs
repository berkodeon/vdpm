use std::path::Path;

use tokio::sync::mpsc;
use tracing::debug;

use crate::{
    cli::{self, args::Commands},
    core::{plugin::Plugin, registry::Registry},
    error::Result,
    interactive::registry_snapshot::RegistrySnapshot,
    utils::hash,
};

#[derive(Debug)]
struct PluginOperation {
    command: Commands,
    plugin: Plugin,
}

pub fn listen(
    mut rx: mpsc::Receiver<RegistrySnapshot>,
    registry_file_path: &Path,
    mut last_processed_registry_snapshot: RegistrySnapshot,
) {
    tokio::spawn(listen_registry_change(
        rx,
        registry_file_path,
        last_processed_registry_snapshot,
    ));
}

async fn listen_registry_change(
    mut rx: mpsc::Receiver<RegistrySnapshot>,
    registry_file_path: &Path,
    mut last_processed_registry_snapshot: RegistrySnapshot,
) -> Result<()> {
    while let Some(registry_snapshot) = rx.recv().await {
        debug!("Got a content change message: {}", &registry_snapshot);
        // TODO @memedov, if registry snapshot created_at < last message processed, we should simply skip the message
        let new_registry: Registry = Registry::from_file(registry_file_path).await?;
        let new_hash: u64 = hash(&new_registry);
        let new_registry_snapshot = RegistrySnapshot {
            registry: new_registry.clone(),
            hash: new_hash,
        };

        debug!(
            "old hash: {}, new hash: {}",
            &last_processed_registry_snapshot.hash, &new_hash
        );

        if new_hash != last_processed_registry_snapshot.hash {
            let all_operations: Vec<PluginOperation> =
                generate_operations(&last_processed_registry_snapshot.registry, &new_registry);

            dispatch_operation(all_operations).await?;
            last_processed_registry_snapshot = RegistrySnapshot {
                registry: new_registry,
                hash: new_hash,
            };
        }
    }
    Ok(())
}

fn generate_operations(old_registry: &Registry, new_registry: &Registry) -> Vec<PluginOperation> {
    let mut operations = Vec::new();

    for (plugin_name, old_plugin) in &old_registry.plugins {
        let op = if !new_registry.plugins.contains_key(plugin_name) {
            Some(PluginOperation {
                command: Commands::Uninstall {
                    name: plugin_name.clone(),
                },
                plugin: old_plugin.clone(),
            })
        } else {
            let new_plugin = &new_registry.plugins[plugin_name];

            if new_plugin.enabled && !old_plugin.enabled {
                Some(PluginOperation {
                    command: Commands::Enable {
                        name: plugin_name.clone(),
                    },
                    plugin: old_plugin.clone(),
                })
            } else if !new_plugin.enabled && old_plugin.enabled {
                Some(PluginOperation {
                    command: Commands::Disable {
                        name: plugin_name.clone(),
                    },
                    plugin: old_plugin.clone(),
                })
            } else {
                None
            }
        };

        if let Some(op) = op {
            operations.push(op);
        }
    }

    for (plugin_name, new_plugin) in &new_registry.plugins {
        let op = if !old_registry.plugins.contains_key(plugin_name) {
            Some(PluginOperation {
                command: Commands::Install {
                    name: plugin_name.clone(),
                },
                plugin: new_plugin.clone(),
            })
        } else {
            None
        };

        if let Some(op) = op {
            operations.push(op);
        }
    }

    operations
}

async fn dispatch_operation(plugin_operations: Vec<PluginOperation>) -> Result<()> {
    for operation in plugin_operations {
        let operation_result = cli::run(&operation.command).await?;
        debug!(
            "Operation({}) is succesfully finished with result({})",
            &operation.command, operation_result
        )
    }

    Ok(())
}
