use crate::error::{MilieuError, Result};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};

pub const UMK_LEN: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    pub salt: String,
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

impl KdfParams {
    pub fn new_default() -> Self {
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        Self {
            salt: B64.encode(salt),
            m_cost: 65536,
            t_cost: 3,
            p_cost: 1,
        }
    }
}

pub fn derive_key(passphrase: &str, params: &KdfParams) -> Result<[u8; UMK_LEN]> {
    let salt = B64.decode(&params.salt).map_err(|e| {
        MilieuError::Crypto(format!("invalid base64 salt: {}", e))
    })?;

    let argon_params = Params::new(params.m_cost, params.t_cost, params.p_cost, Some(UMK_LEN))
        .map_err(|e| MilieuError::Crypto(format!("argon2 params: {}", e)))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);

    let mut out = [0u8; UMK_LEN];
    argon2
        .hash_password_into(passphrase.as_bytes(), &salt, &mut out)
        .map_err(|e| MilieuError::Crypto(format!("argon2 derive: {}", e)))?;
    Ok(out)
}

pub fn generate_umk() -> [u8; UMK_LEN] {
    let mut key = [0u8; UMK_LEN];
    OsRng.fill_bytes(&mut key);
    key
}

pub fn encrypt_bytes(key: &[u8; UMK_LEN], aad: &[u8], plaintext: &[u8]) -> Result<(String, String)> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);
    let nonce_ref = XNonce::from_slice(&nonce);

    let ciphertext = cipher
        .encrypt(nonce_ref, Payload { msg: plaintext, aad })
        .map_err(|e| MilieuError::Crypto(format!("encrypt: {}", e)))?;

    Ok((B64.encode(nonce), B64.encode(ciphertext)))
}

pub fn decrypt_bytes(
    key: &[u8; UMK_LEN],
    aad: &[u8],
    nonce_b64: &str,
    ciphertext_b64: &str,
) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key));
    let nonce_bytes = B64.decode(nonce_b64).map_err(|e| {
        MilieuError::Crypto(format!("invalid nonce base64: {}", e))
    })?;
    if nonce_bytes.len() != 24 {
        return Err(MilieuError::Crypto("invalid nonce length".to_string()));
    }
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ciphertext = B64.decode(ciphertext_b64).map_err(|e| {
        MilieuError::Crypto(format!("invalid ciphertext base64: {}", e))
    })?;

    let plaintext = cipher
        .decrypt(nonce, Payload { msg: &ciphertext, aad })
        .map_err(|e| MilieuError::Crypto(format!("decrypt: {}", e)))?;
    Ok(plaintext)
}

pub fn aad_for(
    schema_version: u32,
    repo_id: &str,
    branch: &str,
    path: &str,
    tag: Option<&str>,
) -> Vec<u8> {
    let tag_value = tag.unwrap_or("-");
    format!(
        "v{}|{}|{}|{}|{}",
        schema_version, repo_id, branch, path, tag_value
    )
    .into_bytes()
}

pub fn encode_key(key: &[u8; UMK_LEN]) -> String {
    B64.encode(key)
}

pub fn decode_key(encoded: &str) -> Result<[u8; UMK_LEN]> {
    let bytes = B64.decode(encoded).map_err(|e| {
        MilieuError::Crypto(format!("invalid key base64: {}", e))
    })?;
    if bytes.len() != UMK_LEN {
        return Err(MilieuError::Crypto("invalid key length".to_string()));
    }
    let mut key = [0u8; UMK_LEN];
    key.copy_from_slice(&bytes);
    Ok(key)
}

pub fn encrypt_umk_blob(pdk: &[u8; UMK_LEN], umk: &[u8; UMK_LEN]) -> Result<String> {
    let aad = b"milieu:umk:v1";
    let (nonce, ciphertext) = encrypt_bytes(pdk, aad, umk)?;
    Ok(format!("{}:{}", nonce, ciphertext))
}

pub fn decrypt_umk_blob(pdk: &[u8; UMK_LEN], blob: &str) -> Result<[u8; UMK_LEN]> {
    let mut parts = blob.splitn(2, ':');
    let nonce = parts
        .next()
        .ok_or_else(|| MilieuError::Crypto("invalid umk blob".to_string()))?;
    let ciphertext = parts
        .next()
        .ok_or_else(|| MilieuError::Crypto("invalid umk blob".to_string()))?;
    let aad = b"milieu:umk:v1";
    let bytes = decrypt_bytes(pdk, aad, nonce, ciphertext)?;
    if bytes.len() != UMK_LEN {
        return Err(MilieuError::Crypto("invalid umk length".to_string()));
    }
    let mut umk = [0u8; UMK_LEN];
    umk.copy_from_slice(&bytes);
    Ok(umk)
}
