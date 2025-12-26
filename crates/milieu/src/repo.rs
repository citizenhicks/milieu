use crate::error::{MilieuError, Result};
use std::path::{Component, Path, PathBuf};

pub fn project_root() -> Result<PathBuf> {
    std::env::current_dir().map_err(|e| MilieuError::Io(e))
}

pub fn manifest_path() -> Result<PathBuf> {
    Ok(project_root()?.join(".milieu").join("manifest.toml"))
}

pub fn milieu_dir() -> Result<PathBuf> {
    Ok(project_root()?.join(".milieu"))
}

pub fn folder_name() -> Result<String> {
    let root = project_root()?;
    let name = root
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| MilieuError::CommandFailed("invalid folder name".to_string()))?;
    Ok(name.to_string())
}

pub fn is_valid_repo_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

pub fn validate_env_path(path: &str) -> Result<()> {
    let candidate = Path::new(path);
    if candidate.as_os_str().is_empty() {
        return Err(MilieuError::CommandFailed("invalid file path".to_string()));
    }
    if candidate.is_absolute() {
        return Err(MilieuError::CommandFailed(
            "only repo-relative .env* paths are allowed".to_string(),
        ));
    }
    for component in candidate.components() {
        match component {
            Component::ParentDir => {
                return Err(MilieuError::CommandFailed(
                    "path cannot contain '..'".to_string(),
                ));
            }
            Component::Prefix(_) | Component::RootDir => {
                return Err(MilieuError::CommandFailed(
                    "only repo-relative .env* paths are allowed".to_string(),
                ));
            }
            Component::CurDir | Component::Normal(_) => {}
        }
    }
    let filename = candidate
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| MilieuError::CommandFailed("invalid file path".to_string()))?;
    if filename == ".env" || filename.starts_with(".env.") {
        return Ok(());
    }
    Err(MilieuError::CommandFailed(
        "only .env* files are allowed".to_string(),
    ))
}
