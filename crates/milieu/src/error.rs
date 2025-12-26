use thiserror::Error;

#[derive(Error, Debug)]
pub enum MilieuError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml decode error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("toml encode error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("keyring error: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("crypto error: {0}")]
    Crypto(String),


    #[error("repo not initialized; run `milieu init` or `milieu clone`")]
    RepoNotInitialized,

    #[error("branch not found: {0}")]
    BranchNotFound(String),

    #[error("missing config directory or HOME")]
    ConfigMissing,

    #[error("missing auth token; run `milieu login`")]
    AuthMissing,

    #[error("missing user id; run `milieu login`")]
    UserIdMissing,

    #[error("missing UMK; run `milieu login`")]
    UmkMissing,

    #[error("command failed: {0}")]
    CommandFailed(String),
}

pub type Result<T> = std::result::Result<T, MilieuError>;
