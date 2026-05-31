//! # Gitcore: The Programmatic Engine for Git Identity Management
//!
//! Gitcore is a high-assurance Rust library designed to solve the complexity of
//! managing multiple Git identities on a single machine. It provides a
//! deterministic, secure, and automated way to isolate SSH keys, commit
//! authorship, and cryptographic signing across different environments.
//!
//! ## Core Philosophy
//! Traditional Git workflows rely on global configurations that lead to "identity
//! leakage" (e.g., using a personal email for a corporate commit). Gitcore
//! eliminates this by providing:
//! - **Cryptographic Isolation**: Every account is backed by its own unique Ed25519 keypair.
//! - **Automated Orchestration**: Seamless management of `~/.ssh/config` without touching existing manual entries.
//! - **Contextual Awareness**: Automatic detection and injection of identity metadata during `clone` or `remote` operations.
//!
//! ## Library vs. CLI
//! This crate serves as the underlying engine for the `gitcore` CLI. By exposing
//! this engine programmatically, Gitcore enables developers to:
//! - Build custom Git automation and CI/CD pipelines.
//! - Integrate identity management into IDEs or developer portals.
//! - Extend the core logic for specialized hosting providers.
//!
//! ## Getting Started
//! ```no_run
//! use gitcore::{Gitcore, AddAccountRequest, Platform};
//!
//! // Initialize the service with standard user paths
//! let service = Gitcore::new();
//!
//! // Provision and register a new identity
//! let request = AddAccountRequest {
//!     name: "work".to_string(),
//!     platform: Platform::Github,
//!     username: "octocat".to_string(),
//!     email: "octocat@example.com".to_string(),
//!     ..Default::default()
//! };
//!
//! service.register_account(request)?;
//! # Ok::<(), gitcore::GitcoreError>(())
//! ```

mod command_runner;
mod config;
mod error;
mod git;
mod gpg;
mod models;
mod service;
mod ssh;
mod vault;

pub use error::{GitcoreError, Result};
pub use gpg::{GpgKey, list_gpg_keys};
pub use models::{Account, GitcoreConfig, Platform};
pub use service::{
    AddAccountRequest, AuditReport, BackupReport, CloneReport, CloneRequest, FileAudit, Gitcore,
    GitcorePaths, KeyAudit, KeyDeletionReport, KeyProvisionReport, RegisteredAccount,
    RemoteAddRequest, RemoteReport, RemoteSwitchRequest, RestoreReport, RotationReport,
    SshTestReport, UpdateAccountRequest,
};
pub use ssh::{HostKeyStatus, delete_account_keys, generate_ssh_key, get_ssh_dir};
