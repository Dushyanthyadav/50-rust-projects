use anyhow::Result;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

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

pub fn hash_object(file_path: &str, write: bool) -> Result<(String)> {
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
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().into_string().unwrap();

        if name == RIT_DIR || name == ".git" || name == "target" {
            continue;
        }

        if path.is_dir() {
            // recursive call!
            // Git convention: mode 40000 for trees
            let dir_hash = write_tree(path.to_str().unwrap())?;
            entries.push(TreeEntry {
                name,
                mode: "40000".to_string(),
                hash: dir_hash,
            });
        } else {
            // it is file
            // git convention: mode 100644 for regular files
            // for executable files it is 100755 but will use the 100644 for all the files
            let file_hash = hash_object(path.to_str().unwrap(), true)?;
            entries.push(TreeEntry{
                name,
                mode: "100644".to_ascii_lowercase(),
                hash: file_hash,
            });
        }
    }

    // sort entries by name required for consistent hashing
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    // building the tree content
    let mut body = Vec::new();
    for entry in entries {
        let line = format!("{} {}\0", entry.mode, entry.name);
        body.extend(line.as_bytes());

        let hash_bytes = hex::decode(&entry.hash)?;
        body.extend(&hash_bytes);
    }

    // store the tree object
    // Like blobs,trees are stored as "tree <size>\0<content>"
    let header = format!("tree {}\0", body.len());
    let mut store = header.into_bytes();
    store.extend(&body);

    let mut hasher = Sha1::new();
    hasher.update(&store);
    let result = hasher.finalize();
    let tree_hash = hex::encode(result);

    // write to .rit/objects
    let dir_name = &tree_hash[..2];
    let file_name = &tree_hash[2..];
    let object_dir = format!("{}/objects/{}", RIT_DIR, dir_name);
    let object_path = format!("{}/{}", object_dir, file_name);

    if !Path::new(&object_dir).exists() {
        fs::create_dir(&object_dir)?;
    }

    let file = fs::File::create(&object_path)?;
    let mut encoder = ZlibEncoder::new(file, Compression::default());
    encoder.write(&store)?;
    encoder.finish()?;

    Ok(tree_hash)
}







