use crate::error::{MilieuError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub repo_id: String,
    pub repo_name: String,
    pub active_branch: String,
    #[serde(rename = "branch")]
    pub branches: Vec<Branch>,

    #[serde(default)]
    pub remote: Option<Remote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remote {
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub last_synced_hash: Option<String>,
    #[serde(default)]
    pub last_synced_version: Option<u32>,
}

impl FileEntry {
    pub fn new(path: String, tag: Option<String>) -> Self {
        Self {
            path,
            tag,
            last_synced_hash: None,
            last_synced_version: None,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    pub fn set_synced(&mut self, hash: String, version: Option<u32>) {
        self.last_synced_hash = Some(hash);
        self.last_synced_version = version;
    }
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(MilieuError::RepoNotInitialized);
        }
        let contents = fs::read_to_string(path)?;
        let manifest = toml::from_str(&contents)?;
        Ok(manifest)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = toml::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn find_branch(&self, name: &str) -> Result<&Branch> {
        self.branches
            .iter()
            .find(|branch| branch.name == name)
            .ok_or_else(|| MilieuError::BranchNotFound(name.to_string()))
    }

    pub fn find_branch_mut(&mut self, name: &str) -> Result<&mut Branch> {
        self.branches
            .iter_mut()
            .find(|branch| branch.name == name)
            .ok_or_else(|| MilieuError::BranchNotFound(name.to_string()))
    }

    pub fn ensure_unique_branch(&self, name: &str) -> Result<()> {
        if self.branches.iter().any(|s| s.name == name) {
            return Err(MilieuError::CommandFailed(format!(
                "branch already exists: {}",
                name
            )));
        }
        Ok(())
    }

    pub fn without_state(&self) -> Self {
        let mut cloned = self.clone();
        for branch in &mut cloned.branches {
            for file in &mut branch.files {
                file.last_synced_hash = None;
                file.last_synced_version = None;
            }
        }
        cloned
    }
}
