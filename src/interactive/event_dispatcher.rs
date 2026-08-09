use tracing::debug;

use crate::{
    cli::{self, args::Commands},
    core::{plugin::Plugin, registry::Registry},
    error::Result,
    interactive::registry_snapshot::RegistrySnapshot,
};

#[derive(Debug)]
struct PluginOperation {
    command: Commands,
    plugin: Plugin,
}

pub async fn process_diff(
    previous: &RegistrySnapshot,
    current: &RegistrySnapshot,
) -> Result<()> {
    debug!(
        "old hash: {}, new hash: {}",
        &previous.hash, &current.hash
    );

    let operations: Vec<PluginOperation> =
        generate_operations(&previous.registry, &current.registry);

    dispatch_operation(operations).await
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
        if !old_registry.plugins.contains_key(plugin_name) {
            operations.push(PluginOperation {
                command: Commands::Install {
                    name: plugin_name.clone(),
                },
                plugin: new_plugin.clone(),
            });

            if new_plugin.enabled {
                operations.push(PluginOperation {
                    command: Commands::Enable {
                        name: plugin_name.clone(),
                    },
                    plugin: new_plugin.clone(),
                });
            }
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
