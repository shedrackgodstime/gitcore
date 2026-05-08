use super::*;

impl Gitcore {
    /// Returns every configured account in persisted order.
    ///
    /// # Errors
    /// Returns an error if the configuration cannot be loaded.
    pub fn list_accounts(&self) -> Result<Vec<Account>> {
        Ok(self.load_config()?.accounts)
    }

    /// Finds an account by exact name or by a host-alias prefix.
    ///
    /// # Errors
    /// Returns [`GitcoreError::AccountNotFound`] if no match is found.
    pub fn find_account(&self, selector: &str) -> Result<Account> {
        let config = self.load_config()?;
        config
            .accounts
            .into_iter()
            .find(|account| account.name == selector || account.host_alias.starts_with(selector))
            .ok_or_else(|| GitcoreError::AccountNotFound(selector.to_string()))
    }

    /// Finds an account by its full generated host alias.
    ///
    /// # Errors
    /// Returns [`GitcoreError::AccountNotFound`] if no match is found.
    pub fn find_account_by_host_alias(&self, host_alias: &str) -> Result<Account> {
        let config = self.load_config()?;
        config
            .accounts
            .into_iter()
            .find(|account| account.host_alias == host_alias)
            .ok_or_else(|| GitcoreError::AccountNotFound(host_alias.to_string()))
    }

    /// Rewrites the managed SSH config block for the supplied accounts.
    ///
    /// # Errors
    /// Returns an error if the account list is invalid or the SSH config cannot be written.
    pub fn sync_ssh_config(&self, accounts: &[Account]) -> Result<()> {
        validate_accounts(accounts).map_err(GitcoreError::InvalidConfig)?;
        storage::sync_managed_ssh_config(accounts, &self.paths.ssh_dir).map_err(Into::into)
    }

    /// Validates and persists an already-constructed account record.
    ///
    /// # Errors
    /// Returns an error if the account record violates invariants (e.g., duplicate names)
    /// or if persistence fails.
    pub fn add_account(&self, account: Account) -> Result<()> {
        if !is_valid_account_name(&account.name) {
            return Err(GitcoreError::InvalidAccountName(account.name));
        }
        if account.username.trim().is_empty() {
            return Err(GitcoreError::EmptyUsername);
        }
        if account.email.trim().is_empty() {
            return Err(GitcoreError::EmptyEmail);
        }

        let mut config = self.load_config()?;

        if config
            .accounts
            .iter()
            .any(|existing| existing.name.eq_ignore_ascii_case(&account.name))
        {
            return Err(GitcoreError::DuplicateAccountName(account.name));
        }

        if config.accounts.iter().any(|existing| {
            existing
                .host_alias
                .eq_ignore_ascii_case(&account.host_alias)
        }) {
            return Err(GitcoreError::DuplicateHostAlias(account.host_alias));
        }

        config.accounts.push(account);
        self.save_config(&config)?;
        self.sync_ssh_config(&config.accounts)
    }

    /// # Errors
    /// Returns an error if the request is invalid or persistence fails.
    ///
    /// # Examples
    /// ```no_run
    /// # use gitcore::{Gitcore, AddAccountRequest, Platform};
    /// # let service = Gitcore::new();
    /// let request = AddAccountRequest {
    ///     name: "work".to_string(),
    ///     platform: Platform::Github,
    ///     username: "octocat".to_string(),
    ///     email: "octocat@example.com".to_string(),
    ///     gpg_key_id: None,
    ///     key_path: None,
    /// };
    /// let report = service.register_account(request)?;
    /// assert_eq!(report.host_alias, "github-work");
    /// # Ok::<(), gitcore::GitcoreError>(())
    /// ```
    pub fn register_account(&self, request: AddAccountRequest) -> Result<RegisteredAccount> {
        let host_alias = format!(
            "{}-{}",
            request.platform.host().split('.').next().unwrap_or("git"),
            request.name
        );
        let account_name = request.name;
        let key_path = request
            .key_path
            .unwrap_or_else(|| format!("id_ed25519_{}", account_name));

        let account = Account {
            name: account_name.clone(),
            platform: request.platform,
            key_path,
            host_alias: host_alias.clone(),
            username: request.username,
            email: request.email,
            gpg_key_id: request.gpg_key_id,
        };

        self.add_account(account.clone())?;
        Ok(RegisteredAccount {
            account,
            host_alias,
        })
    }

