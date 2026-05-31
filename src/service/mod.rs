use crate::command_runner::{CommandRunner, SystemCommandRunner};
use crate::config;
use crate::error::{GitcoreError, Result};
use crate::git;
use crate::models::{
    Account, GitcoreConfig, Platform, Vault, VaultKey, is_valid_account_name, validate_accounts,
};
use crate::ssh::{self, HostKeyStatus};
use crate::vault::{decrypt_vault, encrypt_vault};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::sync::Arc;

mod accounts;
mod audit;
mod backup;
mod repos;
mod ssh_ops;
mod storage;
#[cfg(test)]
mod tests;

/// Filesystem locations used by the Gitcore library.
#[derive(Debug, Clone)]
pub struct GitcorePaths {
    /// Location of the persisted Gitcore JSON configuration.
    pub config_path: PathBuf,
    /// Directory containing managed SSH keys and the generated SSH config block.
    pub ssh_dir: PathBuf,
}

impl Default for GitcorePaths {
    fn default() -> Self {
        Self {
            config_path: config::default_config_path(),
            ssh_dir: ssh::get_ssh_dir(),
        }
    }
}

/// High-level service facade for Gitcore operations.
///
/// This is the primary entry point for the library. It manages the lifecycle of
/// Git accounts, SSH configuration blocks, and repository-level identity injection.
///
/// Most users should initialize this using [`Gitcore::new()`] for default user paths,
/// or [`Gitcore::with_paths()`] for specialized environments.
#[derive(Clone)]
pub struct Gitcore {
    paths: GitcorePaths,
    runner: Arc<dyn CommandRunner>,
}

/// Input required to register a new Gitcore account.
#[derive(Debug, Clone, Default)]
pub struct AddAccountRequest {
    /// Human-facing account name such as `work` or `personal`.
    pub name: String,
    /// Git hosting platform this identity belongs to.
    pub platform: Platform,
    /// Git author name to write into repository config.
    pub username: String,
    /// Email address used for commits and SSH key comments.
    pub email: String,
    /// Optional GPG signing key id to attach to cloned or switched repositories.
    pub gpg_key_id: Option<String>,
    /// Optional custom filename for the SSH key. Defaults to id_ed25519_{name}.
    pub key_path: Option<String>,
}

/// Input required to update an existing Gitcore account.
#[derive(Debug, Clone, Default)]
pub struct UpdateAccountRequest {
    /// New author name.
    pub username: Option<String>,
    /// New email address.
    pub email: Option<String>,
    /// New GPG signing key ID.
    pub gpg_key_id: Option<Option<String>>,
}

/// Normalized account details produced during registration.
#[derive(Debug, Clone)]
pub struct RegisteredAccount {
    /// The persisted account record.
    pub account: Account,
    /// Generated host alias inserted into the managed SSH config block.
    pub host_alias: String,
}

/// Permission status for an audited file.
#[derive(Debug, Clone)]
pub struct FileAudit {
    /// Absolute path of the audited file.
    pub path: PathBuf,
    /// Whether the file currently exists.
    pub exists: bool,
    /// Effective filesystem permissions when available on the current platform.
    pub permissions: Option<u32>,
    /// Permission mode Gitcore expects for the file.
    pub expected_permissions: u32,
}

/// Audit result for a managed SSH keypair.
#[derive(Debug, Clone)]
pub struct KeyAudit {
    /// Account that owns the keypair.
    pub account: Account,
    /// Audit information for the private key.
    pub private_key: FileAudit,
    /// Audit information for the public key.
    pub public_key: FileAudit,
}

/// Security audit report for a Gitcore installation.
#[derive(Debug, Clone)]
pub struct AuditReport {
    /// Audit rows for each configured account keypair.
    pub key_audits: Vec<KeyAudit>,
    /// Audit information for the generated SSH config file.
    pub ssh_config: FileAudit,
    /// Audit information for the persisted Gitcore config file.
    pub config_file: FileAudit,
    /// Human-readable issues detected during the audit pass.
    pub issues: Vec<String>,
}

/// Summary of a created encrypted backup.
#[derive(Debug, Clone)]
pub struct BackupReport {
    /// Path where the encrypted backup was written.
    pub output_path: PathBuf,
    /// Key files successfully embedded into the backup.
    pub included_keys: Vec<String>,
    /// Configured key files that were missing at backup time.
    pub missing_keys: Vec<String>,
}

/// Outcome of restoring persisted state from a vault or JSON file.
#[derive(Debug, Clone)]
pub struct RestoreReport {
    /// Number of accounts restored into config.
    pub restored_accounts: usize,
    /// Restored SSH private key filenames.
    pub restored_keys: Vec<String>,
    /// Input backup path used for the restore.
    pub source_path: PathBuf,
    /// Whether the restore originated from legacy JSON instead of an encrypted vault.
    pub legacy_json: bool,
}

/// Input required to clone or attach a repository for an account.
#[derive(Debug, Clone)]
pub struct CloneRequest {
    /// Account selector, usually an account name.
    pub account_name: String,
    /// Source repository URL in HTTPS, SSH, or provider/path form.
    pub repo_url: String,
    /// Parent directory where the repository should be cloned or discovered.
    pub working_dir: PathBuf,
}

/// Outcome of an identity-aware clone operation.
#[derive(Debug, Clone)]
pub struct CloneReport {
    /// Resolved local repository path.
    pub repo_path: PathBuf,
    /// Final remote URL after host alias rewriting.
    pub remote_url: String,
    /// Git username applied to the repository config.
    pub username: String,
    /// Git email applied to the repository config.
    pub email: String,
    /// Whether Gitcore reused an existing checkout instead of cloning anew.
    pub reused_existing_repo: bool,
}

