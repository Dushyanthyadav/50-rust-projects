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
use walkdir::WalkDir;
use std::os::unix::fs::PermissionsExt; 

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

// The public entry point
pub fn add(path: &str) -> Result<()> {
    // Check if the path exists and what type it is
    let metadata = fs::metadata(path)?;

    if metadata.is_dir() {
        // If it's a directory (like "."), walk through it recursively
        for entry in WalkDir::new(path) {
            let entry = entry?;
            let entry_path = entry.path();
            
            // We only want to add files, not sub-directories themselves
            if entry_path.is_file() {
                let path_str = entry_path.to_str().unwrap();
                
                // IMPORTANT: Do not add the .rit folder itself!
                // We check if the path string contains our hidden dir name
                if path_str.contains(RIT_DIR) || path_str.contains(".git") {
                    continue;
                }

                // Call the actual logic
                add_file(path_str)?;
            }
        }
    } else {
        // If it's just a single file, add it directly
        add_file(path)?;
    }
    
    Ok(())
}

fn add_file(file_path: &str) -> Result<()> {
    let hash = hash_object(file_path, true)?;
    
    let index_path = format!("{}/index", RIT_DIR);
    let mut index_map = HashMap::new();
    
    if Path::new(&index_path).exists() {
        let file = fs::File::open(&index_path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if let Some((path, h)) = line.split_once(' ') {
                index_map.insert(path.to_string(), h.to_string());
            }
        }
    }
    
    // Clean path (remove ./ prefix)
    let clean_path = file_path.trim_start_matches("./");
    index_map.insert(clean_path.to_string(), hash);
    
    let mut file = fs::File::create(&index_path)?;
    let mut entries: Vec<_> = index_map.iter().collect();
    entries.sort_by_key(|(path, _)| *path);
    
    for (path, hash) in entries {
        writeln!(file, "{} {}", path, hash)?;
    }
    
    println!("Added '{}'", clean_path);
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

// Make sure you have these imports
pub fn log(oid: &str) -> Result<()> {
    // Resolve the starting commit hash
    let mut current_hash = oid.to_string();

    // If the user didn't pass a specific hash, look up where HEAD points
    if current_hash == "HEAD" {
        let head_path = format!("{}/HEAD", RIT_DIR);
        let head_content = fs::read_to_string(&head_path)?;
        // head_content is like "ref: refs/heads/main\n"
        let ref_path_str = head_content.trim().strip_prefix("ref: ").unwrap_or(head_content.trim());
        let full_ref_path = format!("{}/{}", RIT_DIR, ref_path_str);

        if Path::new(&full_ref_path).exists() {
            current_hash = fs::read_to_string(full_ref_path)?.trim().to_string();
        } else {
            anyhow::bail!("HEAD points to '{}', which does not exist yet. Make a commit first!", ref_path_str);
        }
    }

    //  Walk the graph backwards
    println!("Printing history for commit: {}\n", current_hash);

    loop {
        // Read the commit object
        let dir_name = &current_hash[..2];
        let file_name = &current_hash[2..];
        let object_path = format!("{}/objects/{}/{}", RIT_DIR, dir_name, file_name);
        
        let file = fs::File::open(&object_path).map_err(|_| anyhow::anyhow!("Commit object {} missing", current_hash))?;
        let mut decoder = ZlibDecoder::new(file);
        let mut contents = Vec::new();
        decoder.read_to_end(&mut contents)?;

        // Split header from body (commit <size>\0<body>)
        let null_index = contents.iter().position(|&b| b == 0).unwrap();
        let body_bytes = &contents[null_index + 1..];
        let body = String::from_utf8_lossy(body_bytes);

        //  Parse the commit body to find metadata
        let mut parent = None;
        let mut author = String::from("(unknown)");
        let mut message = String::new();
        
        // We split by lines. The message is separated by an empty line.
        let mut parsing_headers = true;

        for line in body.lines() {
            if parsing_headers {
                if line.is_empty() {
                    parsing_headers = false;
                } else if line.starts_with("parent ") {
                    parent = Some(line["parent ".len()..].to_string());
                } else if line.starts_with("author ") {
                    author = line["author ".len()..].to_string();
                }
            } else {
                // Collect the message
                message.push_str(line);
                message.push('\n');
            }
        }

        // Pretty print the commit info
        // \x1b[33m makes the hash yellow in the terminal
        println!("\x1b[33mcommit {}\x1b[0m", current_hash);
        println!("Author: {}", author);
        println!("\n    {}", message.trim());
        println!("{}", "-".repeat(50));

        // Move to parent or stop
        match parent {
            Some(p) => current_hash = p,
            None => break, // Reached the root commit
        }
    }

    Ok(())
}


pub fn checkout(target: &str) -> Result<()> {
    // Resolve target: Is it a branch name (like 'main') or a Hash?
    let refs_path = format!("{}/refs/heads/{}", RIT_DIR, target);
    let mut commit_hash = target.to_string();
    let mut new_head_content = format!("ref: refs/heads/{}\n", target);

    if Path::new(&refs_path).exists() {
        // It is a branch! Read the hash inside it.
        commit_hash = fs::read_to_string(&refs_path)?.trim().to_string();
    } else {
        // It's likely a raw commit hash (Detached HEAD state)
        // Verify object exists
        let dir = &commit_hash[..2];
        let file = &commit_hash[2..];
        if !Path::new(&format!("{}/objects/{}/{}", RIT_DIR, dir, file)).exists() {
            anyhow::bail!("Target '{}' does not exist.", target);
        }
        // In detached HEAD, HEAD contains the hash directly, not a ref: path
        new_head_content = format!("{}\n", commit_hash);
    }

    // Read the Commit Object to find the Root Tree
    let commit_content = read_object_content(&commit_hash)?;
    // Content is "tree <hash>\nparent..."
    // We parse the first line
    let tree_line = commit_content.lines().next().ok_or(anyhow::anyhow!("Invalid commit"))?;
    let tree_hash = tree_line.strip_prefix("tree ").ok_or(anyhow::anyhow!("Invalid commit format"))?;

    // Clear current directory (Optional safety step)
    // In a real git, this is complex. Here, we just overwrite files.
    // Ideally, we would delete files that exist here but NOT in the new tree.
    
    // Restore the Tree recursively
    restore_tree(tree_hash, Path::new("."))?;

    //  Update HEAD
    fs::write(format!("{}/HEAD", RIT_DIR), new_head_content)?;

    println!("Switched to '{}'", target);
    Ok(())
}

// Helper to read and decompress any object
fn read_object_content(hash: &str) -> Result<String> {
    let dir = &hash[..2];
    let file = &hash[2..];
    let path = format!("{}/objects/{}/{}", RIT_DIR, dir, file);
    let f = fs::File::open(path)?;
    let mut decoder = ZlibDecoder::new(f);
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;
    
    // Split header and body
    let null_index = buffer.iter().position(|&b| b == 0).unwrap();
    let body = String::from_utf8_lossy(&buffer[null_index+1..]).to_string();
    Ok(body)
}

// The Recursive Restorer
fn restore_tree(tree_hash: &str, current_path: &Path) -> Result<()> {
    // Get raw bytes of the tree object
    // (We can't use read_object_content because trees contain raw binary SHAs, not text)
    let dir = &tree_hash[..2];
    let file = &tree_hash[2..];
    let path = format!("{}/objects/{}/{}", RIT_DIR, dir, file);
    
    let f = fs::File::open(path)?;
    let mut decoder = ZlibDecoder::new(f);
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;
    
    // Skip header "tree <size>\0"
    let null_index = buffer.iter().position(|&b| b == 0).unwrap();
    let mut body = &buffer[null_index+1..];

    // Parse entries loop
    while !body.is_empty() {
        // Format: [mode] [space] [name] [null] [20 bytes sha]
        
        // Find null byte after mode/name
        let null_idx = body.iter().position(|&b| b == 0).unwrap();
        let mode_name = std::str::from_utf8(&body[..null_idx])?;
        let (mode, name) = mode_name.split_once(' ').unwrap();
        
        // Advance past null byte
        body = &body[null_idx + 1..];
        
        // Read 20 bytes for SHA
        let sha_bytes = &body[..20];
        let sha_hex = hex::encode(sha_bytes);
        body = &body[20..]; // Advance for next iteration

        let entry_path = current_path.join(name);

        if mode == "40000" {
            // It's a directory
            if !entry_path.exists() {
                fs::create_dir(&entry_path)?;
            }
            restore_tree(&sha_hex, &entry_path)?;
        } else {
            // It's a file (blob)
            // Read the blob content
            // We reuse our text reader, assuming files are text. 
            // For binary files, you'd need a Vec<u8> version.
            let content = read_object_content(&sha_hex)?;
            fs::write(&entry_path, content)?;
            
            // Optional: Set executable permissions if mode is 100755
            // if mode == "100755" { ... }
        }
    }
    
    Ok(())
}


