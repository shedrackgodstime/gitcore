use crate::error::Result;
use crate::models::GitcoreConfig;
use std::fs;
use std::path::PathBuf;

pub fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("gitcore")
        .join("config.json")
}

pub fn load_config_from_path(path: &PathBuf) -> Result<GitcoreConfig> {
    if !path.exists() {
        return Ok(GitcoreConfig::default());
    }

    let content = fs::read_to_string(path)?;
    let config = serde_json::from_str(&content)?;
    Ok(config)
}

pub fn save_config_to_path(config: &GitcoreConfig, path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600))?;
    }

    fs::rename(&tmp_path, path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Account, Platform};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_config_cycle() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.json");

        let mut config = GitcoreConfig::default();
        config.accounts.push(Account {
            name: "test".to_string(),
            platform: Platform::Github,
            key_path: "key".to_string(),
            host_alias: "alias".to_string(),
            username: "user".to_string(),
            email: "email".to_string(),
            gpg_key_id: None,
        });

        save_config_to_path(&config, &config_path).unwrap();
        assert!(config_path.exists());

        let loaded = load_config_from_path(&config_path).unwrap();
        assert_eq!(loaded.accounts.len(), 1);
        assert_eq!(loaded.accounts[0].name, "test");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&config_path).unwrap();
            assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
        }
    }
}
