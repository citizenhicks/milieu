use crate::api::ApiClient;
use crate::auth;
use crate::config::Config;
use crate::crypto::{aad_for, decrypt_bytes};
use crate::error::{MilieuError, Result};
use crate::keys;
use crate::manifest::Manifest;
use crate::repo::{manifest_path, validate_env_path};
use crate::style;
use similar::TextDiff;
use std::fs;

pub async fn run(
    profile: &str,
    path: Option<String>,
    branch_override: Option<String>,
    version: Option<u32>,
) -> Result<()> {
    let manifest = Manifest::load(&manifest_path()?)?;
    let branch_name = branch_override.unwrap_or_else(|| manifest.active_branch.clone());
    let branch = manifest.find_branch(&branch_name)?;
    crate::commands::print_scope_branch(&manifest, &branch_name);

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

    let entries: Vec<_> = match path {
        Some(target) => branch
            .files
            .iter()
            .filter(|entry| entry.path() == target)
            .collect(),
        None => branch.files.iter().collect(),
    };

    if entries.is_empty() {
        return Err(MilieuError::CommandFailed("no matching files in branch".to_string()));
    }

    let mut printed = false;
    let mut header_printed = false;

    for entry in entries {
        let file_path = entry.path();
        validate_env_path(file_path)?;

        let local_text = fs::read_to_string(file_path).ok();
        let remote_obj = match version {
            Some(ver) => client
                .get_version(&manifest.repo_id, &branch.name, file_path, ver)
                .await
                .map(Some)?,
            None => client
                .get_latest(&manifest.repo_id, &branch.name, file_path)
                .await?,
        };

        let remote_text = match remote_obj {
            Some(ref obj) => {
                let schema_version = obj.schema_version;
                let aad = aad_for(schema_version, &manifest.repo_id, &branch.name, file_path, entry.tag());
                let plaintext = decrypt_bytes(&repo_key, &aad, &obj.nonce, &obj.ciphertext)?;
                Some(String::from_utf8_lossy(&plaintext).to_string())
            }
            None => None,
        };

        if local_text.is_none() && remote_text.is_none() {
            continue;
        }

        println!(
            "{}",
            style::bold(style::MAUVE, &format!("FILE: {}", file_path))
        );

        let remote_body = remote_text.as_deref().unwrap_or("");
        let local_body = local_text.as_deref().unwrap_or("");

        let diff = TextDiff::from_lines(remote_body, local_body);
        if diff.ratio() == 1.0 {
            println!("{}", style::paint(style::GREEN, "NO DIFF"));
            printed = true;
            continue;
        }

        if !header_printed {
            println!("{}", style::paint(style::SUBTEXT1, "--- remote"));
            println!("{}", style::paint(style::SUBTEXT1, "+++ local"));
            header_printed = true;
        }

        for change in diff.iter_all_changes() {
            let line = change.to_string();
            match change.tag() {
                similar::ChangeTag::Insert => {
                    print_colored_diff_line('+', line.as_str());
                }
                similar::ChangeTag::Delete => {
                    print_colored_diff_line('-', line.as_str());
                }
                similar::ChangeTag::Equal => {
                    print_colored_diff_line(' ', line.as_str());
                }
            }
        }

        printed = true;
    }

    if !printed {
        println!("{}", style::paint(style::SUBTEXT1, "no diffs to show"));
    }

    Ok(())
}

// path validation centralized in repo::validate_env_path

fn print_colored_diff_line(prefix: char, line: &str) {
    let content = line.trim_end_matches('\n');
    match prefix {
        '+' => println!("{}", style::paint(style::GREEN, &format!("+{}", content))),
        '-' => println!("{}", style::paint(style::RED, &format!("-{}", content))),
        ' ' => println!("{}", style::paint(style::SUBTEXT1, &format!(" {}", content))),
        _ => println!("{}", style::paint(style::SUBTEXT1, content)),
    }
}
