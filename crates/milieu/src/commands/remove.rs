use crate::error::{MilieuError, Result};
use crate::manifest::Manifest;
use crate::repo::{manifest_path, validate_env_path};
use crate::style;

pub fn run(path: &str, branch_override: Option<String>) -> Result<()> {
    validate_env_path(path)?;
    let manifest_path = manifest_path()?;
    let mut manifest = Manifest::load(&manifest_path)?;
    let branch_name = branch_override.unwrap_or_else(|| manifest.active_branch.clone());
    crate::commands::print_scope_branch(&manifest, &branch_name);
    let branch = manifest.find_branch_mut(&branch_name)?;

    let before = branch.files.len();
    branch.files.retain(|entry| entry.path() != path);
    if branch.files.len() == before {
        return Err(MilieuError::CommandFailed(
            "file not tracked in branch".to_string(),
        ));
    }

    manifest.save(&manifest_path)?;
    println!(
        "{}",
        style::paint(style::PEACH, &format!("removed {} from {}", path, branch_name))
    );
    Ok(())
}