    /// Removes an account from persisted configuration and rewrites the managed SSH config.
    ///
    /// This does not delete key files automatically. Call [`Gitcore::delete_account_key_files`]
    /// if the caller wants filesystem cleanup as part of removal.
    ///
    /// # Errors
    /// Returns [`GitcoreError::AccountNotFound`] if the account does not exist.
    pub fn remove_account(&self, account_name: &str) -> Result<Account> {
        let mut config = self.load_config()?;
        let Some(index) = config
            .accounts
            .iter()
            .position(|acc| acc.name == account_name)
        else {
            return Err(GitcoreError::AccountNotFound(account_name.to_string()));
        };

        let account = config.accounts.remove(index);
        self.save_config(&config)?;
        self.sync_ssh_config(&config.accounts)?;
        Ok(account)
    }

    /// Updates an existing account's metadata.
    ///
    /// # Errors
    /// Returns [`GitcoreError::AccountNotFound`] if the account does not exist.
    pub fn update_account(
        &self,
        account_name: &str,
        request: UpdateAccountRequest,
    ) -> Result<Account> {
        let mut config = self.load_config()?;
        let Some(account) = config
            .accounts
            .iter_mut()
            .find(|acc| acc.name == account_name)
        else {
            return Err(GitcoreError::AccountNotFound(account_name.to_string()));
        };

        if let Some(username) = request.username {
            account.username = username;
        }
        if let Some(email) = request.email {
            account.email = email;
        }
        if let Some(gpg_id) = request.gpg_key_id {
            account.gpg_key_id = gpg_id;
        }

        let updated_account = account.clone();
        self.save_config(&config)?;
        Ok(updated_account)
    }

    /// Generates the managed SSH keypair that should be uploaded before registration succeeds.
    ///
    /// The returned `public_key` can be pasted into the provider page referenced by
    /// [`Platform::provider_key_url`](crate::Platform::provider_key_url).
    ///
    /// # Errors
    /// Returns an error if key generation fails (e.g., `ssh-keygen` not found).
    pub fn provision_account_keys(
        &self,
        request: &AddAccountRequest,
        passphrase: &str,
    ) -> Result<KeyProvisionReport> {
        let key_path = request
            .key_path
            .clone()
            .unwrap_or_else(|| format!("id_ed25519_{}", request.name));

        let key_material = storage::generate_account_key(
            self.runner(),
            &self.paths.ssh_dir,
            &key_path,
            &request.email,
            passphrase,
        )?;

        Ok(KeyProvisionReport {
            key_path,
            public_key: key_material.public_key,
        })
    }

    /// Deletes the managed SSH key files for an account when cleanup is requested.
    ///
    /// # Errors
    /// Returns an error if filesystem deletions fail.
    pub fn delete_account_key_files(&self, key_path: &str) -> Result<KeyDeletionReport> {
        let deleted = storage::delete_account_keys(&self.paths.ssh_dir, key_path)?;
        Ok(KeyDeletionReport {
            deleted_paths: deleted.deleted_paths,
        })
    }

    /// Parses a user-facing platform string such as `github` or `gitlab`.
    ///
    /// # Errors
    /// Returns [`GitcoreError::UnsupportedPlatform`] if the string is unknown.
    pub fn parse_platform(&self, value: &str) -> Result<Platform> {
        value
            .parse::<Platform>()
            .map_err(|()| GitcoreError::UnsupportedPlatform(value.to_string()))
    }
}