/// Input required to add or replace the origin remote for a repository.
#[derive(Debug, Clone)]
pub struct RemoteAddRequest {
    /// Account selector, usually an account name.
    pub account_name: String,
    /// Remote URL to rewrite and attach as `origin`.
    pub repo_url: String,
    /// Existing repository path to update.
    pub repo_path: PathBuf,
}

/// Input required to switch an existing repository to a different account.
#[derive(Debug, Clone)]
pub struct RemoteSwitchRequest {
    /// Account selector, usually an account name.
    pub account_name: String,
    /// Existing repository path whose `origin` should be rewritten.
    pub repo_path: PathBuf,
}

/// Outcome of a remote add or switch operation.
#[derive(Debug, Clone)]
pub struct RemoteReport {
    /// Repository path whose remote configuration was changed.
    pub repo_path: PathBuf,
    /// Final rewritten `origin` remote URL.
    pub remote_url: String,
    /// Git username applied to the repository config.
    pub username: String,
    /// Git email applied to the repository config.
    pub email: String,
}

/// Outcome of testing SSH connectivity for an account.
#[derive(Debug, Clone)]
pub struct SshTestReport {
    /// Account used for the SSH test.
    pub account: Account,
    /// Known-host status for the provider host before the test ran.
    pub host_status: HostKeyStatus,
    /// Raw process exit status from the SSH probe.
    pub status: ExitStatus,
    /// Captured SSH stderr output, which typically contains provider messages.
    pub stderr: String,
    /// Whether the provider reported successful authentication.
    pub authenticated: bool,
    /// Whether the TCP/SSH exchange succeeded even though no shell was offered.
    pub connected_without_shell: bool,
}

/// Outcome of rotating an account SSH key.
#[derive(Debug, Clone)]
pub struct RotationReport {
    /// Account whose keypair was rotated.
    pub account: Account,
    /// Deleted old key paths before regeneration.
    pub deleted_paths: Vec<PathBuf>,
    /// Newly generated public key content to upload to the provider.
    pub public_key: String,
}

/// Outcome of generating a managed SSH key before account registration.
#[derive(Debug, Clone)]
pub struct KeyProvisionReport {
    /// Filename of the managed private key.
    pub key_path: String,
    /// Public key content to upload to the provider.
    pub public_key: String,
}

/// Outcome of deleting managed SSH key files for an account.
#[derive(Debug, Clone)]
pub struct KeyDeletionReport {
    /// Paths deleted from the SSH directory.
    pub deleted_paths: Vec<PathBuf>,
}

impl Gitcore {
    /// Initializes a new Git repository at the specified path.
    pub fn init_git_repo(&self, path: &Path) -> Result<()> {
        git::init_repository_with(self.runner(), path)?;
        Ok(())
    }

    /// Creates a library instance using the default user paths.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a library instance using explicit filesystem paths.
    #[must_use]
    pub fn with_paths(paths: GitcorePaths) -> Self {
        Self::with_runner(paths, Arc::new(SystemCommandRunner))
    }

    /// Returns the paths this instance operates against.
    #[must_use]
    pub fn paths(&self) -> &GitcorePaths {
        &self.paths
    }

    pub(crate) fn runner(&self) -> &dyn CommandRunner {
        self.runner.as_ref()
    }

    #[must_use]
    pub(crate) fn with_runner(paths: GitcorePaths, runner: Arc<dyn CommandRunner>) -> Self {
        Self { paths, runner }
    }

    /// Loads the persisted Gitcore configuration.
    ///
    /// # Errors
    /// Returns an error if the configuration file is corrupted or cannot be read.
    pub fn load_config(&self) -> Result<GitcoreConfig> {
        config::load_config_from_path(&self.paths.config_path)
    }

    /// Saves the Gitcore configuration after validating account invariants.
    ///
    /// # Errors
    /// Returns an error if the account invariants are violated or the file cannot
    /// be written atomically.
    pub fn save_config(&self, config: &GitcoreConfig) -> Result<()> {
        validate_accounts(&config.accounts).map_err(GitcoreError::InvalidConfig)?;
        config::save_config_to_path(config, &self.paths.config_path)
    }
}

impl Default for Gitcore {
    fn default() -> Self {
        Self::with_paths(GitcorePaths::default())
    }
}

impl std::fmt::Debug for Gitcore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gitcore")
            .field("paths", &self.paths)
            .field("runner", &"<command-runner>")
            .finish()
    }
}

fn audit_file(path: &Path, expected_permissions: u32) -> FileAudit {
    FileAudit {
        path: path.to_path_buf(),
        exists: path.exists(),
        permissions: file_permissions(path),
        expected_permissions,
    }
}

fn file_permissions(path: &Path) -> Option<u32> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .ok()
            .map(|metadata| metadata.permissions().mode() & 0o777)
    }

    #[cfg(not(unix))]
    {
        let _ = path;
        None
    }
}

fn write_atomic(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, contents)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

fn set_permissions_if_unix(path: &Path, mode: u32) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }

    #[cfg(not(unix))]
    {
        let _ = (path, mode);
    }

    Ok(())
}

fn repository_path(working_dir: &Path, repo_url: &str) -> PathBuf {
    let name = repo_url
        .split('/')
        .map(|s| s.trim())
        .rfind(|s| !s.is_empty())
        .map(|s| s.trim_end_matches(".git"))
        .filter(|s| !s.is_empty())
        .unwrap_or("repo");
    working_dir.join(name)
}
