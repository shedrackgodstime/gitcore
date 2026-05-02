use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gity")]
#[command(version)]
#[command(about = "Manage multiple Git accounts safely with SSH keys", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new git account
    Add {
        name: Option<String>,
        platform: Option<String>,
    },

    /// List all configured accounts
    List,

    /// Clone a repo using a specific account
    Clone { repo: Option<String> },

    /// Test SSH connection to a provider
    Test { name: Option<String> },

    /// Manage git remotes for repositories
    Remote {
        #[command(subcommand)]
        subcommand: RemoteCommands,
    },

    /// Create an encrypted backup of all accounts and keys
    Backup { file: Option<String> },

    /// Restore accounts and keys from a backup or JSON
    Restore { file: Option<String> },

    /// Remove an account from Gity
    Remove { name: Option<String> },

    /// Run security audit on keys and permissions
    Audit,

    /// Rotate the SSH key for an account
    Rotate { name: Option<String> },
}

#[derive(Subcommand)]
pub enum RemoteCommands {
    /// Add a remote URL using a specific account
    Add { repo_url: Option<String> },

    /// Switch existing remote to use a different account
    Switch { account: Option<String> },
}
