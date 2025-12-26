use crate::error::Result;
use crate::manifest::Manifest;
use crate::repo::{manifest_path, validate_env_path};
use crate::style;

pub fn run(path: &str, tag: Option<String>, branch_override: Option<String>) -> Result<()> {
    validate_env_path(path)?;
    let manifest_path = manifest_path()?;
    let mut manifest = Manifest::load(&manifest_path)?;

    let branch_name = branch_override.unwrap_or_else(|| manifest.active_branch.clone());
    crate::commands::print_scope_branch(&manifest, &branch_name);
    let branch = manifest.find_branch_mut(&branch_name)?;

    if branch.files.iter().any(|entry| entry.path() == path) {
        println!(
            "{}",
            style::paint(style::YELLOW, &format!("already tracked: {}", path))
        );
        return Ok(());
    }

    branch.files.push(crate::manifest::FileEntry::new(
        path.to_string(),
        tag,
    ));

    manifest.save(&manifest_path)?;
    println!(
        "{}",
        style::paint(style::GREEN, &format!("added {} to {} (+tracked)", path, branch_name))
    );
    Ok(())
}

// path validation centralized in repo::validate_env_path
