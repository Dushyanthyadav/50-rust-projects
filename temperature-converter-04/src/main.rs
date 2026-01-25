use std::io;
use std::io::Write;

fn main() {
    let from: u8;
    let to: u8;
    let mut buffer = String::new();

    println!("1. Celsius\n2. Kelvin\n3. Fahrenheit\n");
    print!("From: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut buffer).unwrap();
    from = buffer.trim().parse().unwrap();
    buffer.clear();

    print!("Enter the Temperature: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut buffer).unwrap();
    let temp: f64 = buffer.trim().parse().unwrap();
    buffer.clear();

    println!("1. Celsius\n2. Kelvin\n3. Fahrenheit\n");
    print!("To: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut buffer).unwrap();
    to = buffer.trim().parse().unwrap();

    match (from, to) {
        (1, 1) => println!("{:.2} Celsisus", temp),
        (2, 2) => println!("{:.2} Kelvin", temp),
        (3, 3) => println!("{:.2} Fahrenheit", temp),
        (1, 2) => println!("{:.2} Kelvin", temp + 273.15),
        (1, 3) => println!("{:.2} Fahrenheit", (temp * 1.8) + 32.0),
        (2, 1) => println!("{:.2} Celsius", temp - 273.15),
        (2, 3) => println!("{:.2} Fahrenheit", (temp - 273.15) * 1.8 + 32.0),
        (3, 1) => println!("{:.2} Celsius", (temp - 32.0) / 1.8),
        (3, 2) => println!("{:.2} Kelvin", (temp - 32.0) / 1.8 + 273.15),
        (_, _) => {}
    }
}
