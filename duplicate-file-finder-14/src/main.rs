use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use jwalk::WalkDir;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::{fs, io};

const PARTIAL_SIZE: u64 = 4096;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    ///Directory to search     
    search_dir: PathBuf,

    ///Depth (default = 1)
    #[arg(short, long, default_value_t = 1, conflicts_with = "recursive")]
    depth: usize,

    ///Recursive (default = 1)
    #[arg(short, long, default_value_t = false)]
    recursive: bool,
}
fn main() {
    let args = Args::parse();
    let directory = args.search_dir;

    let scanner_spinner = ProgressBar::new_spinner();
    scanner_spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    scanner_spinner.set_message("Traversing directory tree...");
    let mut size_map: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    let mut file_count = 0;

    // This is where we collect all the file paths to analyze
    if !args.recursive {
        for entry in WalkDir::new(directory).max_depth(args.depth) {
            let entry = entry.unwrap();
            let path = entry.path();
            let file_size = entry.metadata().unwrap().len();
            size_map.entry(file_size).or_default().push(path);
            file_count += 1;
            if file_count % 100 == 0 {
                scanner_spinner.set_message(format!("Building {} files...", file_count));
            }
            scanner_spinner.tick();
        }
    } else {
        for entry in WalkDir::new(directory) {
            let entry = entry.unwrap();
            let path = entry.path();
            let file_size = entry.metadata().unwrap().len();
            size_map.entry(file_size).or_default().push(path);
            file_count += 1;
            if file_count % 100 == 0 {
                scanner_spinner.set_message(format!("Building {} files...", file_count));
            }
            scanner_spinner.tick();
        }
    }
    scanner_spinner.finish_with_message("Finished building files");

    let potential_duplicates: Vec<Vec<PathBuf>> =
        size_map.into_values().filter(|v| v.len() > 1).collect();

    if potential_duplicates.is_empty() {
        println!("No files with matching sizes found. ");
        return;
    }

    println!("\n--- verifying hash ----");

    let total_groups = potential_duplicates.len() as u64;

    let pb = ProgressBar::new(total_groups);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    pb.set_message("Analyzing groups...");

    let duplicates: Vec<Vec<PathBuf>> = potential_duplicates
        .par_iter()
        .flat_map(|group| {
            let size = fs::metadata(&group[0]).map(|m| m.len()).unwrap_or(0);

            let confirmed_dupes = if size <= PARTIAL_SIZE {
                find_dupes_in_full_hash(group)
            } else {
                let partially_filterted = find_dupes_by_partial_hash(group);

                partially_filterted
                    .into_iter()
                    .flat_map(|subgroup| find_dupes_in_full_hash(&subgroup))
                    .collect()
            };

            pb.inc(1);
            confirmed_dupes
        })
        .collect();

    pb.finish_with_message("Duplicate search complete!");

    println!("\nFound {} sets of duplicates.", duplicates.len());

    for (i, group) in duplicates.iter().take(5).enumerate() {
        println!("Set #{}: {:?}", i + 1, group);
    }
}

#[allow(unused)]
fn find_dupes_by_partial_hash(files: &[PathBuf]) -> Vec<Vec<PathBuf>> {
    let mut hashes: HashMap<blake3::Hash, Vec<PathBuf>> = HashMap::new();

    for path in files {
        if let Ok(hash) = compute_partial_hash(path) {
            hashes.entry(hash).or_default().push(path.clone());
        }
    }

    hashes.into_values().filter(|v| v.len() > 1).collect()
}

fn find_dupes_in_full_hash(files: &[PathBuf]) -> Vec<Vec<PathBuf>> {
    let mut hashes: HashMap<blake3::Hash, Vec<PathBuf>> = HashMap::new();

    for path in files {
        if let Ok(hash) = compute_full_hash(path) {
            hashes.entry(hash).or_default().push(path.clone());
        }
    }

    hashes.into_values().filter(|v| v.len() > 1).collect()
}

fn compute_partial_hash(path: &PathBuf) -> io::Result<blake3::Hash> {
    let mut file = File::open(path)?;
    let mut buffer = [0u8; PARTIAL_SIZE as usize];

    let bytes_read = file.read(&mut buffer)?;

    Ok(blake3::hash(&buffer[..bytes_read]))
}

fn compute_full_hash(path: &PathBuf) -> io::Result<blake3::Hash> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(hasher.finalize())
}
