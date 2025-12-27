use crate::api::{ApiClient, LoginRequest, UmkRequest};
use crate::auth;
use crate::commands::{prompt, prompt_password};
use crate::config::Config;
use crate::crypto::{decrypt_umk_blob, derive_key, encrypt_umk_blob, generate_umk, KdfParams};
use crate::error::Result;
use crate::keys;
use crate::style;
use bip39::{Language, Mnemonic};
use rand_core::{OsRng, RngCore};
use tracing::debug;

pub async fn run(profile_override: Option<String>) -> Result<()> {
    crate::commands::print_scope_user(profile_override.as_deref().unwrap_or("default"));
    let mut config = Config::load()?;
    let base_url = config.base_url_for(profile_override.as_deref().unwrap_or("default"))?;

    let email = prompt("Email: ")?;
    let password = prompt_password("Password: ")?;
    let normalized_email = email.trim().to_lowercase();
    let profile = profile_override.unwrap_or_else(|| normalized_email.clone());

    let client = ApiClient::new(&base_url, None)?;
    let host = hostname::get()
        .ok()
        .and_then(|value| value.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string());

    let login = client
        .login(&LoginRequest {
            email: email.clone(),
            password,
            host,
        })
        .await?;

    auth::store_auth(&profile, &login.access_token, &login.user_id)?;
    auth::store_email(&profile, &normalized_email)?;

    let token_client = ApiClient::new(&base_url, Some(login.access_token))?;
    let umk = match token_client.get_umk().await? {
        None => {
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
            token_client.put_umk(&request).await?;
            auth::store_phrase(&profile, &phrase)?;

            println!(
                "{}",
                style::paint(
                    style::YELLOW,
                    "Recovery phrase (save this for new devices):"
                )
            );
            println!("{}", style::bold(style::LAVENDER, &phrase));
            umk
        }
        Some(response) => {
            let params: KdfParams = serde_json::from_value(response.kdf_params)?;
            let phrase = match auth::load_phrase(&profile)? {
                Some(value) => value,
                None => prompt_password("Recovery phrase: ")?,
            };
            let pdk = derive_key(&phrase, &params)?;
            match decrypt_umk_blob(&pdk, &response.encrypted_umk) {
                Ok(umk) => umk,
                Err(err) => {
                    debug!(error = ?err, "failed to decrypt UMK");
                    return Err(crate::error::MilieuError::CommandFailed(
                        "recovery phrase incorrect or does not match this account; run `milieu phrase show` to confirm, or `milieu logout` and login again with the correct phrase".to_string(),
                    ));
                }
            }
        }
    };

    let umk_b64 = crate::crypto::encode_key(&umk);
    auth::store_umk(&profile, &umk_b64)?;
    config.active_profile = profile.clone();
    config.set_base_url(&profile, base_url);
    config.save()?;
    let _ = keys::ensure_user_keypair(&profile, &token_client).await?;
    if let Ok(user_key) = token_client.get_user_key().await {
        let updated_at = user_key.as_ref().map(|key| key.updated_at.as_str());
        crate::commands::user::warn_login_key_age(updated_at);
    }

    let warning = login
        .warning
        .unwrap_or_else(|| "Beta testing: use at your own risk.".to_string());
    println!("{}", style::paint(style::YELLOW, &warning));

    println!(
        "{}",
        style::paint(
            style::GREEN,
            &format!("Logged in as {} ({})", email, login.user_id)
        )
    );
    Ok(())
}
