use std::path::Path;

use crate::error::Result;
use crate::storage::Workspace;

pub fn run(path: &Path) -> Result<()> {
    let ws = Workspace::create(path)?;
    eprintln!("initialized stdai workspace at {}", ws.root().display());
    Ok(())
}
