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
    /// Add a new git account (creates SSH key + config)
    /// Add a new git account (prompts if name/platform not given)
    /// Usage: gity add [name] [platform]
    Add {
        name: Option<String>,
        platform: Option<String>,
    },

    /// List all configured accounts with usage instructions
    List,

    /// Clone a repo using a specific account (auto-sets git config)
    /// Usage: gity clone <repo_url>
    Clone { repo: Option<String> },

    /// Test SSH connection (use host_alias like github-work)
    /// Usage: gity test github-work
    Test { name: Option<String> },

    /// Manage git remotes for repositories
    Remote {
        #[command(subcommand)]
        subcommand: RemoteCommands,
    },

    /// Export configuration (for backup or moving to another PC)
    Export,

    /// Import configuration from a file or stdin
    Import { file: Option<String> },

    /// Remove an account from gity config (prompts if no name given)
    Remove { name: Option<String> },

    /// Run security audit (check file permissions, key protection, etc.)
    Audit,

    /// Rotate SSH key for an account (regenerate and show new public key)
    Rotate { name: Option<String> },
}

#[derive(Subcommand)]
pub enum RemoteCommands {
    /// Add a remote URL using a specific account
    Add { repo_url: Option<String> },

    /// Switch existing remote to use a different account
    Switch { account: Option<String> },
}
