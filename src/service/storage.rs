use super::*;

pub(crate) struct BackupKeySnapshot {
    pub(crate) vault_keys: Vec<VaultKey>,
    pub(crate) included_keys: Vec<String>,
    pub(crate) missing_keys: Vec<String>,
}

pub(crate) struct RotatedKeyMaterial {
    pub(crate) deleted_paths: Vec<PathBuf>,
    pub(crate) public_key: String,
}

pub(crate) struct DeletedKeyMaterial {
    pub(crate) deleted_paths: Vec<PathBuf>,
}

pub(crate) struct GeneratedKeyMaterial {
    pub(crate) public_key: String,
}

pub(crate) struct RestoredKeyMaterial {
    pub(crate) filenames: Vec<String>,
}

pub(crate) fn collect_backup_keys(
    accounts: &[Account],
    ssh_dir: &Path,
) -> std::io::Result<BackupKeySnapshot> {
    let mut included_keys = Vec::new();
    let mut missing_keys = Vec::new();
    let mut vault_keys = Vec::new();

    for account in accounts {
        let private_key_path = ssh_dir.join(&account.key_path);
        let public_key_path = ssh_dir.join(format!("{}.pub", account.key_path));

        if private_key_path.exists() {
            let private_content = fs::read_to_string(&private_key_path)?;
            let public_content = if public_key_path.exists() {
                fs::read_to_string(&public_key_path)?
            } else {
                String::new()
            };

            vault_keys.push(VaultKey {
                filename: account.key_path.clone(),
                private_content,
                public_content,
            });
            included_keys.push(account.key_path.clone());
        } else {
            missing_keys.push(account.key_path.clone());
        }
    }

    Ok(BackupKeySnapshot {
        vault_keys,
        included_keys,
        missing_keys,
    })
}

pub(crate) fn restore_backup_keys(
    keys: Vec<VaultKey>,
    ssh_dir: &Path,
) -> std::io::Result<RestoredKeyMaterial> {
    ssh::ensure_ssh_dir(ssh_dir)?;

    let mut restored_keys = Vec::new();
    for key in keys {
        let private_path = ssh_dir.join(&key.filename);
        let public_path = ssh_dir.join(format!("{}.pub", key.filename));

        write_atomic(&private_path, key.private_content.as_bytes())?;
        set_permissions_if_unix(&private_path, 0o600)?;

        if !key.public_content.is_empty() {
            write_atomic(&public_path, key.public_content.as_bytes())?;
            set_permissions_if_unix(&public_path, 0o644)?;
        }

        restored_keys.push(key.filename);
    }

    Ok(RestoredKeyMaterial {
        filenames: restored_keys,
    })
}

pub(crate) fn generate_account_key(
    runner: &dyn CommandRunner,
    ssh_dir: &Path,
    key_path: &str,
    email: &str,
    passphrase: &str,
) -> std::io::Result<GeneratedKeyMaterial> {
    let public_key =
        ssh::generate_ssh_key_in_dir_with(runner, ssh_dir, key_path, email, passphrase)?;
    Ok(GeneratedKeyMaterial { public_key })
}

pub(crate) fn delete_account_keys(
    ssh_dir: &Path,
    key_path: &str,
) -> std::io::Result<DeletedKeyMaterial> {
    let deleted_paths = ssh::delete_account_keys_in_dir(ssh_dir, key_path)?;
    Ok(DeletedKeyMaterial { deleted_paths })
}

pub(crate) fn rotate_account_key(
    runner: &dyn CommandRunner,
    ssh_dir: &Path,
    key_path: &str,
    email: &str,
    passphrase: &str,
) -> std::io::Result<RotatedKeyMaterial> {
    let deleted_paths = delete_account_keys(ssh_dir, key_path)?.deleted_paths;
    let public_key = generate_account_key(runner, ssh_dir, key_path, email, passphrase)?.public_key;

    Ok(RotatedKeyMaterial {
        deleted_paths,
        public_key,
    })
}

pub(crate) fn sync_managed_ssh_config(accounts: &[Account], ssh_dir: &Path) -> std::io::Result<()> {
    ssh::update_ssh_config_in_dir(accounts, ssh_dir)
}
