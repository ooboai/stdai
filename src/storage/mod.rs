pub mod db;
pub mod objects;

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    pub fn find() -> Result<Self> {
        let mut dir = std::env::current_dir()?;
        loop {
            let candidate = dir.join(".stdai");
            if candidate.is_dir() {
                return Ok(Self { root: candidate });
            }
            if !dir.pop() {
                return Err(Error::NotInitialized);
            }
        }
    }

    pub fn create(at: &Path) -> Result<Self> {
        let root = at.join(".stdai");
        if root.is_dir() {
            return Err(Error::AlreadyInitialized(root.display().to_string()));
        }
        std::fs::create_dir_all(root.join("objects"))?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn objects_dir(&self) -> PathBuf {
        self.root.join("objects")
    }

    pub fn db_path(&self) -> PathBuf {
        self.root.join("stdai.db")
    }

    pub fn config_path(&self) -> PathBuf {
        self.root.join("config.toml")
    }
}
