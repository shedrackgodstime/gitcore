use super::*;

impl Gitcore {
    /// Creates an encrypted backup containing config and any readable managed SSH keys.
    ///
    /// Missing private keys are recorded in the returned report rather than treated as fatal.
    ///
    /// # Errors
    /// Returns [`GitcoreError::EmptyPassword`] if the password is empty, or an IO/Encryption
    /// error if the process fails.
    pub fn backup_to_path(&self, output_path: &Path, password: &str) -> Result<BackupReport> {
        if password.is_empty() {
            return Err(GitcoreError::EmptyPassword);
        }

        let config = self.load_config()?;
        let snapshot = storage::collect_backup_keys(&config.accounts, &self.paths.ssh_dir)?;

        let encrypted = encrypt_vault(
            Vault {
                config,
                keys: snapshot.vault_keys,
            },
            password,
        )?;

        write_atomic(output_path, &encrypted)?;

        Ok(BackupReport {
            output_path: output_path.to_path_buf(),
            included_keys: snapshot.included_keys,
            missing_keys: snapshot.missing_keys,
        })
    }

    /// Restores configuration and managed SSH material from a vault or legacy JSON export.
    ///
    /// When `input_path` ends with `.json`, the file is treated as a legacy config-only export
    /// and no password is required. Otherwise an encrypted vault is expected.
    ///
    /// # Errors
    /// Returns an error if the password is incorrect, the vault is corrupted,
    /// or account validation fails.
    pub fn restore_from_path(
        &self,
        input_path: &Path,
        password: Option<&str>,
    ) -> Result<RestoreReport> {
        let input_name = input_path.to_string_lossy();
        if input_name.ends_with(".json") {
            let content = fs::read_to_string(input_path)?;
            let config: GitcoreConfig = serde_json::from_str(&content)?;
            validate_accounts(&config.accounts).map_err(GitcoreError::InvalidConfig)?;
            self.save_config(&config)?;
            self.sync_ssh_config(&config.accounts)?;
            return Ok(RestoreReport {
                restored_accounts: config.accounts.len(),
                restored_keys: Vec::new(),
                source_path: input_path.to_path_buf(),
                legacy_json: true,
            });
        }

        let password = password.ok_or(GitcoreError::EmptyPassword)?;
        if password.is_empty() {
            return Err(GitcoreError::EmptyPassword);
        }

        let data = fs::read(input_path)?;
        let vault = decrypt_vault(&data, password)?;
        validate_accounts(&vault.config.accounts).map_err(GitcoreError::InvalidConfig)?;
        self.save_config(&vault.config)?;
        let restored_keys = storage::restore_backup_keys(vault.keys, &self.paths.ssh_dir)?;

        self.sync_ssh_config(&vault.config.accounts)?;

        Ok(RestoreReport {
            restored_accounts: vault.config.accounts.len(),
            restored_keys: restored_keys.filenames,
            source_path: input_path.to_path_buf(),
            legacy_json: false,
        })
    }
}
