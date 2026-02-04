use std::fs;
use std::path::Path;
use anyhow::Result;

pub const RIT_DIR: &str = ".rit";

pub fn init() -> Result<()> {
    fs::create_dir(RIT_DIR)?;

    // This stores blobs and trees
    let objects_path = format!("{}/objects", RIT_DIR);
    fs::create_dir(&objects_path)?;

    // we store branches
    let refs_path = format!("{}/refs", RIT_DIR);
    fs::create_dir(&refs_path)?;

    let heads_path = format!("{}/heads", refs_path);
    fs::create_dir(&heads_path)?;

    // HEAD file pointer
    let head_path = format!("{}/HEAD", RIT_DIR);
    fs::write(&head_path, "ref: refs/heads/main\n")?;

    println!("Initialized empty Rit  repository in {}", RIT_DIR);
    Ok(())
}