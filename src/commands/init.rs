use std::path::Path;

use crate::error::Result;
use crate::storage::db;
use crate::storage::Workspace;

pub fn run(path: &Path) -> Result<()> {
    let ws = Workspace::create(path)?;
    db::initialize(&ws.db_path())?;

    let config = "\
[stdai]
# Default configuration for stdai workspace
# version = \"0.1\"
";
    std::fs::write(ws.config_path(), config)?;

    eprintln!("initialized stdai workspace at {}", ws.root().display());
    Ok(())
}
