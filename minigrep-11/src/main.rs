use std::fs;
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;

fn main() {
    let mut pattern = String::new();
    let mut files: Vec<String> = Vec::new();

    loop {
        print_menu(&pattern, &files);

        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");

        match choice.trim() {
            "1" => {
                print!("Enter search pattern: ");
                io::stdout().flush().unwrap();
                let mut p = String::new();
                io::stdin().read_line(&mut p).unwrap();
                pattern = p.trim().to_string();
            }
            "2" => {
                print!("Enter file path: ");
                io::stdout().flush().unwrap();
                let mut f = String::new();
                io::stdin().read_line(&mut f).unwrap();
                let path = f.trim().to_string();
                if !path.is_empty() {
                    files.push(path);
                }
            }
            "3" => {
                files.clear();
                println!("\n>>> File list cleared.");
            }
            "4" => {
                if pattern.is_empty() || files.is_empty() {
                    println!("\n>>> Error: You need both a pattern and files to search!");
                } else {
                    run_parallel_search(&pattern, &files);
                }
            }
            "5" => {
                println!("Goodbye!");
                break;
            }
            _ => println!("\n>>> Invalid option. Please enter 1-5."),
        }
    }
}

/// Displays the interactive menu
fn print_menu(pattern: &str, files: &[String]) {
    println!("\n==============================");
    println!("      RUST MINI-GREP        ");
    println!("==============================");
    println!("1. Set Pattern   [Current: \"{}\"]", pattern);
    println!("2. Add File      [Files Loaded: {}]", files.len());
    if !files.is_empty() {
        println!("   L Files: {:?}", files);
    }
    println!("3. Clear Files");
    println!("4. RUN SEARCH");
    println!("5. Exit");
    println!("==============================");
    print!("Selection: ");
    io::stdout().flush().unwrap();
}

/// The core multi-threaded engine
fn run_parallel_search(pattern: &str, files: &[String]) {
    // tx = transmitter, rx = receiver
    let (tx, rx) = mpsc::channel();
    let mut thread_handles = vec![];

    println!("\n--- Starting Search ---");

    for path in files {
        let tx_clone = tx.clone();
        let pattern_owned = pattern.to_string();
        let path_owned = path.to_string();

        // Spawn a thread for every file
        let handle = thread::spawn(move || match fs::read_to_string(&path_owned) {
            Ok(content) => {
                for (index, line) in content.lines().enumerate() {
                    if line.contains(&pattern_owned) {
                        let msg =
                            format!("[MATCH] {}:{} -> {}", path_owned, index + 1, line.trim());
                        tx_clone.send(msg).unwrap();
                    }
                }
            }
            Err(_) => {
                tx_clone
                    .send(format!("[ERROR] Could not read file: {}", path_owned))
                    .unwrap();
            }
        });
        thread_handles.push(handle);
    }

    // Crucial: Drop the original transmitter so the loop below knows when to finish
    drop(tx);

    // Collect and print results as they come in
    let mut match_count = 0;
    for received in rx {
        if received.starts_with("[MATCH]") {
            match_count += 1;
        }
        println!("{}", received);
    }

    // Ensure all threads are finished before returning to menu
    for handle in thread_handles {
        handle.join().unwrap();
    }

    println!("--- Search Finished. Total Matches: {} ---\n", match_count);
}
