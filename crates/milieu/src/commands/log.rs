use crate::api::ApiClient;
use crate::auth;
use crate::config::Config;
use crate::error::{MilieuError, Result};
use crate::manifest::Manifest;
use crate::repo::manifest_path;
use crate::style;

pub async fn run(profile: &str, path: String, branch_override: Option<String>) -> Result<()> {
    let manifest = Manifest::load(&manifest_path()?)?;
    let branch_name = branch_override.unwrap_or_else(|| manifest.active_branch.clone());
    let _branch = manifest.find_branch(&branch_name)?;
    crate::commands::print_scope_branch(&manifest, &branch_name);

    let config = Config::load()?;
    let mut base_url = config.base_url_for(profile)?;
    if let Some(remote) = &manifest.remote {
        if let Some(url) = &remote.base_url {
            base_url = url.clone();
        }
    }

    let token = auth::load_auth_token(profile)?;
    let client = ApiClient::new(&base_url, Some(token))?;

    let history = client.get_history(&manifest.repo_id, &branch_name, &path).await?;
    if history.is_empty() {
        return Err(MilieuError::CommandFailed("no history for file".to_string()));
    }

    println!(
        "{}",
        style::bold(style::MAUVE, &format!("history: {}", path))
    );
    println!(
        "{}  {}",
        style::bold(style::MAUVE, "version"),
        style::bold(style::MAUVE, "created_at")
    );
    println!(
        "{}  {}",
        style::paint(style::SUBTEXT1, "-------"),
        style::paint(style::SUBTEXT1, "----------")
    );

    for entry in history {
        println!(
            "{}  {}",
            style::paint(style::TEXT, &entry.version.to_string()),
            style::paint(style::SUBTEXT1, &entry.created_at)
        );
    }

    Ok(())
}
