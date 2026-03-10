pub mod db;
pub mod objects;

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};

const DEFAULT_CONFIG: &str = "\
[stdai]
# Default configuration for stdai workspace
# version = \"0.1\"
";

pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// Walk up from cwd looking for an existing `.stdai/` directory.
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

    /// Find an existing workspace, or transparently create one.
    /// Prefers the git repo root; falls back to cwd.
    pub fn find_or_init() -> Result<Self> {
        match Self::find() {
            Ok(ws) => Ok(ws),
            Err(Error::NotInitialized) => {
                let target = auto_init_target();
                let ws = Self::init_at(&target)?;
                eprintln!(
                    "stdai: auto-initialized workspace at {}",
                    ws.root().display()
                );
                Ok(ws)
            }
            Err(e) => Err(e),
        }
    }

    /// Explicitly create a workspace (used by `stdai init`).
    pub fn create(at: &Path) -> Result<Self> {
        let root = at.join(".stdai");
        if root.is_dir() {
            return Err(Error::AlreadyInitialized(root.display().to_string()));
        }
        Self::init_at(at)
    }

    fn init_at(at: &Path) -> Result<Self> {
        let root = at.join(".stdai");
        std::fs::create_dir_all(root.join("objects"))?;
        db::initialize(&root.join("stdai.db"))?;
        std::fs::write(root.join("config.toml"), DEFAULT_CONFIG)?;
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

    #[allow(dead_code)]
    pub fn config_path(&self) -> PathBuf {
        self.root.join("config.toml")
    }
}

fn auto_init_target() -> PathBuf {
    Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| PathBuf::from(s.trim()))
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine current directory"))
}
