pub mod db;
pub mod migration;
pub mod objects;

use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::metadata;

const DEFAULT_CONFIG: &str = "\
[stdai]
# version = \"1.1\"
";

pub struct Workspace {
    root: PathBuf,
    project: Option<String>,
}

impl Workspace {
    /// Open (or create) the global store at the resolved path.
    /// Auto-migrates any legacy per-project `.stdai/` found in the current project.
    pub fn open() -> Result<Self> {
        let root = global_store_path();

        if !root.join("stdai.db").exists() {
            std::fs::create_dir_all(root.join("objects"))?;
            db::initialize(&root.join("stdai.db"))?;
            std::fs::write(root.join("config.toml"), DEFAULT_CONFIG)?;
        }

        let project = metadata::detect_project();

        if let Some(proj_root) = metadata::project_root() {
            let legacy = proj_root.join(".stdai");
            if legacy.is_dir() {
                if let Err(e) = migration::migrate_legacy(&legacy, &root, project.as_deref()) {
                    eprintln!("stdai: migration warning: {}", e);
                }
            }
        }

        Ok(Self { root, project })
    }

    /// Alias kept for backward compatibility during transition.
    #[allow(dead_code)]
    pub fn find_or_init() -> Result<Self> {
        Self::open()
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn objects_dir(&self) -> PathBuf {
        self.root.join("objects")
    }

    pub fn identities_dir(&self) -> PathBuf {
        self.root.join("identities")
    }

    pub fn db_path(&self) -> PathBuf {
        self.root.join("stdai.db")
    }

    pub fn project(&self) -> Option<&str> {
        self.project.as_deref()
    }
}

/// Resolve the global store path.
/// Priority: $STDAI_HOME > $XDG_DATA_HOME/stdai > ~/.stdai
pub fn global_store_path() -> PathBuf {
    if let Ok(home) = std::env::var("STDAI_HOME") {
        if !home.is_empty() {
            return PathBuf::from(home);
        }
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("stdai");
        }
    }
    home_dir().join(".stdai")
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}
