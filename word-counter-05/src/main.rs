use clap::{CommandFactory, Parser};
use std::{fs::File, io::Read};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file_name: Option<String>,

    text: Option<String>,
}

fn main() {
    let args = Args::parse();
    let mut file_content = String::new();
    if let Some(file_name) = args.file_name {
        let mut file = File::open(file_name).unwrap();
        file.read_to_string(&mut file_content).unwrap();
    } else {
        match args.text {
            Some(text_content) => {
                file_content = text_content;
            }
            None => {
                let mut cmd = Args::command();
                cmd.print_help().unwrap();
                std::process::exit(0);
            }
        }
    }

    let count = file_content.split_whitespace().into_iter().count();
    println!("Word Count: {}", count);
}
