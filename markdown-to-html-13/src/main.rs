use std::{fs, path::PathBuf};
use pulldown_cmark::{Parser, html::push_html};
use clap::{self};


#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Args {
    ///The input mark-down file
    markdown_file: PathBuf,

    ///The output file
    #[arg(short, long, default_value ="output.html")]
    output_file: PathBuf,
}

fn main() {
    let args = <Args as clap::Parser>::parse();

    let md_content = fs::read_to_string(args.markdown_file.as_path()).unwrap();

    let parser = Parser::new(md_content.as_str());

    let mut html_content = String::new();

    push_html(&mut html_content, parser);
    
    fs::write(args.output_file.as_path(), html_content).unwrap();
}
