use crate::api::ApiClient;
use crate::auth;
use crate::config::Config;
use crate::crypto::{aad_for, decrypt_bytes};
use crate::error::{MilieuError, Result};
use crate::keys;
use crate::manifest::Manifest;
use crate::repo::manifest_path;
use crate::style;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub async fn run(
    profile: &str,
    path: String,
    version: u32,
    branch_override: Option<String>,
) -> Result<()> {
    let manifest = Manifest::load(&manifest_path()?)?;
    let branch_name = branch_override.unwrap_or_else(|| manifest.active_branch.clone());
    let branch = manifest.find_branch(&branch_name)?;
    crate::commands::print_scope_branch(&manifest, &branch_name);

    let entry = branch
        .files
        .iter()
        .find(|f| f.path() == path)
        .ok_or_else(|| MilieuError::CommandFailed("file not tracked".to_string()))?;

    let config = Config::load()?;
    let mut base_url = config.base_url_for(profile)?;
    if let Some(remote) = &manifest.remote {
        if let Some(url) = &remote.base_url {
            base_url = url.clone();
        }
    }

    let token = auth::load_auth_token(profile)?;
    let client = ApiClient::new(base_url, Some(token))?;
    let repo_key = keys::get_or_fetch_repo_key(profile, &client, &manifest.repo_id).await?;

    let remote = client
        .get_version(&manifest.repo_id, &branch_name, &path, version)
        .await?;

    let schema_version = remote.schema_version;
    let aad = aad_for(schema_version, &manifest.repo_id, &branch_name, &path, entry.tag());
    let plaintext = decrypt_bytes(&repo_key, &aad, &remote.nonce, &remote.ciphertext)?;
    write_secure(&path, &plaintext)?;

    println!(
        "{}",
        style::paint(style::GREEN, &format!("checked out {}@v{}", path, version))
    );
    Ok(())
}

fn write_secure(path: &str, data: &[u8]) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let mut file = fs::File::create(path)?;
    file.write_all(data)?;
    let mut perms = file.metadata()?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(path, perms)?;
    Ok(())
}
