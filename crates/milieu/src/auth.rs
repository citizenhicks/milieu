use crate::error::{MilieuError, Result};
use crate::keychain;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SessionSecret {
    auth_token: Option<String>,
    user_id: Option<String>,
    umk: Option<String>,
    phrase: Option<String>,
    email: Option<String>,
}

static SESSION_CACHE: OnceLock<Mutex<HashMap<String, SessionSecret>>> = OnceLock::new();

fn cache() -> &'static Mutex<HashMap<String, SessionSecret>> {
    SESSION_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn profile_key(profile: &str) -> String {
    profile.trim().to_lowercase()
}

fn session_key(profile: &str) -> String {
    format!("session:{}", profile_key(profile))
}

fn load_session(profile: &str) -> Result<SessionSecret> {
    let key = profile_key(profile);
    if let Ok(cache) = cache().lock() {
        if let Some(entry) = cache.get(&key) {
            return Ok(entry.clone());
        }
    }

    let entry = keychain::get_secret(&session_key(profile))?;
    let secret = match entry.as_ref() {
        Some(value) => serde_json::from_str::<SessionSecret>(value)
            .map_err(|e| MilieuError::Json(e))?,
        None => SessionSecret::default(),
    };


    if let Ok(mut cache) = cache().lock() {
        cache.insert(key, secret.clone());
    }

    Ok(secret)
}

fn store_session(profile: &str, secret: &SessionSecret) -> Result<()> {
    let key = profile_key(profile);
    let data = serde_json::to_string(secret)?;
    keychain::set_secret(&session_key(profile), &data)?;
    if let Ok(mut cache) = cache().lock() {
        cache.insert(key, secret.clone());
    }
    Ok(())
}

pub fn store_auth(profile: &str, token: &str, user_id: &str) -> Result<()> {
    let mut session = load_session(profile)?;
    session.auth_token = Some(token.to_string());
    session.user_id = Some(user_id.to_string());
    store_session(profile, &session)
}

pub fn store_email(profile: &str, email: &str) -> Result<()> {
    let mut session = load_session(profile)?;
    session.email = Some(email.to_string());
    store_session(profile, &session)
}

pub fn load_email(profile: &str) -> Result<Option<String>> {
    let session = load_session(profile)?;
    Ok(session.email)
}

pub fn load_auth_token(profile: &str) -> Result<String> {
    let session = load_session(profile)?;
    session.auth_token.ok_or(MilieuError::AuthMissing)
}

pub fn load_user_id(profile: &str) -> Result<String> {
    let session = load_session(profile)?;
    session.user_id.ok_or(MilieuError::UserIdMissing)
}

pub fn store_umk(profile: &str, umk_b64: &str) -> Result<()> {
    let mut session = load_session(profile)?;
    session.umk = Some(umk_b64.to_string());
    store_session(profile, &session)
}

pub fn load_umk(profile: &str) -> Result<String> {
    let session = load_session(profile)?;
    session.umk.ok_or(MilieuError::UmkMissing)
}

pub fn store_phrase(profile: &str, phrase: &str) -> Result<()> {
    let mut session = load_session(profile)?;
    session.phrase = Some(phrase.to_string());
    store_session(profile, &session)
}

pub fn load_phrase(profile: &str) -> Result<Option<String>> {
    let session = load_session(profile)?;
    Ok(session.phrase)
}

pub fn clear_auth(profile: &str) -> Result<()> {
    let mut session = load_session(profile)?;
    session.auth_token = None;
    session.user_id = None;
    store_session(profile, &session)
}

pub fn clear_umk(profile: &str) -> Result<()> {
    let mut session = load_session(profile)?;
    session.umk = None;
    session.phrase = None;
    store_session(profile, &session)
}

pub fn delete_session(profile: &str) -> Result<()> {
    let key = profile_key(profile);
    let _ = keychain::delete_secret(&session_key(profile));
    if let Ok(mut cache) = cache().lock() {
        cache.remove(&key);
    }
    Ok(())
}
