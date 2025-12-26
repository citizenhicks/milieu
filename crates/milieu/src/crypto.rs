use crate::error::{MilieuError, Result};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

pub const UMK_LEN: usize = 32;
const REPO_KEY_AAD: &[u8] = b"milieu:repo-key:v1";
const REPO_KEY_WRAP_INFO: &[u8] = b"milieu:repo-key-wrap";
const USER_KEYPAIR_INFO: &[u8] = b"milieu:user-keypair:v1";

#[derive(Debug, Clone)]
pub struct KeyPair {
    pub private_key_b64: String,
    pub public_key_b64: String,
}

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

pub fn generate_keypair() -> Result<KeyPair> {
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);
    Ok(KeyPair {
        private_key_b64: B64.encode(secret.to_bytes()),
        public_key_b64: B64.encode(public.to_bytes()),
    })
}

pub fn derive_keypair_from_umk(umk: &[u8; UMK_LEN]) -> Result<KeyPair> {
    let hk = Hkdf::<Sha256>::new(None, umk);
    let mut seed = [0u8; 32];
    hk.expand(USER_KEYPAIR_INFO, &mut seed)
        .map_err(|_| MilieuError::Crypto("hkdf expand failed".to_string()))?;
    let secret = StaticSecret::from(seed);
    let public = PublicKey::from(&secret);
    Ok(KeyPair {
        private_key_b64: B64.encode(secret.to_bytes()),
        public_key_b64: B64.encode(public.to_bytes()),
    })
}

pub fn public_key_from_private(private_key_b64: &str) -> Result<String> {
    let bytes = B64.decode(private_key_b64).map_err(|e| {
        MilieuError::Crypto(format!("invalid private key base64: {}", e))
    })?;
    if bytes.len() != 32 {
        return Err(MilieuError::Crypto("invalid private key length".to_string()));
    }
    let mut buf = [0u8; 32];
    buf.copy_from_slice(&bytes);
    let secret = StaticSecret::from(buf);
    let public = PublicKey::from(&secret);
    Ok(B64.encode(public.to_bytes()))
}

pub fn wrap_repo_key_for_public_key(
    repo_key: &[u8; UMK_LEN],
    recipient_public_b64: &str,
) -> Result<String> {
    let recipient_bytes = B64.decode(recipient_public_b64).map_err(|e| {
        MilieuError::Crypto(format!("invalid public key base64: {}", e))
    })?;
    if recipient_bytes.len() != 32 {
        return Err(MilieuError::Crypto("invalid public key length".to_string()));
    }
    let mut pk_bytes = [0u8; 32];
    pk_bytes.copy_from_slice(&recipient_bytes);
    let recipient = PublicKey::from(pk_bytes);

    let ephemeral = StaticSecret::random_from_rng(OsRng);
    let ephemeral_public = PublicKey::from(&ephemeral);
    let shared = ephemeral.diffie_hellman(&recipient);

    let hk = Hkdf::<Sha256>::new(None, shared.as_bytes());
    let mut key_bytes = [0u8; UMK_LEN];
    hk.expand(REPO_KEY_WRAP_INFO, &mut key_bytes)
        .map_err(|_| MilieuError::Crypto("hkdf expand failed".to_string()))?;

    let (nonce, ciphertext) = encrypt_bytes(&key_bytes, REPO_KEY_AAD, repo_key)?;
    Ok(format!(
        "v1:{}:{}:{}",
        B64.encode(ephemeral_public.to_bytes()),
        nonce,
        ciphertext
    ))
}

pub fn unwrap_repo_key_with_private_key(
    private_key_b64: &str,
    blob: &str,
) -> Result<[u8; UMK_LEN]> {
    let mut parts = blob.splitn(4, ':');
    let version = parts
        .next()
        .ok_or_else(|| MilieuError::Crypto("invalid repo key blob".to_string()))?;
    if version != "v1" {
        return Err(MilieuError::Crypto("unsupported repo key version".to_string()));
    }
    let eph_b64 = parts
        .next()
        .ok_or_else(|| MilieuError::Crypto("invalid repo key blob".to_string()))?;
    let nonce = parts
        .next()
        .ok_or_else(|| MilieuError::Crypto("invalid repo key blob".to_string()))?;
    let ciphertext = parts
        .next()
        .ok_or_else(|| MilieuError::Crypto("invalid repo key blob".to_string()))?;

    let private_bytes = B64.decode(private_key_b64).map_err(|e| {
        MilieuError::Crypto(format!("invalid private key base64: {}", e))
    })?;
    if private_bytes.len() != 32 {
        return Err(MilieuError::Crypto("invalid private key length".to_string()));
    }
    let mut priv_buf = [0u8; 32];
    priv_buf.copy_from_slice(&private_bytes);
    let secret = StaticSecret::from(priv_buf);

    let eph_bytes = B64.decode(eph_b64).map_err(|e| {
        MilieuError::Crypto(format!("invalid ephemeral key base64: {}", e))
    })?;
    if eph_bytes.len() != 32 {
        return Err(MilieuError::Crypto("invalid ephemeral key length".to_string()));
    }
    let mut eph_buf = [0u8; 32];
    eph_buf.copy_from_slice(&eph_bytes);
    let eph_public = PublicKey::from(eph_buf);

    let shared = secret.diffie_hellman(&eph_public);
    let hk = Hkdf::<Sha256>::new(None, shared.as_bytes());
    let mut key_bytes = [0u8; UMK_LEN];
    hk.expand(REPO_KEY_WRAP_INFO, &mut key_bytes)
        .map_err(|_| MilieuError::Crypto("hkdf expand failed".to_string()))?;

    let plaintext = decrypt_bytes(&key_bytes, REPO_KEY_AAD, nonce, ciphertext)?;
    if plaintext.len() != UMK_LEN {
        return Err(MilieuError::Crypto("invalid repo key length".to_string()));
    }
    let mut repo_key = [0u8; UMK_LEN];
    repo_key.copy_from_slice(&plaintext);
    Ok(repo_key)
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
