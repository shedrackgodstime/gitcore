use super::*;

impl Gitcore {
    /// Performs a security and health audit of the Gitcore environment.
    ///
    /// Checks for missing SSH keys, incorrect file permissions, and configuration
    /// consistency.
    ///
    /// # Errors
    /// Returns an error if the configuration cannot be loaded.
    pub fn audit(&self) -> Result<AuditReport> {
        let config = self.load_config()?;
        let key_audits: Vec<KeyAudit> = config
            .accounts
            .iter()
            .cloned()
            .map(|account| {
                let private_key_path = self.paths.ssh_dir.join(&account.key_path);
                let public_key_path = self.paths.ssh_dir.join(format!("{}.pub", account.key_path));
                KeyAudit {
                    account,
                    private_key: audit_file(&private_key_path, 0o600),
                    public_key: audit_file(&public_key_path, 0o644),
                }
            })
            .collect();

        let ssh_config = audit_file(&self.paths.ssh_dir.join("config"), 0o600);
        let config_file = audit_file(&self.paths.config_path, 0o600);

        let mut issues = Vec::new();
        for key_audit in &key_audits {
            if !key_audit.private_key.exists {
                issues.push(format!("SSH key missing: {}", key_audit.account.key_path));
            }
        }
        if !ssh_config.exists {
            issues.push("SSH config file missing".to_string());
        }
        if !config_file.exists {
            issues.push("Gitcore config file missing".to_string());
        }

        Ok(AuditReport {
            key_audits,
            ssh_config,
            config_file,
            issues,
        })
    }
}
