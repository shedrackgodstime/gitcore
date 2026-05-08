use super::*;

impl Gitcore {
    /// Probes SSH authentication for an account and classifies the provider response.
    ///
    /// # Errors
    /// Returns an error if the account is not found or the SSH probe fails to execute.
    pub fn test_ssh_account(&self, selector: &str) -> Result<SshTestReport> {
        let account = self.find_account(selector)?;
        let host_status = ssh::check_host_key_in_dir(&self.paths.ssh_dir, account.platform.host());
        let probe = ssh::probe_ssh_connection_with(self.runner(), &account.host_alias)?;

        Ok(SshTestReport {
            account,
            host_status,
            status: probe.status,
            authenticated: matches!(probe.state, ssh::SshConnectionState::Authenticated),
            connected_without_shell: matches!(
                probe.state,
                ssh::SshConnectionState::ConnectedWithoutShell
            ),
            stderr: probe.stderr,
        })
    }

    /// Replaces the managed SSH keypair for an account and returns the new public key.
    ///
    /// # Errors
    /// Returns an error if the account is not found or key generation/deletion fails.
    pub fn rotate_key(&self, selector: &str, passphrase: &str) -> Result<RotationReport> {
        let account = self.find_account(selector)?;
        let key_material = storage::rotate_account_key(
            self.runner(),
            &self.paths.ssh_dir,
            &account.key_path,
            &account.email,
            passphrase,
        )?;

        Ok(RotationReport {
            account,
            deleted_paths: key_material.deleted_paths,
            public_key: key_material.public_key,
        })
    }
}
