use crate::error::{MilieuError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const DEFAULT_BASE_URL: &str = "https://milieu.sh";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub active_profile: String,
    pub profiles: HashMap<String, Profile>,
    #[serde(default = "default_history_limit")]
    pub history_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub base_url: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert(
            "default".to_string(),
            Profile {
                base_url: DEFAULT_BASE_URL.to_string(),
            },
        );
        Self {
            active_profile: "default".to_string(),
            profiles,
            history_limit: default_history_limit(),
        }
    }
}

fn default_history_limit() -> u32 {
    12
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if !path.exists() {
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }
        let contents = fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&contents)?;
        if config.profiles.is_empty() {
            config
                .profiles
                .insert("default".to_string(), Profile { base_url: DEFAULT_BASE_URL.to_string() });
            config.save()?;
        } else if !config.profiles.contains_key(&config.active_profile) {
            config
                .profiles
                .insert(config.active_profile.clone(), Profile { base_url: DEFAULT_BASE_URL.to_string() });
            config.save()?;
        }
        Ok(config)
    }

    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = toml::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn base_url_for(&self, profile: &str) -> Result<String> {
        let name = if profile.is_empty() {
            self.active_profile.as_str()
        } else {
            profile
        };
        if let Some(entry) = self.profiles.get(name) {
            return Ok(entry.base_url.clone());
        }
        if let Ok(value) = std::env::var("MILIEU_BASE_URL") {
            if !value.trim().is_empty() {
                return Ok(value);
            }
        }
        Ok(DEFAULT_BASE_URL.to_string())
    }

    #[allow(dead_code)]
    pub fn set_base_url(&mut self, profile: &str, base_url: String) {
        self.profiles
            .insert(profile.to_string(), Profile { base_url });
    }
}

pub fn config_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(dir).join("milieu"));
    }
    let home = std::env::var("HOME").map_err(|_| MilieuError::ConfigMissing)?;
    Ok(PathBuf::from(home).join(".config").join("milieu"))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}
