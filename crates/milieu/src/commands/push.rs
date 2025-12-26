use crate::api::{ApiClient, ObjectRequest};
use crate::auth;
use crate::config::Config;
use crate::crypto::{aad_for, decode_key, decrypt_bytes, encrypt_bytes};
use crate::error::{MilieuError, Result};
use crate::manifest::Manifest;
use crate::repo::{manifest_path, validate_env_path};
use crate::style;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use similar::TextDiff;
use similar::ChangeTag;
use std::collections::HashSet;
use std::fs;

const MAX_REPO_BYTES: u64 = 1 * 1024 * 1024;

pub async fn run(profile: &str, branch_override: Option<String>) -> Result<()> {
    let manifest_path = manifest_path()?;
    let mut manifest = Manifest::load(&manifest_path)?;
    enforce_repo_size_limit(&manifest)?;

    let branch_name = branch_override.unwrap_or_else(|| manifest.active_branch.clone());
    let repo_id = manifest.repo_id.clone();
    let branch_snapshot = manifest.find_branch(&branch_name)?.clone();
    crate::commands::print_scope_branch(&manifest, &branch_name);

    let config = Config::load()?;
    let mut base_url = config.base_url_for(profile)?;
    if let Some(remote) = &manifest.remote {
        if let Some(url) = &remote.base_url {
            base_url = url.clone();
        }
    }

    let token = auth::load_auth_token(profile)?;
    let umk_b64 = auth::load_umk(profile)?;
    let umk = decode_key(&umk_b64)?;

    let client = ApiClient::new(base_url, Some(token))?;

    let mut conflicts = Vec::new();
    for entry in &branch_snapshot.files {
        let path = entry.path();
        validate_env_path(path)?;
        let data = fs::read(&path).map_err(|_| {
            MilieuError::CommandFailed(format!("missing file: {}", path))
        })?;
        let local_hash = blake3::hash(&data);

        let remote = client
            .get_latest(&repo_id, &branch_snapshot.name, path)
            .await?;

        if let Some(remote_obj) = &remote {
            let aad = aad_for(1, &repo_id, &branch_snapshot.name, path, entry.tag());
            let plaintext = decrypt_bytes(&umk, &aad, &remote_obj.nonce, &remote_obj.ciphertext)
                .map_err(|_| {
                    MilieuError::CommandFailed(format!(
                        "failed to decrypt remote for {}; run `milieu pull`",
                        path
                    ))
                })?;
            let remote_hash = blake3::hash(&plaintext);
            let base_hash = entry
                .last_synced_hash
                .as_deref()
                .and_then(|hex| blake3::Hash::from_hex(hex).ok());

            match base_hash {
                None => {
                    if local_hash != remote_hash {
                        conflicts.push(path.to_string());
                    }
                }
                Some(base) => {
                    if remote_hash != base {
                        conflicts.push(path.to_string());
                    }
                }
            }
        }
    }

    if !conflicts.is_empty() {
        let mut message = String::from("remote has new changes; run `milieu pull` first:");
        for path in conflicts {
            message.push_str(&format!("\n  - {}", path));
        }
        return Err(MilieuError::CommandFailed(message));
    }

    client.put_manifest(&manifest).await?;

    let branch = manifest.find_branch_mut(&branch_name)?;
    let branch_label = branch.name.clone();
    for entry in &mut branch.files {
        let path = entry.path.clone();
        validate_env_path(&path)?;
        let data = fs::read(&path).map_err(|_| {
            MilieuError::CommandFailed(format!("missing file: {}", path))
        })?;

        let remote = client
            .get_latest(&repo_id, &branch_label, &path)
            .await?;

        let (adds, dels, same_as_remote, remote_hash, remote_version) = match remote {
            Some(ref obj) => {
                let aad = aad_for(1, &repo_id, &branch_label, &path, entry.tag());
                let plaintext = decrypt_bytes(&umk, &aad, &obj.nonce, &obj.ciphertext)?;
                let remote_hash = blake3::hash(&plaintext);
                let remote_text = String::from_utf8_lossy(&plaintext);
                let local_text = String::from_utf8_lossy(&data);
                let (adds, dels) = diff_stats(&remote_text, &local_text);
                (adds, dels, adds == 0 && dels == 0, Some(remote_hash), obj.version)
            }
            None => {
                let local_text = String::from_utf8_lossy(&data);
                (local_text.lines().count() as i64, 0, false, None, None)
            }
        };

        if same_as_remote {
            if let Some(remote_hash) = remote_hash {
                entry.set_synced(remote_hash.to_hex().to_string(), remote_version);
            }
            println!(
                "{}",
                style::paint(style::SUBTEXT1, &format!("unchanged {}", path))
            );
            continue;
        }

        let aad = aad_for(1, &repo_id, &branch_label, &path, entry.tag());
        let (nonce, ciphertext) = encrypt_bytes(&umk, &aad, &data)?;
        let ciphertext_hash = blake3::hash(ciphertext.as_bytes()).to_hex().to_string();
        let request = ObjectRequest {
            path: path.to_string(),
            nonce,
            ciphertext,
            aad: B64.encode(aad),
            ciphertext_hash,
            created_at: chrono::Utc::now().to_rfc3339(),
            schema_version: 1,
        };
        let response = client
            .post_object(&repo_id, &branch_label, &request)
            .await?;

        let summary = format!("+{} -{}", adds, dels);
        println!(
            "{}",
            style::paint(style::GREEN, &format!("pushed {} ({})", path, summary))
        );

        entry.set_synced(
            blake3::hash(&data).to_hex().to_string(),
            response.version,
        );
    }

    manifest.save(&manifest_path)?;
    Ok(())
}

fn diff_stats(old_text: &str, new_text: &str) -> (i64, i64) {
    let diff = TextDiff::from_lines(old_text, new_text);
    let mut adds = 0;
    let mut dels = 0;
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Insert => adds += 1,
            ChangeTag::Delete => dels += 1,
            ChangeTag::Equal => {}
        }
    }
    (adds, dels)
}

// path validation centralized in repo::validate_env_path

fn enforce_repo_size_limit(manifest: &Manifest) -> Result<()> {
    let mut seen = HashSet::new();
    let mut total: u64 = 0;

    for branch in &manifest.branches {
        for entry in &branch.files {
            let path = entry.path();
            if !seen.insert(path.to_string()) {
                continue;
            }
            validate_env_path(path)?;
            if let Ok(meta) = fs::metadata(path) {
                total += meta.len();
            }
        }
    }

    if total > MAX_REPO_BYTES {
        return Err(MilieuError::CommandFailed(format!(
            "repo size {} bytes exceeds 1MB cap",
            total
        )));
    }

    Ok(())
}
