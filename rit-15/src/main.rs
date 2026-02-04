use anyhow::Result;
use clap::{Parser, Subcommand};

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

    /// Compute object ID and optionlly create blob a blob from a file
    HashObject {
        /// The file to hash
        file: String,

        /// Actually write the object into the database
        #[arg(short = 'w', long)]
        write: bool,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Init => {
            commands::init()?;
        }
        Commands::HashObject { file, write } => {
            commands::hash_object(&file, write)?;
        }
    }
    Ok(())
}
