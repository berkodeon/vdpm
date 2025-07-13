use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Opts {
    #[arg(long = "debug-tty")]
    pub debug_tty: Option<String>,
}

pub fn read_cli_args() -> Opts {
  Opts::parse()
}
