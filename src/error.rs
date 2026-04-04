use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ShipError {
    #[error("Config error: {0}")]
    Config(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("Test failure: {0}")]
    TestFailed(String),

    #[error("Docs gate failure: {0}")]
    DocsGateFailed(String),

    #[error("Push failed: {0}")]
    PushFailed(String),

    #[error("PR creation failed: {0}")]
    PrFailed(String),

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("On protected branch: {0}")]
    ProtectedBranch(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ShipError>;
