use ed25519_dalek::{Signature, Signer, Verifier, VerifyingKey, SigningKey};
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

/// Build the payload that gets signed: SHA-256(content_hash || kind || created_at || agent_id)
/// with NUL byte separators.
pub fn build_signing_payload(
    content_hash: &str,
    kind: Option<&str>,
    created_at: &str,
    agent_id: Option<&str>,
) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(content_hash.as_bytes());
    buf.push(0);
    buf.extend_from_slice(kind.unwrap_or("").as_bytes());
    buf.push(0);
    buf.extend_from_slice(created_at.as_bytes());
    buf.push(0);
    buf.extend_from_slice(agent_id.unwrap_or("").as_bytes());
    let hash = Sha256::digest(&buf);
    hash.to_vec()
}

pub fn sign(signing_key: &SigningKey, payload: &[u8]) -> String {
    let sig = signing_key.sign(payload);
    hex::encode(sig.to_bytes())
}

pub fn verify(pubkey_hex: &str, payload: &[u8], signature_hex: &str) -> Result<bool> {
    let pubkey_bytes = hex::decode(pubkey_hex)
        .map_err(|e| Error::SignatureInvalid(format!("bad public key hex: {}", e)))?;
    let pubkey_array: [u8; 32] = pubkey_bytes
        .try_into()
        .map_err(|_| Error::SignatureInvalid("public key must be 32 bytes".to_string()))?;
    let verifying_key = VerifyingKey::from_bytes(&pubkey_array)
        .map_err(|e| Error::SignatureInvalid(format!("invalid public key: {}", e)))?;

    let sig_bytes = hex::decode(signature_hex)
        .map_err(|e| Error::SignatureInvalid(format!("bad signature hex: {}", e)))?;
    let sig_array: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| Error::SignatureInvalid("signature must be 64 bytes".to_string()))?;
    let signature = Signature::from_bytes(&sig_array);

    match verifying_key.verify(payload, &signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}
