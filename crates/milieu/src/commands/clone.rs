use crate::api::ApiClient;
use crate::auth;
use crate::config::Config;
use crate::error::{MilieuError, Result};
use crate::manifest::{Manifest, Remote};
use crate::repo::{folder_name, manifest_path, milieu_dir};
use crate::style;
use std::fs;

pub async fn run(profile: &str, repo_name: Option<String>) -> Result<()> {
    let manifest_path = manifest_path()?;
    if manifest_path.exists() {
        return Err(MilieuError::CommandFailed(
            "manifest already exists; remove .milieu to re-clone".to_string(),
        ));
    }

    let name = match repo_name {
        Some(value) => value,
        None => folder_name()?,
    };

    let config = Config::load()?;
    let base_url = config.base_url_for(profile)?;
    let token = auth::load_auth_token(profile)?;
    let client = ApiClient::new(base_url, Some(token))?;

    let repo = client.get_repo_by_name(&name).await?;
    let manifest = client.get_manifest(&repo.repo_id).await?;

    let milieu_dir = milieu_dir()?;
    fs::create_dir_all(&milieu_dir)?;

    let manifest = Manifest { remote: Some(Remote { base_url: None }), ..manifest };

    crate::commands::print_scope_repo(&manifest);
    manifest.save(&manifest_path)?;
    println!(
        "{}",
        style::paint(style::GREEN, &format!("cloned repo '{}'", manifest.repo_name))
    );
    Ok(())
}
