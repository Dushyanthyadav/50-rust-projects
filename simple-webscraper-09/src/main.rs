use clap::Parser;
use scraper::{Html, Selector};

#[derive(Parser, Debug)]
#[command(author, version, about = "Async CLI web scraper in Rust")]
struct Args {
    /// The URL to scrape
    #[arg(short, long)]
    url: String,

    /// The CSS selector to search for
    #[arg(short, long)]
    selector: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Fetching {} (non-blocking)...", args.url);

    let response = reqwest::get(&args.url).await.unwrap();

    if !response.status().is_success() {
        eprint!("Error: Failed to fetch URL. Status: {}", response.status());
        // return Ok(());
    }

    let body = response.text().await.unwrap();

    let document = Html::parse_document(&body);

    let selector = Selector::parse(&args.selector)
        .map_err(|e| format!("Invalid CSS selector: {:?}", e))
        .unwrap();

    println!("--- Results for '{}' ---\n", args.selector);

    let mut count = 0;

    for element in document.select(&selector) {
        let text = element.text().collect::<Vec<_>>().join("");

        if !text.trim().is_empty() {
            println!("Match {}: {}", count + 1, text.trim());
            count += 1;
        }
    }
    if count == 0 {
        println!("No elements found.");
    }
}
