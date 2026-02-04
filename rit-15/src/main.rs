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

    CatFile {
        /// Pretty-print the contents
        #[arg(short = 'p')]
        pretty_print: bool,

        /// The object hash to read
        object_hash: String,
    },
    /// Create a tree object from the current index
    WriteTree,

    CommitTree {
        /// The tree object to commit
        tree_hash: String,

        /// The parent commit hash (optional)
        #[arg(short = 'p')]
        parent_hash: Option<String>,

        /// The commit message
        #[arg(short = 'm')]
        message: String,
    },

}

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Init => {
            commands::init()?;
        }
        Commands::HashObject { file, write } => {
            let hash = commands::hash_object(&file, write)?;
            if !write {
                println!("{hash}");
            }
        }
        Commands::CatFile { pretty_print, object_hash } => {
            commands::cat_file(&object_hash, pretty_print)?;
        }
        Commands::WriteTree => {
            let tree_hash = commands::write_tree(".")?;
            println!("{}", tree_hash);
        }
        Commands::CommitTree { tree_hash, parent_hash, message } => {
        let commit_hash = commands::commit_tree(&tree_hash, parent_hash.as_deref(), &message)?;
        println!("{}", commit_hash);
    }

    }
    Ok(())
}
