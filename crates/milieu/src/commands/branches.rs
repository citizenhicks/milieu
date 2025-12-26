use crate::api::ApiClient;
use crate::auth;
use crate::config::Config;
use crate::error::{MilieuError, Result};
use crate::manifest::{Branch, FileEntry, Manifest};
use crate::repo::{manifest_path, validate_env_path};
use crate::style;

pub fn add(name: &str, files: Vec<String>, tags: Vec<String>) -> Result<()> {
    if files.is_empty() {
        return Err(MilieuError::CommandFailed("at least one --file is required".to_string()));
    }
    if !tags.is_empty() && tags.len() != files.len() {
        return Err(MilieuError::CommandFailed(
            "--tag count must match --file count".to_string(),
        ));
    }
    let path = manifest_path()?;
    let mut manifest = Manifest::load(&path)?;
    manifest.ensure_unique_branch(name)?;
    let mut entries = Vec::new();
    for (idx, file) in files.into_iter().enumerate() {
        validate_env_path(&file)?;
        let tag = tags.get(idx).cloned();
        entries.push(FileEntry::new(file, tag));
    }
    manifest.branches.push(Branch {
        name: name.to_string(),
        files: entries,
    });
    crate::commands::print_scope_repo(&manifest);
    manifest.save(&path)?;
    println!(
        "{}",
        style::paint(style::GREEN, &format!("added branch {}", name))
    );
    Ok(())
}

pub fn remove(name: &str) -> Result<()> {
    let path = manifest_path()?;
    let mut manifest = Manifest::load(&path)?;
    if manifest.active_branch == name {
        return Err(MilieuError::CommandFailed(
            "cannot remove the active branch".to_string(),
        ));
    }
    let before = manifest.branches.len();
    manifest.branches.retain(|s| s.name != name);
    if manifest.branches.len() == before {
        return Err(MilieuError::BranchNotFound(name.to_string()));
    }
    crate::commands::print_scope_repo(&manifest);
    manifest.save(&path)?;
    println!(
        "{}",
        style::paint(style::PEACH, &format!("removed branch {}", name))
    );
    Ok(())
}

pub fn set_default(name: &str) -> Result<()> {
    let path = manifest_path()?;
    let mut manifest = Manifest::load(&path)?;
    let _ = manifest.find_branch(name)?;
    manifest.active_branch = name.to_string();
    crate::commands::print_scope_repo(&manifest);
    manifest.save(&path)?;
    println!(
        "{}",
        style::paint(style::GREEN, &format!("active branch set to {}", name))
    );
    Ok(())
}

pub fn list() -> Result<()> {
    let path = manifest_path()?;
    let manifest = Manifest::load(&path)?;
    crate::commands::print_scope_repo(&manifest);

    println!("{}", style::bold(style::MAUVE, "branches:"));
    for branch in &manifest.branches {
        let label = if branch.name == manifest.active_branch {
            format!("* {}", branch.name)
        } else {
            format!("  {}", branch.name)
        };
        println!("{}", style::paint(style::TEXT, &label));
    }
    Ok(())
}

pub async fn sync(profile: &str) -> Result<()> {
    let path = manifest_path()?;
    let manifest = Manifest::load(&path)?;
    let config = Config::load()?;
    let base_url = config.base_url_for(profile)?;
    let token = auth::load_auth_token(profile)?;
    let client = ApiClient::new(base_url, Some(token))?;
    client.put_manifest(&manifest).await?;
    Ok(())
}

pub async fn add_and_sync(profile: &str, name: &str, files: Vec<String>, tags: Vec<String>) -> Result<()> {
    add(name, files, tags)?;
    sync(profile).await
}

pub async fn remove_and_sync(profile: &str, name: &str) -> Result<()> {
    remove(name)?;
    sync(profile).await
}

pub async fn set_default_and_sync(profile: &str, name: &str) -> Result<()> {
    set_default(name)?;
    sync(profile).await
}

// path validation centralized in repo::validate_env_path
