use crate::api::{ApiClient, UmkRequest};
use crate::auth;
use crate::commands::prompt;
use crate::config::Config;
use crate::crypto::{
    derive_key, derive_keypair_from_umk, encode_key, encrypt_umk_blob, generate_umk, KdfParams,
};
use crate::error::Result;
use crate::keys;
use crate::style;
use bip39::{Language, Mnemonic};
use chrono::{DateTime, Utc};
use rand_core::{OsRng, RngCore};

const ROTATE_DAYS: i64 = 90;

fn age_days(updated_at: &str) -> Option<i64> {
    let parsed = DateTime::parse_from_rfc3339(updated_at).ok()?;
    let now = Utc::now();
    Some((now - parsed.with_timezone(&Utc)).num_days())
}

fn warn_if_old(updated_at: Option<&str>) {
    let Some(updated_at) = updated_at else { return };
    if let Some(days) = age_days(updated_at) {
        if days >= ROTATE_DAYS {
            println!(
                "{}",
                style::paint(
                    style::YELLOW,
                    &format!(
                        "Key age is {} days. Rotate keys every {} days (milieu user rotate-keys).",
                        days, ROTATE_DAYS
                    )
                )
            );
        }
    }
}

pub async fn info(profile: &str) -> Result<()> {
    crate::commands::print_scope_user(profile);
    let config = Config::load()?;
    let base_url = config.base_url_for(profile)?;
    let token = auth::load_auth_token(profile)?;
    let client = ApiClient::new(&base_url, Some(token))?;

    let email = auth::load_email(profile)?.unwrap_or_else(|| "-".to_string());
    let user_id = auth::load_user_id(profile).unwrap_or_else(|_| "-".to_string());

    let repos = client.get_repos().await.unwrap_or_default();
    let sessions = client.get_sessions().await.unwrap_or_default();
    let active_sessions = sessions.iter().filter(|s| s.active).count();
    let user_key = client.get_user_key().await.ok().flatten();
    let umk = client.get_umk().await.ok().flatten();

    println!("{}", style::bold(style::MAUVE, "User"));
    println!("{} {}", style::paint(style::SUBTEXT1, "Email:"), email);
    println!("{} {}", style::paint(style::SUBTEXT1, "User ID:"), user_id);
    println!(
        "{} {}",
        style::paint(style::SUBTEXT1, "Repos:"),
        repos.len()
    );
    println!(
        "{} {}",
        style::paint(style::SUBTEXT1, "Active sessions:"),
        active_sessions
    );

    if let Some(user_key) = &user_key {
        if let Some(days) = age_days(&user_key.updated_at) {
            println!(
                "{} {} days (rotate every {})",
                style::paint(style::SUBTEXT1, "User key age:"),
                days,
                ROTATE_DAYS
            );
        }
    }
    if let Some(umk) = &umk {
        if let Some(updated_at) = umk.updated_at.as_deref() {
            if let Some(days) = age_days(updated_at) {
                println!(
                    "{} {} days",
                    style::paint(style::SUBTEXT1, "Recovery key age:"),
                    days
                );
            }
        }
    }

    warn_if_old(user_key.as_ref().map(|k| k.updated_at.as_str()));
    Ok(())
}

pub async fn rotate_keys(profile: &str) -> Result<()> {
    crate::commands::print_scope_user(profile);
    let confirm = prompt(
        "Rotate recovery phrase + user keys? This will rewrap repo keys. [y/N] ",
    )?;
    let confirm = confirm.trim().to_lowercase();
    if confirm != "y" && confirm != "yes" {
        println!("{}", style::paint(style::SUBTEXT1, "aborted"));
        return Ok(());
    }

    let config = Config::load()?;
    let base_url = config.base_url_for(profile)?;
    let token = auth::load_auth_token(profile)?;
    let client = ApiClient::new(&base_url, Some(token))?;

    let repos = client.get_repos().await?;
    let mut repo_keys = Vec::new();
    for repo in repos {
        let key = keys::get_or_fetch_repo_key(profile, &client, &repo.repo_id).await?;
        repo_keys.push((repo.repo_id, key));
    }

    let umk = generate_umk();
    let mut entropy = [0u8; 16];
    OsRng.fill_bytes(&mut entropy);
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
        .map_err(|e| crate::error::MilieuError::Crypto(e.to_string()))?;
    let phrase = mnemonic.to_string();
    let kdf_params = KdfParams::new_default();
    let pdk = derive_key(&phrase, &kdf_params)?;
    let encrypted_umk = encrypt_umk_blob(&pdk, &umk)?;
    let request = UmkRequest {
        encrypted_umk,
        kdf_params: serde_json::to_value(&kdf_params)?,
        version: 1,
    };
    client.put_umk(&request).await?;
    auth::store_phrase(profile, &phrase)?;
    auth::store_umk(profile, &encode_key(&umk))?;

    let keypair = derive_keypair_from_umk(&umk)?;
    client
        .put_user_key(&keypair.public_key_b64, "x25519-hkdf-xchacha20poly1305")
        .await?;

    for (repo_id, repo_key) in repo_keys {
        let wrapped = keys::wrap_repo_key_for_user(&keypair.public_key_b64, &repo_key).await?;
        client
            .put_repo_key(&repo_id, &wrapped, "x25519-hkdf-xchacha20poly1305", None)
            .await?;
    }

    println!(
        "{}",
        style::paint(
            style::YELLOW,
            "New recovery phrase (save this for new devices):"
        )
    );
    println!("{}", style::bold(style::LAVENDER, &phrase));
    println!("{}", style::paint(style::GREEN, "Rotated keys successfully."));
    Ok(())
}

