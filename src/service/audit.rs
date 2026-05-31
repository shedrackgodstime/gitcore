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
            } else if let Some(perms) = key_audit
                .private_key
                .permissions
                .filter(|&p| p != key_audit.private_key.expected_permissions)
            {
                issues.push(format!(
                    "SSH private key '{}' has insecure permissions ({:o}, expected {:o})",
                    key_audit.account.key_path, perms, key_audit.private_key.expected_permissions
                ));
            }

            if let Some(perms) = key_audit
                .public_key
                .permissions
                .filter(|&p| p != key_audit.public_key.expected_permissions)
            {
                issues.push(format!(
                    "SSH public key '{}.pub' has insecure permissions ({:o}, expected {:o})",
                    key_audit.account.key_path, perms, key_audit.public_key.expected_permissions
                ));
            }
        }

        if ssh_config.exists {
            if let Some(perms) = ssh_config
                .permissions
                .filter(|&p| p != ssh_config.expected_permissions)
            {
                issues.push(format!(
                    "SSH config file has insecure permissions ({:o}, expected {:o})",
                    perms, ssh_config.expected_permissions
                ));
            }
        } else {
            issues.push("SSH config file missing".to_string());
        }

        if config_file.exists {
            if let Some(perms) = config_file
                .permissions
                .filter(|&p| p != config_file.expected_permissions)
            {
                issues.push(format!(
                    "Gitcore config file has insecure permissions ({:o}, expected {:o})",
                    perms, config_file.expected_permissions
                ));
            }
        } else {
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
