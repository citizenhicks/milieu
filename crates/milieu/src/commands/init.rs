use crate::api::ApiClient;
use crate::auth;
use crate::config::Config;
use crate::crypto;
use crate::error::{MilieuError, Result};
use crate::keys;
use crate::manifest::{Branch, Manifest, Remote};
use crate::repo::{folder_name, is_valid_repo_name, manifest_path, milieu_dir};
use crate::style;
use std::fs;

pub async fn run(profile: &str, name_override: Option<String>) -> Result<()> {
    let manifest_path = manifest_path()?;
    if manifest_path.exists() {
        println!(
            "{}",
            style::paint(
                style::YELLOW,
                &format!("manifest already exists at {}", manifest_path.display())
            )
        );
        return Ok(());
    }

    let repo_name = match name_override {
        Some(name) => name,
        None => folder_name()?,
    };

    if !is_valid_repo_name(&repo_name) {
        return Err(MilieuError::CommandFailed(
            "repo name must be letters, numbers, '-' or '_' (use --name)".to_string(),
        ));
    }

    let config = Config::load()?;
    let base_url = config.base_url_for(profile)?;
    let token = auth::load_auth_token(profile)?;
    let client = ApiClient::new(base_url, Some(token))?;

    let repo = client.create_repo(&repo_name).await?;

    let milieu_dir = milieu_dir()?;
    fs::create_dir_all(&milieu_dir)?;

    let manifest = Manifest {
        version: 1,
        repo_id: repo.repo_id,
        repo_name: repo.name,
        active_branch: "dev".to_string(),
        branches: vec![Branch {
            name: "dev".to_string(),
            files: Vec::new(),
        }],
        remote: Some(Remote { base_url: None }),
    };

    let keypair = keys::ensure_user_keypair(profile, &client).await?;
    let repo_key = crypto::generate_umk();
    let wrapped =
        keys::wrap_repo_key_for_user(&keypair.public_key_b64, &repo_key).await?;
    client
        .put_repo_key(
            &manifest.repo_id,
            &wrapped,
            "x25519-hkdf-xchacha20poly1305",
            None,
        )
        .await?;
    keys::store_repo_key(profile, &manifest.repo_id, &repo_key)?;

    crate::commands::print_scope_repo(&manifest);
    manifest.save(&manifest_path)?;
    client.put_manifest(&manifest).await?;
    println!(
        "{}",
        style::paint(
            style::GREEN,
            &format!("initialized Milieu repo '{}'", manifest.repo_name)
        )
    );
    Ok(())
}
