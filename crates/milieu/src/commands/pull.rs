use crate::api::ApiClient;
use crate::auth;
use crate::config::Config;
use crate::crypto::{aad_for, decrypt_bytes};
use crate::error::{MilieuError, Result};
use crate::keys;
use crate::manifest::{Branch, FileEntry, Manifest};
use crate::repo::{manifest_path, validate_env_path};
use crate::style;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub async fn run(profile: &str, branch_override: Option<String>) -> Result<()> {
    let manifest_path = manifest_path()?;
    let mut manifest = Manifest::load(&manifest_path)?;

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

    if let Ok(remote_manifest) = client.get_manifest(&manifest.repo_id).await {
        manifest = merge_manifests(&manifest, &remote_manifest);
        manifest.save(&manifest_path)?;
    }

    let branch_name = branch_override.unwrap_or_else(|| manifest.active_branch.clone());
    let repo_id = manifest.repo_id.clone();
    crate::commands::print_scope_branch(&manifest, &branch_name);
    let branch = manifest.find_branch_mut(&branch_name)?;
    let branch_label = branch.name.clone();

    for entry in &mut branch.files {
        let path = entry.path.clone();
        validate_env_path(&path)?;
        let response = match client
            .get_latest(&repo_id, &branch_label, &path)
            .await?
        {
            None => {
                println!(
                    "{}",
                    style::paint(style::YELLOW, &format!("missing remote for {}", path))
                );
                continue;
            }
            Some(value) => value,
        };

        let schema_version = response.schema_version;
        let aad = aad_for(schema_version, &repo_id, &branch_label, &path, entry.tag());
        let aad_b64 = B64.encode(&aad);
        if response.aad != aad_b64 {
            return Err(MilieuError::Crypto(format!(
                "aad mismatch for {}",
                path
            )));
        }
        let remote_plain = decrypt_bytes(&repo_key, &aad, &response.nonce, &response.ciphertext)?;
        let remote_hash = blake3::hash(&remote_plain);
        let local_plain = fs::read(&path).ok();
        let local_hash = local_plain.as_ref().map(|data| blake3::hash(data));
        let base_hash = entry
            .last_synced_hash
            .as_deref()
            .and_then(|hex| blake3::Hash::from_hex(hex).ok());

        match (local_plain, base_hash) {
            (None, _) => {
                write_secure(&path, &remote_plain)?;
                entry.set_synced(remote_hash.to_hex().to_string(), response.version);
                println!(
                    "{}",
                    style::paint(style::GREEN, &format!("pulled {}", path))
                );
            }
            (Some(local_bytes), None) => {
                if local_hash == Some(remote_hash) {
                    entry.set_synced(remote_hash.to_hex().to_string(), response.version);
                    println!(
                        "{}",
                        style::paint(style::GREEN, &format!("up to date {}", path))
                    );
                } else {
                entry.set_synced(remote_hash.to_hex().to_string(), response.version);
                let merged = format!(
                    "<<<<<<< local\n{}\n=======\n{}\n>>>>>>> remote\n",
                    String::from_utf8_lossy(&local_bytes),
                    String::from_utf8_lossy(&remote_plain)
                );
                write_secure(&path, merged.as_bytes())?;
                    println!(
                        "{}",
                        style::paint(
                            style::RED,
                            &format!("merge conflict in {}; resolve then push", path)
                        )
                    );
                }
            }
            (Some(local_bytes), Some(base)) => {
                let local_hash = local_hash.unwrap_or_else(|| blake3::hash(&local_bytes));
                if local_hash == base {
                    write_secure(&path, &remote_plain)?;
                    entry.set_synced(remote_hash.to_hex().to_string(), response.version);
                    println!(
                        "{}",
                        style::paint(style::GREEN, &format!("pulled {}", path))
                    );
                } else if remote_hash == base {
                    entry.last_synced_version = response.version;
                    println!(
                        "{}",
                        style::paint(style::GREEN, &format!("kept local {}", path))
                    );
                } else if local_hash == remote_hash {
                    entry.set_synced(remote_hash.to_hex().to_string(), response.version);
                    println!(
                        "{}",
                        style::paint(style::GREEN, &format!("up to date {}", path))
                    );
                } else {
                    entry.set_synced(remote_hash.to_hex().to_string(), response.version);
                    let merged = format!(
                        "<<<<<<< local\n{}\n=======\n{}\n>>>>>>> remote\n",
                        String::from_utf8_lossy(&local_bytes),
                        String::from_utf8_lossy(&remote_plain)
                    );
                    write_secure(&path, merged.as_bytes())?;
                    println!(
                        "{}",
                        style::paint(
                            style::RED,
                            &format!("merge conflict in {}; resolve then push", path)
                        )
                    );
                }
            }
        }
    }

    manifest.save(&manifest_path)?;
    Ok(())
}

fn merge_manifests(local: &Manifest, remote: &Manifest) -> Manifest {
    let mut branches: HashMap<String, Branch> = HashMap::new();

    for branch in &remote.branches {
        branches.insert(branch.name.clone(), branch.clone());
    }

    for branch in &local.branches {
        let entry = branches.entry(branch.name.clone()).or_insert_with(|| branch.clone());
        let mut existing: HashMap<String, FileEntry> = entry
            .files
            .iter()
            .cloned()
            .map(|file| (file.path.clone(), file))
            .collect();
        for file in &branch.files {
            existing.insert(file.path.clone(), file.clone());
        }
        entry.files = existing.into_values().collect();
    }

    let mut merged = Manifest {
        version: remote.version,
        repo_id: remote.repo_id.clone(),
        repo_name: remote.repo_name.clone(),
        active_branch: remote.active_branch.clone(),
        branches: branches.into_values().collect(),
        remote: local.remote.clone(),
    };

    if merged.branches.is_empty() {
        merged.branches = local.branches.clone();
    }

    merged
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

// path validation centralized in repo::validate_env_path
