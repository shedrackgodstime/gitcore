use gitcore::{AddAccountRequest, Gitcore, GitcorePaths, Platform};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_account_lifecycle_integration() {
    let root = tempdir().unwrap();
    let paths = GitcorePaths {
        config_path: root.path().join("config.json"),
        ssh_dir: root.path().join(".ssh"),
    };
    let service = Gitcore::with_paths(paths.clone());

    // 1. Register account
    let request = AddAccountRequest {
        name: "work".to_string(),
        platform: Platform::Github,
        username: "tester".to_string(),
        email: "tester@example.com".to_string(),
        gpg_key_id: None,
        key_path: None,
    };
    service
        .register_account(request)
        .expect("registration failed");

    // 2. Verify persistence
    let accounts = service.list_accounts().expect("listing failed");
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].name, "work");

    // 3. Verify SSH config sync
    let ssh_config = fs::read_to_string(paths.ssh_dir.join("config")).expect("missing ssh config");
    assert!(ssh_config.contains("Host github-work"));
    assert!(ssh_config.contains("HostName github.com"));

    // 4. Remove account
    service.remove_account("work").expect("removal failed");
    let accounts = service.list_accounts().expect("listing failed");
    assert!(accounts.is_empty());

    // 5. Verify SSH config cleanup
    let ssh_config = fs::read_to_string(paths.ssh_dir.join("config")).unwrap();
    assert!(!ssh_config.contains("Host github-work"));
}

#[test]
fn test_backup_and_restore_vault_integration() {
    let root = tempdir().unwrap();
    let paths = GitcorePaths {
        config_path: root.path().join("config.json"),
        ssh_dir: root.path().join(".ssh"),
    };
    let service = Gitcore::with_paths(paths);

    // Setup: Register account and "mock" key files
    let request = AddAccountRequest {
        name: "work".to_string(),
        platform: Platform::Github,
        username: "tester".to_string(),
        email: "tester@example.com".to_string(),
        gpg_key_id: None,
        key_path: None,
    };
    let registered = service.register_account(request).unwrap();

    fs::create_dir_all(&service.paths().ssh_dir).unwrap();
    let private_path = service.paths().ssh_dir.join(&registered.account.key_path);
    let public_path = service
        .paths()
        .ssh_dir
        .join(format!("{}.pub", registered.account.key_path));
    fs::write(&private_path, "PRIVATE CONTENT").unwrap();
    fs::write(&public_path, "PUBLIC CONTENT").unwrap();

    // 1. Backup
    let backup_path = root.path().join("backup.vault");
    service
        .backup_to_path(&backup_path, "password")
        .expect("backup failed");

    // 2. Restore to a clean location
    let restore_root = tempdir().unwrap();
    let restore_paths = GitcorePaths {
        config_path: restore_root.path().join("restored.json"),
        ssh_dir: restore_root.path().join(".ssh"),
    };
    let restore_service = Gitcore::with_paths(restore_paths);

    let report = restore_service
        .restore_from_path(&backup_path, Some("password"))
        .expect("restore failed");

    assert_eq!(report.restored_accounts, 1);
    assert_eq!(report.restored_keys.len(), 1);

    // 3. Verify restored files
    let restored_config = restore_service.load_config().unwrap();
    assert_eq!(restored_config.accounts[0].name, "work");

    let restored_private = restore_service
        .paths()
        .ssh_dir
        .join(&registered.account.key_path);
    assert_eq!(
        fs::read_to_string(restored_private).unwrap(),
        "PRIVATE CONTENT"
    );
}
