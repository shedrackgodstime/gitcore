use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub name: String,
    pub platform: Platform,
    pub key_path: String,
    pub host_alias: String,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Github,
    Gitlab,
    Codeberg,
    Bitbucket,
}

impl Platform {
    pub fn host(&self) -> &str {
        match self {
            Platform::Github => "github.com",
            Platform::Gitlab => "gitlab.com",
            Platform::Codeberg => "codeberg.org",
            Platform::Bitbucket => "bitbucket.org",
        }
    }

    pub fn from_str(s: &str) -> Option<Platform> {
        match s.to_lowercase().as_str() {
            "github" => Some(Platform::Github),
            "gitlab" => Some(Platform::Gitlab),
            "codeberg" => Some(Platform::Codeberg),
            "bitbucket" | "bb" => Some(Platform::Bitbucket),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct GityConfig {
    pub accounts: Vec<Account>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vault {
    pub config: GityConfig,
    pub keys: Vec<VaultKey>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultKey {
    pub filename: String,
    pub private_content: String,
    pub public_content: String,
}

pub fn is_valid_account_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

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
                name: "Work".to_string(),
                platform: Platform::Github,
                key_path: "k1".to_string(),
                host_alias: "h1".to_string(),
                username: "u1".to_string(),
                email: "e1".to_string(),
            },
            Account {
                name: "work".to_string(), // Duplicate name (case-insensitive)
                platform: Platform::Gitlab,
                key_path: "k2".to_string(),
                host_alias: "h2".to_string(),
                username: "u2".to_string(),
                email: "e2".to_string(),
            },
        ];
        assert!(validate_accounts(&accounts).is_err());

        let accounts = vec![
            Account {
                name: "a1".to_string(),
                platform: Platform::Github,
                key_path: "k1".to_string(),
                host_alias: "alias".to_string(),
                username: "u1".to_string(),
                email: "e1".to_string(),
            },
            Account {
                name: "a2".to_string(),
                platform: Platform::Github,
                key_path: "k2".to_string(),
                host_alias: "ALIAS".to_string(), // Duplicate alias (case-insensitive)
                username: "u2".to_string(),
                email: "e2".to_string(),
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
        }];
        assert!(validate_accounts(&accounts).is_err());
    }
}
