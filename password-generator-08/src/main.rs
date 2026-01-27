use clap::Parser;
use rand::rng;
use rand::seq::IndexedRandom;

const UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Length of the password
    #[arg(short = 'n', long, default_value_t = 12)]
    length: usize,

    /// Include Uppercase letters (A-Z)
    #[arg(short, long)]
    uppercase: bool,

    /// Include lowercase letters (a-z)
    #[arg(short, long)]
    lowercase: bool,

    /// Include numbers (0-9)
    #[arg(short = 'N', long)]
    numbers: bool,

    /// Include symbols (!@#$...)
    #[arg(short, long)]
    symbols: bool,
}

fn main() {
    let args = Cli::parse();

    let mut charset = String::new();

    let explicit_selection = args.uppercase || args.lowercase || args.numbers || args.symbols;

    if explicit_selection {
        if args.uppercase {
            charset.push_str(UPPER);
        }
        if args.lowercase {
            charset.push_str(LOWER);
        }
        if args.numbers {
            charset.push_str(NUMBERS);
        }
        if args.symbols {
            charset.push_str(SYMBOLS);
        }
    } else {
        charset.push_str(UPPER);
        charset.push_str(LOWER);
        charset.push_str(NUMBERS);
        charset.push_str(SYMBOLS);
    }

    if charset.is_empty() {
        eprintln!("Error You must include at least one character set!");
        std::process::exit(1);
    }

    let mut password = String::new();
    let mut rng = rng();

    let chars: Vec<char> = charset.chars().collect();

    for _ in 0..args.length {
        let random_char = chars.choose(&mut rng).expect("Charset is empty");
        password.push(*random_char);
    }

    println!("{}", password);
}
