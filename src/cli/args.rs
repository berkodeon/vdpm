use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
// #[command(name="vdpm")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    List,
    Interactive,
}
