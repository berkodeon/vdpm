pub mod args;
pub mod commands;
use crate::error::Result;

use args::Commands;
use tabled::Table;

pub async fn run(command: &Commands) -> Result<Table> {
    // TODO @memedov shall we add trait to these commands?
    let result: Table = match command {
        Commands::List => commands::list::execute().await?,
        Commands::Enable { name } => commands::enable::execute(&name).await?,
        Commands::Disable { name } => commands::disable::execute(&name).await?,
        Commands::Install { name } => commands::install::execute(&name).await?,
        Commands::Uninstall { name } => commands::uninstall::execute(&name).await?,
        Commands::Interactive => unreachable!("Interactive mode is handled in main!"),
    };
    Ok(result)
}
