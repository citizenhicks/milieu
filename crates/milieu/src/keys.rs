use crate::api::{ApiClient, RepoKeyResponse, UserKeyResponse};
use crate::auth;
use crate::crypto::{
    decode_key, encode_key, generate_keypair, public_key_from_private,
    unwrap_repo_key_with_private_key, wrap_repo_key_for_public_key, KeyPair, UMK_LEN,
};
use crate::error::{MilieuError, Result};
use crate::keychain;

const USER_KEYPAIR_KEY_PREFIX: &str = "user_keypair:";
const REPO_KEY_PREFIX: &str = "repo_key:";

fn profile_email(profile: &str) -> Result<String> {
    auth::load_email(profile)?
        .ok_or_else(|| MilieuError::CommandFailed("missing email; run `milieu login`".to_string()))
}

fn user_keypair_key(email: &str) -> String {
    format!("{}{}", USER_KEYPAIR_KEY_PREFIX, email)
}

fn repo_key_key(email: &str, repo_id: &str) -> String {
    format!("{}{}:{}", REPO_KEY_PREFIX, email, repo_id)
}

pub async fn ensure_user_keypair(
    profile: &str,
    client: &ApiClient,
) -> Result<KeyPair> {
    let email = profile_email(profile)?;
    let stored = keychain::get_secret(&user_keypair_key(&email))?;
    let local = match stored.as_ref() {
        Some(private_key_b64) => KeyPair {
            private_key_b64: private_key_b64.clone(),
            public_key_b64: public_key_from_private(private_key_b64)?,
        },
        None => generate_keypair()?,
    };

    let remote = client.get_user_key().await.ok().flatten();
    match remote {
        Some(UserKeyResponse { public_key, .. }) => {
            if public_key != local.public_key_b64 {
                // Keep local keypair and overwrite remote to match.
                client
                    .put_user_key(&local.public_key_b64, "x25519-hkdf-xchacha20poly1305")
                    .await?;
            }
        }
        None => {
            client
                .put_user_key(&local.public_key_b64, "x25519-hkdf-xchacha20poly1305")
                .await?;
        }
    }

    if stored.is_none() {
        keychain::set_secret(&user_keypair_key(&email), &local.private_key_b64)?;
    }

    Ok(local)
}

pub fn store_repo_key(profile: &str, repo_id: &str, key: &[u8; UMK_LEN]) -> Result<()> {
    let email = profile_email(profile)?;
    let encoded = encode_key(key);
    keychain::set_secret(&repo_key_key(&email, repo_id), &encoded)?;
    Ok(())
}

pub fn load_repo_key(profile: &str, repo_id: &str) -> Result<Option<[u8; UMK_LEN]>> {
    let email = profile_email(profile)?;
    let encoded = keychain::get_secret(&repo_key_key(&email, repo_id))?;
    match encoded {
        Some(value) => Ok(Some(decode_key(&value)?)),
        None => Ok(None),
    }
}

pub async fn get_or_fetch_repo_key(
    profile: &str,
    client: &ApiClient,
    repo_id: &str,
) -> Result<[u8; UMK_LEN]> {
    if let Some(key) = load_repo_key(profile, repo_id)? {
        return Ok(key);
    }

    let email = profile_email(profile)?;
    let private_key = keychain::get_secret(&user_keypair_key(&email))?
        .ok_or_else(|| MilieuError::CommandFailed("missing user keypair; run `milieu login` again".to_string()))?;

    let RepoKeyResponse { wrapped_key, .. } = client
        .get_repo_key(repo_id)
        .await?
        .ok_or_else(|| {
            MilieuError::CommandFailed(
                "repo key not shared yet; ask the owner to run `milieu repos manage share --repo <name>`"
                    .to_string(),
            )
        })?;

    let repo_key = unwrap_repo_key_with_private_key(&private_key, &wrapped_key)?;
    store_repo_key(profile, repo_id, &repo_key)?;
    Ok(repo_key)
}

pub async fn wrap_repo_key_for_user(
    recipient_public_key: &str,
    repo_key: &[u8; UMK_LEN],
) -> Result<String> {
    Ok(wrap_repo_key_for_public_key(repo_key, recipient_public_key)?)
}
