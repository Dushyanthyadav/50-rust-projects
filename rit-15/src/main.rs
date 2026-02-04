use clap::{Parser, Subcommand};
use anyhow::Result;

mod commands;

#[derive(Parser, Debug)]
#[command(name = "rit")]
#[command(about = "A small custom git clone written in rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a new rit repository
    Init,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Init => {
            commands::init()?;
        }
    }
    Ok(())
}



