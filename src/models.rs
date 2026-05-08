use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;

/// A persisted identity record for a Git hosting platform.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    /// Human-facing account name such as `work` or `personal`.
    pub name: String,
    /// Git hosting platform this identity belongs to.
    pub platform: Platform,
    /// Filename of the associated SSH private key in the managed SSH directory.
    pub key_path: String,
    /// Unique host alias used in SSH config and rewritten Git URLs.
    pub host_alias: String,
    /// Git author name (user.name).
    pub username: String,
    /// Git author email (user.email) and SSH key comment.
    pub email: String,
    /// Optional GPG signing key ID.
    pub gpg_key_id: Option<String>,
}

/// Supported Git hosting platforms.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// GitHub (github.com)
    #[default]
    Github,
    /// GitLab (gitlab.com)
    Gitlab,
    /// Codeberg (codeberg.org)
    Codeberg,
    /// Bitbucket (bitbucket.org)
    Bitbucket,
}

impl Platform {
    /// Returns the Git hosting domain used for SSH and URL rewriting.
    #[must_use]
    pub fn host(&self) -> &str {
        match self {
            Platform::Github => "github.com",
            Platform::Gitlab => "gitlab.com",
            Platform::Codeberg => "codeberg.org",
            Platform::Bitbucket => "bitbucket.org",
        }
    }

    /// Returns the browser URL where SSH public keys are managed.
    #[must_use]
    pub fn provider_key_url(&self) -> &'static str {
        match self {
            Platform::Github => "https://github.com/settings/keys",
            Platform::Gitlab => "https://gitlab.com/-/profile/keys",
            Platform::Codeberg => "https://codeberg.org/user/keys",
            Platform::Bitbucket => "https://bitbucket.org/account/settings/ssh-keys/",
        }
    }
}

impl FromStr for Platform {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "github" => Ok(Platform::Github),
            "gitlab" => Ok(Platform::Gitlab),
            "codeberg" => Ok(Platform::Codeberg),
            "bitbucket" | "bb" => Ok(Platform::Bitbucket),
            _ => Err(()),
        }
    }
}

/// Root configuration object containing all managed accounts.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct GitcoreConfig {
    /// List of registered identities.
    pub accounts: Vec<Account>,
}

/// An encrypted container for configuration and private key material.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vault {
    /// The Gitcore configuration.
    pub config: GitcoreConfig,
    /// Embedded SSH key material.
    pub keys: Vec<VaultKey>,
}

/// Embedded SSH keypair material.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultKey {
    /// Filename for the key (e.g., `id_ed25519_work`).
    pub filename: String,
    /// Full content of the private key file.
    pub private_content: String,
    /// Full content of the public key file.
    pub public_content: String,
}

/// Validates that an account name contains only safe characters.
#[must_use]
pub fn is_valid_account_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Checks a list of accounts for name and alias collisions and valid fields.
///
/// # Errors
/// Returns an error string if any invariant is violated.
pub fn validate_accounts(accounts: &[Account]) -> Result<(), String> {
    let mut names = HashSet::new();
    let mut aliases = HashSet::new();

    for acc in accounts {
        if !is_valid_account_name(&acc.name) {
            return Err(format!("Invalid account name '{}'", acc.name));
        }

        if acc.username.trim().is_empty() {
            return Err(format!("Account '{}' has an empty username", acc.name));
        }

        if acc.email.trim().is_empty() {
            return Err(format!("Account '{}' has an empty email", acc.name));
        }

        let normalized_name = acc.name.to_ascii_lowercase();
        if !names.insert(normalized_name) {
            return Err(format!("Duplicate account name '{}'", acc.name));
        }

        let normalized_alias = acc.host_alias.to_ascii_lowercase();
        if !aliases.insert(normalized_alias) {
            return Err(format!("Duplicate host alias '{}'", acc.host_alias));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_account_name() {
        assert!(is_valid_account_name("work"));
        assert!(is_valid_account_name("personal-git"));
        assert!(is_valid_account_name("user_123"));
        assert!(!is_valid_account_name(""));
        assert!(!is_valid_account_name("work account"));
        assert!(!is_valid_account_name("work@git"));
    }

    #[test]
    fn test_validate_accounts_duplicates() {
        let accounts = vec![
            Account {
                name: "test".to_string(),
                platform: Platform::Github,
                key_path: "key".to_string(),
                host_alias: "alias".to_string(),
                username: "user".to_string(),
                email: "email".to_string(),
                gpg_key_id: None,
            },
            Account {
                name: "TEST".to_string(), // Duplicate name (case-insensitive)
                platform: Platform::Gitlab,
                key_path: "k2".to_string(),
                host_alias: "h2".to_string(),
                username: "u2".to_string(),
                email: "e2".to_string(),
                gpg_key_id: None,
            },
        ];
        assert!(validate_accounts(&accounts).is_err());

        let accounts = vec![
            Account {
                name: "test".to_string(),
                platform: Platform::Github,
                key_path: "key".to_string(),
                host_alias: "alias".to_string(),
                username: "user".to_string(),
                email: "email".to_string(),
                gpg_key_id: None,
            },
            Account {
                name: "a2".to_string(),
                platform: Platform::Github,
                key_path: "k2".to_string(),
                host_alias: "ALIAS".to_string(), // Duplicate alias (case-insensitive)
                username: "u2".to_string(),
                email: "e2".to_string(),
                gpg_key_id: None,
            },
        ];
        assert!(validate_accounts(&accounts).is_err());
    }

    #[test]
    fn test_validate_accounts_empty_fields() {
        let accounts = vec![Account {
            name: "work".to_string(),
            platform: Platform::Github,
            key_path: "k1".to_string(),
            host_alias: "h1".to_string(),
            username: "  ".to_string(),
            email: "e1".to_string(),
            gpg_key_id: None,
        }];
        assert!(validate_accounts(&accounts).is_err());
    }

    #[test]
    fn provider_key_urls_match_expected_hosts() {
        assert_eq!(
            Platform::Github.provider_key_url(),
            "https://github.com/settings/keys"
        );
        assert_eq!(
            Platform::Gitlab.provider_key_url(),
            "https://gitlab.com/-/profile/keys"
        );
    }
}