pub async fn sessions(profile: &str) -> Result<()> {
    crate::commands::print_scope_user(profile);
    let config = Config::load()?;
    let base_url = config.base_url_for(profile)?;
    let token = auth::load_auth_token(profile)?;
    let client = ApiClient::new(&base_url, Some(token))?;
    let sessions = client.get_sessions().await?;
    if sessions.is_empty() {
        println!("{}", style::paint(style::SUBTEXT1, "no active sessions"));
        return Ok(());
    }

    let mut host_width = "Host".len();
    let mut active_width = "Active".len();
    let mut created_width = "Created".len();
    let mut expires_width = "Expires".len();
    for session in &sessions {
        host_width = host_width.max(session.host.len());
        active_width = active_width.max(if session.active { 3 } else { 2 });
        created_width = created_width.max(session.created_at.len());
        expires_width = expires_width.max(session.expires_at.len());
    }

    println!(
        "{}  {}  {}  {}",
        style::bold(style::MAUVE, &format!("{:<width$}", "Host", width = host_width)),
        style::bold(style::MAUVE, &format!("{:<width$}", "Active", width = active_width)),
        style::bold(style::MAUVE, &format!("{:<width$}", "Created", width = created_width)),
        style::bold(style::MAUVE, &format!("{:<width$}", "Expires", width = expires_width)),
    );
    println!(
        "{}  {}  {}  {}",
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = host_width)),
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = active_width)),
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = created_width)),
        style::paint(style::SUBTEXT1, &format!("{:-<width$}", "", width = expires_width)),
    );
    for session in sessions {
        let active = if session.active { "yes" } else { "no" };
        println!(
            "{}  {}  {}  {}",
            style::paint(style::TEXT, &format!("{:<width$}", session.host, width = host_width)),
            style::paint(style::TEXT, &format!("{:<width$}", active, width = active_width)),
            style::paint(
                style::SUBTEXT1,
                &format!("{:<width$}", session.created_at, width = created_width)
            ),
            style::paint(
                style::SUBTEXT1,
                &format!("{:<width$}", session.expires_at, width = expires_width)
            ),
        );
    }
    Ok(())
}

pub fn doctor(profile: &str) -> Result<()> {
    crate::commands::print_scope_user(profile);
    let has_session = auth::load_user_id(profile).is_ok();
    if has_session {
        match auth::load_umk(profile) {
            Ok(_) => println!("{}", style::paint(style::GREEN, "umk: ok")),
            Err(_) => println!(
                "{}",
                style::paint(style::YELLOW, "umk: missing (run `milieu login`)")
            ),
        }
        println!("{}", style::paint(style::GREEN, "keychain: ok"));
    } else {
        println!(
            "{}",
            style::paint(style::YELLOW, "auth: missing (run `milieu login`)")
        );
    }
    Ok(())
}

pub fn phrase_show(profile: &str) -> Result<()> {
    crate::commands::print_scope_user(profile);
    match auth::load_phrase(profile)? {
        Some(phrase) => {
            println!(
                "{}",
                style::paint(
                    style::YELLOW,
                    "Recovery phrase (store this somewhere safe):"
                )
            );
            println!("{}", style::bold(style::LAVENDER, &phrase));
            Ok(())
        }
        None => Err(crate::error::MilieuError::CommandFailed(
            "no recovery phrase found in keychain".to_string(),
        )),
    }
}

pub fn phrase_status(profile: &str) -> Result<()> {
    crate::commands::print_scope_user(profile);
    let exists = auth::load_phrase(profile)?.is_some();
    if exists {
        println!("{}", style::paint(style::GREEN, "phrase: present"));
    } else {
        println!("{}", style::paint(style::YELLOW, "phrase: missing"));
    }
    Ok(())
}

pub fn warn_login_key_age(user_key_updated_at: Option<&str>) {
    warn_if_old(user_key_updated_at);
}
