use crate::error::Result;

pub fn run() -> Result<()> {
    eprintln!(
        "stdai: `init` is no longer needed — the global store auto-creates on first use.\n\
         Storage location: {}",
        crate::storage::global_store_path().display(),
    );
    Ok(())
}
