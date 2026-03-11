pub mod address;
pub mod keys;
pub mod signing;

use std::path::Path;

pub use keys::Identity;

use crate::error::{Error, Result};

/// Resolve which identity to use for signing.
///
/// Priority: explicit flag > $STDAI_IDENTITY env var > error with instructions.
pub fn resolve_identity(
    flag: Option<&str>,
    store_root: &Path,
) -> Result<Identity> {
    let identities_dir = store_root.join("identities");

    let addr_input = if let Some(a) = flag {
        Some(a.to_string())
    } else if let Ok(env_val) = std::env::var("STDAI_IDENTITY") {
        if !env_val.is_empty() {
            Some(env_val)
        } else {
            None
        }
    } else {
        None
    };

    let addr_input = match addr_input {
        Some(a) => a,
        None => return Err(Error::IdentityRequired),
    };

    let resolved_addr = keys::resolve_address(&identities_dir, &addr_input)?;
    keys::load_identity_meta(&identities_dir, &resolved_addr)
}
