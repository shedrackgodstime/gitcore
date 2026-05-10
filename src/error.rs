use crate::models::Platform;
use std::path::PathBuf;
use thiserror::Error;

/// Standard result type for Gitcore library operations.
pub type Result<T> = std::result::Result<T, GitcoreError>;

/// Typed error surface for library consumers.
#[derive(Debug, Error)]
pub enum GitcoreError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("invalid account name '{0}'")]
    InvalidAccountName(String),

    #[error("account '{0}' already exists")]
    DuplicateAccountName(String),

    #[error("host alias '{0}' already exists")]
    DuplicateHostAlias(String),

    #[error("unsupported platform '{0}'")]
    UnsupportedPlatform(String),

    #[error("account '{0}' was not found")]
    AccountNotFound(String),

    #[error("username cannot be empty")]
    EmptyUsername,

    #[error("email cannot be empty")]
    EmptyEmail,

    #[error("password cannot be empty")]
    EmptyPassword,

    #[error("path is not a git repository: {0} (Did you forget to run 'git init'?)")]
    NotGitRepository(PathBuf),

    #[error("origin remote could not be read for repository: {0}")]
    MissingOriginRemote(PathBuf),

    #[error("platform {0:?} is missing a provider URL")]
    MissingProviderUrl(Platform),
}
