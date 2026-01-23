use rand::prelude::*;
use std::cmp::Ordering;
use std::io::{self, Write};
use std::process;
fn main() {
    println!("Welcome to number guessing game!!!");
    println!(
        "Rule:\n1. computer generates a random number between 0 and 100. both 0 and 100 included.\n2. The secret random number is not shown to you. you have guess it.\n3. You only get 6 tries to guess the number"
    );

    let mut rng = rand::rng();
    let secret_num = rng.random_range(0..101);

    for i in 0..7 {
        print!("Enter your {} guess: ", i + 1);
        io::stdout().flush().unwrap();
        let mut guess: String = String::new();

        io::stdin()
            .read_line(&mut guess)
            .expect("error while reading the line");
        let guess: i32 = guess.trim().parse().expect("Guess is not a number");

        match guess.cmp(&secret_num) {
            Ordering::Less => println!("Less than Secret number XXXX"),
            Ordering::Greater => println!("Greater than Secret number XXXX"),
            Ordering::Equal => {
                println!("You correctly guess the number.!!!! \nThank you for playing the game");
                process::exit(0);
            }
        }
    }

    println!("You have exhausted your tries better luck next time!!!!");
}
