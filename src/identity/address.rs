use sha2::{Digest, Sha256};

pub fn derive_address(pubkey_bytes: &[u8]) -> String {
    let hash = Sha256::digest(pubkey_bytes);
    let truncated = &hash[..20];
    format!("stdai:{}", hex::encode(truncated))
}

pub fn strip_prefix(address: &str) -> &str {
    address.strip_prefix("stdai:").unwrap_or(address)
}
