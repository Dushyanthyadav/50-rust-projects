use anyhow::Result;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::BufRead;
use std::io::BufReader;

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

pub fn hash_object(file_path: &str, write: bool) -> Result<String> {
    let content = fs::read(file_path)?;

    let header = format!("blob {}\0", content.len());

    // combine header and content
    let mut store = header.into_bytes();
    store.extend(&content);

    // Calculate SHA-1 Hash
    let mut hasher = Sha1::new();
    hasher.update(&store);
    let result = hasher.finalize();
    let hash_string = hex::encode(result);

    if write {
        let dir_name = &hash_string[..2];
        let file_name = &hash_string[2..];

        let object_dir = format!("{}/objects/{}", RIT_DIR, dir_name);
        let object_path = format!("{}/{}", object_dir, file_name);

        if !Path::new(&object_dir).exists() {
            fs::create_dir(&object_dir)?;
        }

        // compress data using zlib and write to file
        let file = fs::File::create(&object_path)?;
        let mut encoder = ZlibEncoder::new(file, Compression::default());
        encoder.write_all(&store)?;
        encoder.finish()?;
    }

    //println!("{}", hash_string);
    Ok(hash_string)
}

pub fn cat_file(object_hash: &str, _pretty_print: bool) -> Result<()> {

    // We assume the user gives a full 40-char hash not the file name of the file
    let dir_name = &object_hash[..2];
    let file_name = &object_hash[2..];
    let object_path = format!("{}/objects/{}/{}", RIT_DIR, dir_name, file_name);

    if !Path::new(&object_path).exists() {
        anyhow::bail!("Object {} not found", object_hash);
    }

    let file = fs::File::open(&object_path)?;

    // decompress the data
    let mut decoder = ZlibDecoder::new(file);
    let mut contents = Vec::new();
    decoder.read_to_end(&mut contents)?;

    
    // split header from body
    // format <type> <size>\0<content>
    let null_index = contents.iter()
        .position(|&b| b == 0) 
        .ok_or_else(|| anyhow::anyhow!("Invalid object format"))?;

    let header = String::from_utf8(contents[..null_index].to_vec())?;

    let body = &contents[null_index + 1..];
    let mut stdout = std::io::stdout();
    stdout.write_all(body)?;
    Ok(())
}

struct TreeEntry {
    name: String,
    mode: String,
    hash: String,
}

pub fn write_tree(path: &str) -> Result<String> {
   let index_path = format!("{}/index", RIT_DIR);
    let mut index_map = HashMap::new();
    
    if Path::new(&index_path).exists() {
        let file = fs::File::open(&index_path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if let Some((p, h)) = line.split_once(' ') {
                index_map.insert(p.to_string(), h.to_string());
            }
        }
    }

    write_tree_recursive(path, &index_map)
}

// The internal recursive function
fn write_tree_recursive(dir_path: &str, index: &HashMap<String, String>) -> Result<String> {
    let mut entries = Vec::new();
    
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().into_string().unwrap();
        
        if name == RIT_DIR || name == ".git" || name == "target" { continue; }

        // Clean path string (remove ./ for matching)
        let path_str = path.to_str().unwrap();
        let relative_path = path_str.trim_start_matches("./");

        if path.is_dir() {
            // For directories, we recurse
            // Optimization: In real git, we would check if the directory contains *any* staged files
            // For now, we just recurse and see what returns
            if let Ok(dir_hash) = write_tree_recursive(path_str, index) {
                // Only add the directory if it wasn't empty
                entries.push(TreeEntry {
                    name,
                    mode: "40000".to_string(),
                    hash: dir_hash,
                });
            }
        } else {
            // Check if this file is in our Index
            if let Some(hash) = index.get(relative_path) {
                // Use the hash from the index! (We don't need to re-hash the file)
                entries.push(TreeEntry {
                    name,
                    mode: "100644".to_string(),
                    hash: hash.clone(),
                });
            } else {
                // File is on disk but not in index -> IGNORE IT (Untracked)
                // println!("Skipping untracked file: {}", name);
            }
        }
    }
    
    // If the tree is empty (no staged files in this folder), we might want to return an error or handle it
    if entries.is_empty() {
        // For simplicity, let's just return a "empty tree" hash
    }

    // ... (The rest of sorting, packing, and writing is exactly the same as before) ...
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    
    let mut body = Vec::new();
    for entry in entries {
        let line = format!("{} {}\0", entry.mode, entry.name);
        body.extend(line.as_bytes());
        let hash_bytes = hex::decode(&entry.hash)?;
        body.extend(&hash_bytes);
    }

    let header = format!("tree {}\0", body.len());
    let mut store = header.into_bytes();
    store.extend(&body);

    let mut hasher = Sha1::new();
    hasher.update(&store);
    let tree_hash = hex::encode(hasher.finalize());
    
    // Write to disk
   
    let dir_name = &tree_hash[..2];
    let file_name = &tree_hash[2..];
    let object_dir = format!("{}/objects/{}", RIT_DIR, dir_name);
    if !Path::new(&object_dir).exists() { fs::create_dir(&object_dir)?; }
    let path = format!("{}/{}", object_dir, file_name);
    let file = fs::File::create(path)?;
    let mut encoder = ZlibEncoder::new(file, Compression::default());
    encoder.write_all(&store)?;
    encoder.finish()?;

    Ok(tree_hash)
}

