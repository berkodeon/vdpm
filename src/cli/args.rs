use clap::{Parser, Subcommand};
use std::fmt::{self, Display, Formatter};

#[derive(Parser, Debug)]
// #[command(name="vdpm")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    List,
    Enable { name: String },
    Disable { name: String },
    Install { name: String },
    Uninstall { name: String },
    Interactive,
}

impl Display for Commands {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Commands::List => write!(f, "list"),
            Commands::Enable { name } => write!(f, "enable {}", name),
            Commands::Disable { name } => write!(f, "disable {}", name),
            Commands::Install { name } => write!(f, "install {}", name),
            Commands::Uninstall { name } => write!(f, "uninstall {}", name),
            Commands::Interactive => write!(f, "interactive"),
        }
    }
}
