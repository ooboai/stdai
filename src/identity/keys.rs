use std::fs;
use std::path::Path;

use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use super::address;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub created_at: String,
    pub has_secret: bool,
}

#[derive(Serialize, Deserialize)]
struct IdentityToml {
    address: String,
    label: Option<String>,
    created_at: String,
}

pub fn generate_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

pub fn save_identity(
    identities_root: &Path,
    label: Option<&str>,
    signing_key: &SigningKey,
) -> Result<Identity> {
    let verifying_key = signing_key.verifying_key();
    let addr = address::derive_address(verifying_key.as_bytes());
    let dir = identities_root.join(address::strip_prefix(&addr));
    fs::create_dir_all(&dir)?;

    let secret_path = dir.join("secret.key");
    fs::write(&secret_path, hex::encode(signing_key.to_bytes()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&secret_path, fs::Permissions::from_mode(0o600))?;
    }

    fs::write(dir.join("public.key"), hex::encode(verifying_key.as_bytes()))?;

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let meta = IdentityToml {
        address: addr.clone(),
        label: label.map(|s| s.to_string()),
        created_at: now.clone(),
    };
    fs::write(dir.join("identity.toml"), toml::to_string_pretty(&meta).unwrap())?;

    Ok(Identity {
        address: addr,
        label: label.map(|s| s.to_string()),
        created_at: now,
        has_secret: true,
    })
}

pub fn load_signing_key(identities_root: &Path, addr: &str) -> Result<SigningKey> {
    let bare = address::strip_prefix(addr);
    let dir = identities_root.join(bare);
    let secret_path = dir.join("secret.key");
    if !secret_path.exists() {
        return Err(Error::IdentityNotFound(format!(
            "{} (no secret key — import-only identity?)",
            addr
        )));
    }
    let hex_str = fs::read_to_string(&secret_path)?;
    let bytes = hex::decode(hex_str.trim())
        .map_err(|e| Error::Other(format!("corrupt secret key: {}", e)))?;
    let key_bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| Error::Other("secret key must be 32 bytes".to_string()))?;
    Ok(SigningKey::from_bytes(&key_bytes))
}

pub fn load_verifying_key(identities_root: &Path, addr: &str) -> Result<ed25519_dalek::VerifyingKey> {
    let bare = address::strip_prefix(addr);
    let dir = identities_root.join(bare);
    let pub_path = dir.join("public.key");
    if !pub_path.exists() {
        return Err(Error::IdentityNotFound(addr.to_string()));
    }
    let hex_str = fs::read_to_string(&pub_path)?;
    let bytes = hex::decode(hex_str.trim())
        .map_err(|e| Error::Other(format!("corrupt public key: {}", e)))?;
    let key_bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| Error::Other("public key must be 32 bytes".to_string()))?;
    ed25519_dalek::VerifyingKey::from_bytes(&key_bytes)
        .map_err(|e| Error::Other(format!("invalid public key: {}", e)))
}

pub fn load_identity_meta(identities_root: &Path, addr: &str) -> Result<Identity> {
    let bare = address::strip_prefix(addr);
    let dir = identities_root.join(bare);
    let toml_path = dir.join("identity.toml");
    if !toml_path.exists() {
        return Err(Error::IdentityNotFound(addr.to_string()));
    }
    let content = fs::read_to_string(&toml_path)?;
    let meta: IdentityToml = toml::from_str(&content)
        .map_err(|e| Error::Other(format!("corrupt identity.toml: {}", e)))?;
    let has_secret = dir.join("secret.key").exists();
    Ok(Identity {
        address: meta.address,
        label: meta.label,
        created_at: meta.created_at,
        has_secret,
    })
}

pub fn list_identities(identities_root: &Path) -> Result<Vec<Identity>> {
    if !identities_root.exists() {
        return Ok(Vec::new());
    }
    let mut identities = Vec::new();
    for entry in fs::read_dir(identities_root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let addr = format!("stdai:{}", entry.file_name().to_string_lossy());
            if let Ok(id) = load_identity_meta(identities_root, &addr) {
                identities.push(id);
            }
        }
    }
    identities.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(identities)
}

/// Resolve an address by prefix match against local identities.
pub fn resolve_address(identities_root: &Path, partial: &str) -> Result<String> {
    let bare = address::strip_prefix(partial);
    let full = format!("stdai:{}", bare);
    let dir = identities_root.join(bare);
    if dir.exists() {
        return Ok(full);
    }
    // Try prefix match
    if !identities_root.exists() {
        return Err(Error::IdentityNotFound(partial.to_string()));
    }
    let mut matches = Vec::new();
    for entry in fs::read_dir(identities_root)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(bare) {
            matches.push(format!("stdai:{}", name));
        }
    }
    match matches.len() {
        0 => Err(Error::IdentityNotFound(partial.to_string())),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => Err(Error::Other(format!(
            "ambiguous identity prefix '{}' — matches: {}",
            partial,
            matches.join(", ")
        ))),
    }
}

pub fn import_pubkey(
    identities_root: &Path,
    pubkey_hex: &str,
    label: Option<&str>,
) -> Result<Identity> {
    let bytes = hex::decode(pubkey_hex.trim())
        .map_err(|e| Error::Other(format!("invalid public key hex: {}", e)))?;
    if bytes.len() != 32 {
        return Err(Error::Other("public key must be 32 bytes (64 hex chars)".to_string()));
    }
    let addr = address::derive_address(&bytes);
    let dir = identities_root.join(address::strip_prefix(&addr));
    fs::create_dir_all(&dir)?;

    fs::write(dir.join("public.key"), pubkey_hex.trim())?;

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let meta = IdentityToml {
        address: addr.clone(),
        label: label.map(|s| s.to_string()),
        created_at: now.clone(),
    };
    fs::write(dir.join("identity.toml"), toml::to_string_pretty(&meta).unwrap())?;

    Ok(Identity {
        address: addr,
        label: label.map(|s| s.to_string()),
        created_at: now,
        has_secret: false,
    })
}