pub fn commit_tree(tree_hash: &str, parent_hash: Option<&str>, message: &str) -> Result<String> {
    let mut body = String::new();
    
    //  Add Tree
    body.push_str(&format!("tree {}\n", tree_hash));
    
    // Add Parent (if it exists)
    if let Some(parent) = parent_hash {
        body.push_str(&format!("parent {}\n", parent));
    }
    
    //  Add Author/Committer (Using current time)
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH)?;
    let timestamp = since_the_epoch.as_secs();
    
    let author_string = format!("author Rit User <rit@example.com> {} +0000\n", timestamp);
    let committer_string = format!("committer Rit User <rit@example.com> {} +0000\n", timestamp);
    
    body.push_str(&author_string);
    body.push_str(&committer_string);
    
    // Add Message (preceded by an empty line)
    body.push_str("\n");
    body.push_str(message);
    body.push_str("\n"); // Git usually ends with a newline

    //  Create the object header + body
    let header = format!("commit {}\0", body.len());
    let mut store = header.into_bytes();
    store.extend(body.as_bytes());

    //  Hash and Save (Standard logic)
    let mut hasher = Sha1::new();
    hasher.update(&store);
    let result = hasher.finalize();
    let commit_hash = hex::encode(result);

    // Write to disk
    let dir_name = &commit_hash[..2];
    let file_name = &commit_hash[2..];
    let object_dir = format!("{}/objects/{}", RIT_DIR, dir_name);
    let object_path = format!("{}/{}", object_dir, file_name);

    if !Path::new(&object_dir).exists() {
        fs::create_dir(&object_dir)?;
    }
    
    let file = fs::File::create(&object_path)?;
    let mut encoder = ZlibEncoder::new(file, Compression::default());
    encoder.write_all(&store)?;
    encoder.finish()?;

    Ok(commit_hash)
}

pub fn update_ref(ref_name: &str, oid: &str) -> Result<()> {
    // ref_name will be something like "refs/heads/main"
    let ref_path = format!("{}/{}", RIT_DIR, ref_name);
    
    let path_obj = Path::new(&ref_path);
    if let Some(parent) = path_obj.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // We add a newline because Git creates refs with a trailing newline
    fs::write(&ref_path, format!("{}\n", oid))?;
    
    println!("Updated {} to {}", ref_name, oid);
    Ok(())
}

pub fn add(file_path: &str) -> Result<()> {
    let hash = hash_object(file_path, true)?;

    let index_path = format!("{}/index", RIT_DIR);
    let mut index_map = HashMap::new();

    if Path::new(&index_path).exists() {
        let file = fs::File::open(&index_path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            // format: "path hash"
            if let Some((path, h)) = line.split_once(' ') {
                index_map.insert(path.to_string(), h.to_string());
            }
        }
    }

    let clean_path = file_path.trim_start_matches("./");
    index_map.insert(clean_path.to_string(), hash);

    let mut file = fs::File::create(&index_path)?;
    let mut entries:Vec<_> = index_map.iter().collect();
    entries.sort_by_key(|(path, _)| *path);

    for (path, hash) in entries {
        writeln!(file, "{} {}", path, hash)?;
    }

    println!("Added '{}' ", clean_path);
    Ok(())
}

pub fn commit(message: &str) -> Result<()> {
    // Create the Tree from the Index
    // (This uses your new filtered write_tree logic)
    let tree_hash = write_tree(".")?;
    
    // Find the Parent Commit (if it exists)
    let head_path = format!("{}/HEAD", RIT_DIR);
    let head_content = fs::read_to_string(&head_path)?;
    // head_content is "ref: refs/heads/main\n"
    
    let ref_path_str = head_content.trim().strip_prefix("ref: ").unwrap_or(head_content.trim());
    let full_ref_path = format!("{}/{}", RIT_DIR, ref_path_str);
    
    let parent_hash = if Path::new(&full_ref_path).exists() {
        // If refs/heads/main exists, read the hash inside it
        Some(fs::read_to_string(full_ref_path)?.trim().to_string())
    } else {
        // If it doesn't exist, this is the FIRST commit (Root commit)
        None
    };
    
    // Create the Commit Object
    // We pass parent_hash.as_deref() to convert Option<String> to Option<&str>
    let commit_hash = commit_tree(&tree_hash, parent_hash.as_deref(), message)?;
    
    // Update the Reference (Move the branch pointer)
    update_ref(ref_path_str, &commit_hash)?;
    
    println!("[{}] {}", &commit_hash[..7], message);
    Ok(())
}

