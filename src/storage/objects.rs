use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::Result;

/// Store content in the object store. Returns (hash, relative_object_path).
/// Skips writing if the object already exists (content-addressed dedup).
pub fn store(objects_dir: &Path, content: &[u8]) -> Result<(String, String)> {
    let hash = compute_hash(content);
    let (dir_prefix, file_name) = hash.split_at(2);
    let dir = objects_dir.join(dir_prefix);
    let path = dir.join(file_name);

    if !path.exists() {
        fs::create_dir_all(&dir)?;
        fs::write(&path, content)?;
    }

    let object_path = format!("{}/{}", dir_prefix, file_name);
    Ok((hash, object_path))
}

pub fn load(objects_dir: &Path, hash: &str) -> Result<Vec<u8>> {
    if hash.len() < 3 {
        return Err(crate::error::Error::Other(format!(
            "invalid hash: {}",
            hash
        )));
    }
    let (dir_prefix, file_name) = hash.split_at(2);
    let path = objects_dir.join(dir_prefix).join(file_name);
    Ok(fs::read(path)?)
}

pub fn compute_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_hash() {
        let h1 = compute_hash(b"hello world");
        let h2 = compute_hash(b"hello world");
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_content_different_hash() {
        let h1 = compute_hash(b"hello");
        let h2 = compute_hash(b"world");
        assert_ne!(h1, h2);
    }
}
