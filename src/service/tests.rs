use super::*;
use crate::command_runner::CommandRunner;
use std::collections::VecDeque;
use std::io;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;
use std::process::Output;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

fn service_with_temp_paths() -> (Gitcore, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let paths = GitcorePaths {
        config_path: dir.path().join("config/gitcore/config.json"),
        ssh_dir: dir.path().join(".ssh"),
    };
    (Gitcore::with_paths(paths), dir)
}

fn service_with_runner(runner: Arc<dyn CommandRunner>) -> (Gitcore, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let paths = GitcorePaths {
        config_path: dir.path().join("config/gitcore/config.json"),
        ssh_dir: dir.path().join(".ssh"),
    };
    (Gitcore::with_runner(paths, runner), dir)
}

#[derive(Debug)]
struct FakeCommandRunner {
    responses: Mutex<VecDeque<io::Result<Output>>>,
    calls: Mutex<Vec<(String, Vec<String>)>>,
}

impl FakeCommandRunner {
    fn from_outputs(outputs: Vec<Output>) -> Self {
        Self {
            responses: Mutex::new(outputs.into_iter().map(Ok).collect()),
            calls: Mutex::new(Vec::new()),
        }
    }

    fn calls(&self) -> Vec<(String, Vec<String>)> {
        self.calls.lock().unwrap().clone()
    }
}

impl CommandRunner for FakeCommandRunner {
    fn run(&self, command: &str, args: &[&str]) -> io::Result<Output> {
        self.calls.lock().unwrap().push((
            command.to_string(),
            args.iter().map(|arg| (*arg).to_string()).collect(),
        ));
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Err(io::Error::other("missing fake command response")))
    }
}

#[derive(Debug, Default)]
struct KeygenCommandRunner {
    calls: Mutex<Vec<(String, Vec<String>)>>,
}

impl CommandRunner for KeygenCommandRunner {
    fn run(&self, command: &str, args: &[&str]) -> io::Result<Output> {
        self.calls.lock().unwrap().push((
            command.to_string(),
            args.iter().map(|arg| (*arg).to_string()).collect(),
        ));

        if command == "ssh-keygen" {
            let key_path = args
                .windows(2)
                .find(|window| window[0] == "-f")
                .map(|window| window[1])
                .ok_or_else(|| io::Error::other("missing -f in fake ssh-keygen args"))?;
            fs::write(key_path, "NEW PRIVATE KEY")?;
            fs::write(
                format!("{key_path}.pub"),
                "ssh-ed25519 NEW tester@example.com",
            )?;
        }

        Ok(output(0, "", ""))
    }
}

fn output(status: i32, stdout: &str, stderr: &str) -> Output {
    Output {
        status: ExitStatus::from_raw(status),
        stdout: stdout.as_bytes().to_vec(),
        stderr: stderr.as_bytes().to_vec(),
    }
}

fn sample_request(name: &str) -> AddAccountRequest {
    AddAccountRequest {
        name: name.to_string(),
        platform: Platform::Github,
        username: "tester".to_string(),
        email: "tester@example.com".to_string(),
        gpg_key_id: None,
        key_path: None,
    }
}

#[test]
fn register_account_persists_and_updates_ssh_config() {
    let (service, _dir) = service_with_temp_paths();
    let report = service.register_account(sample_request("work")).unwrap();

    assert_eq!(report.account.host_alias, "github-work");
    assert!(service.paths().config_path.exists());
    assert!(service.paths().ssh_dir.join("config").exists());
}

#[test]
fn add_remote_rejects_non_repo_paths() {
    let (service, dir) = service_with_temp_paths();
    service.register_account(sample_request("work")).unwrap();

    let result = service.add_remote(RemoteAddRequest {
        account_name: "work".to_string(),
        repo_url: "github.com/acme/project.git".to_string(),
        repo_path: dir.path().join("not-a-repo"),
    });

    assert!(matches!(result, Err(GitcoreError::NotGitRepository(_))));
}

#[test]
fn backup_and_restore_round_trip_with_temp_ssh_dir() {
    let (service, dir) = service_with_temp_paths();
    let registered = service.register_account(sample_request("work")).unwrap();

    fs::create_dir_all(&service.paths().ssh_dir).unwrap();
    fs::write(
        service.paths().ssh_dir.join(&registered.account.key_path),
        "PRIVATE KEY",
    )
    .unwrap();
    fs::write(
        service
            .paths()
            .ssh_dir
            .join(format!("{}.pub", registered.account.key_path)),
        "PUBLIC KEY",
    )
    .unwrap();

    let backup_path = dir.path().join("backup.gitcore");
    let backup = service.backup_to_path(&backup_path, "password").unwrap();
    assert_eq!(
        backup.included_keys,
        vec![registered.account.key_path.clone()]
    );

    let restore_root = tempdir().unwrap();
    let restore_service = Gitcore::with_paths(GitcorePaths {
        config_path: restore_root.path().join("config/gitcore/config.json"),
        ssh_dir: restore_root.path().join(".ssh"),
    });
    let restore = restore_service
        .restore_from_path(&backup_path, Some("password"))
        .unwrap();

    assert_eq!(restore.restored_accounts, 1);
    assert_eq!(restore.restored_keys, vec![registered.account.key_path]);
}

#[test]
fn restore_skips_missing_public_key_files() {
    let (service, dir) = service_with_temp_paths();
    let registered = service.register_account(sample_request("work")).unwrap();

    fs::create_dir_all(&service.paths().ssh_dir).unwrap();
    fs::write(
        service.paths().ssh_dir.join(&registered.account.key_path),
        "PRIVATE KEY",
    )
    .unwrap();

    let backup_path = dir.path().join("backup.gitcore");
    let backup = service.backup_to_path(&backup_path, "password").unwrap();
    assert_eq!(
        backup.included_keys,
        vec![registered.account.key_path.clone()]
    );

    let restore_root = tempdir().unwrap();
    let restore_service = Gitcore::with_paths(GitcorePaths {
        config_path: restore_root.path().join("config/gitcore/config.json"),
        ssh_dir: restore_root.path().join(".ssh"),
    });
    let restore = restore_service
        .restore_from_path(&backup_path, Some("password"))
        .unwrap();

    let restored_private = restore_service
        .paths()
        .ssh_dir
        .join(&registered.account.key_path);
    let restored_public = restore_service
        .paths()
        .ssh_dir
        .join(format!("{}.pub", registered.account.key_path));

    assert_eq!(restore.restored_keys, vec![registered.account.key_path]);
    assert!(restored_private.exists());
    assert!(!restored_public.exists());
}

#[test]
fn restore_legacy_json_rebuilds_config_and_ssh_block() {
    let (service, dir) = service_with_temp_paths();
    let registered = service.register_account(sample_request("work")).unwrap();
    let json_path = dir.path().join("legacy-config.json");

    let config = GitcoreConfig {
        accounts: vec![registered.account.clone()],
    };
    fs::write(&json_path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();

    let restore_root = tempdir().unwrap();
    let restore_service = Gitcore::with_paths(GitcorePaths {
        config_path: restore_root.path().join("config/gitcore/config.json"),
        ssh_dir: restore_root.path().join(".ssh"),
    });
    let restore = restore_service.restore_from_path(&json_path, None).unwrap();
    let restored_config = restore_service.load_config().unwrap();
    let ssh_config = fs::read_to_string(restore_service.paths().ssh_dir.join("config")).unwrap();

    assert!(restore.legacy_json);
    assert_eq!(restore.restored_accounts, 1);
    assert!(restore.restored_keys.is_empty());
    assert_eq!(restored_config.accounts.len(), 1);
    assert!(ssh_config.contains("Host github-work"));
}

#[test]
fn rotate_key_uses_injected_ssh_dir() {
    let runner = Arc::new(KeygenCommandRunner::default());
    let (service, _dir) = service_with_runner(runner);
    let registered = service.register_account(sample_request("work")).unwrap();

    fs::create_dir_all(&service.paths().ssh_dir).unwrap();
    fs::write(
        service.paths().ssh_dir.join(&registered.account.key_path),
        "OLD PRIVATE KEY",
    )
    .unwrap();
    fs::write(
        service
            .paths()
            .ssh_dir
            .join(format!("{}.pub", registered.account.key_path)),
        "OLD PUBLIC KEY",
    )
    .unwrap();

    let result = service.rotate_key("work", "").unwrap();

    assert!(!result.deleted_paths.is_empty());
    assert!(
        service
            .paths()
            .ssh_dir
            .join(&registered.account.key_path)
            .exists()
    );
    assert!(!result.public_key.is_empty());
}

#[test]
fn audit_reports_missing_private_keys() {
    let (service, _dir) = service_with_temp_paths();
    service.register_account(sample_request("work")).unwrap();

    let report = service.audit().unwrap();
    assert_eq!(report.key_audits.len(), 1);
    assert_eq!(report.issues, vec!["SSH key missing: id_ed25519_work"]);
}

#[test]
#[cfg(unix)]
fn audit_reports_incorrect_file_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let (service, _dir) = service_with_temp_paths();
    let registered = service.register_account(sample_request("work")).unwrap();

    let private_path = service.paths().ssh_dir.join(&registered.account.key_path);
    let public_path = service
        .paths()
        .ssh_dir
        .join(format!("{}.pub", registered.account.key_path));

    // Create the key files
    fs::create_dir_all(&service.paths().ssh_dir).unwrap();
    fs::write(&private_path, "PRIVATE").unwrap();
    fs::write(&public_path, "PUBLIC").unwrap();

    // Set insecure permissions (0o777 instead of 0o600 for private, 0o755 instead of 0o644 for public)
    fs::set_permissions(&private_path, fs::Permissions::from_mode(0o777)).unwrap();
    fs::set_permissions(&public_path, fs::Permissions::from_mode(0o755)).unwrap();

    // Set insecure permissions on the config file
    fs::set_permissions(
        &service.paths().config_path,
        fs::Permissions::from_mode(0o777),
    )
    .unwrap();

    let report = service.audit().unwrap();

    let issues_str = report.issues.join("\n");
    assert!(issues_str.contains("insecure permissions"));
    assert!(issues_str.contains("id_ed25519_work"));
    assert!(issues_str.contains("Gitcore config file"));
}

#[test]
fn ssh_report_shape_can_be_built_without_network() {
    let (service, _dir) = service_with_temp_paths();
    let registered = service.register_account(sample_request("work")).unwrap();
    fs::create_dir_all(&service.paths().ssh_dir).unwrap();
    fs::write(
        service.paths().ssh_dir.join("known_hosts"),
        registered.account.platform.host(),
    )
    .unwrap();

    let account = service.find_account("work").unwrap();
    let host_status = ssh::check_host_key_in_dir(&service.paths().ssh_dir, account.platform.host());
    let report = SshTestReport {
        account,
        host_status,
        status: ExitStatus::from_raw(255),
        stderr: "Permission denied".to_string(),
        authenticated: false,
        connected_without_shell: false,
    };

    assert!(matches!(report.host_status, HostKeyStatus::Known));
    assert!(!report.authenticated);
}

#[test]
fn clone_repository_uses_working_dir_and_configures_repo() {
    let runner = Arc::new(FakeCommandRunner::from_outputs(vec![
        output(0, "", ""),
        output(0, "", ""),
        output(0, "", ""),
    ]));
    let (service, dir) = service_with_runner(runner.clone());
    service.register_account(sample_request("work")).unwrap();

    let working_dir = dir.path().join("worktrees");
    let repo_url = "github.com/acme/project.git";
    let report = service
        .clone_repository(CloneRequest {
            account_name: "work".to_string(),
            repo_url: repo_url.to_string(),
            working_dir: working_dir.clone(),
        })
        .unwrap();

    assert_eq!(report.repo_path, working_dir.join("project"));
    assert_eq!(report.remote_url, "github-work:acme/project");

    let calls = runner.calls();
    assert_eq!(calls[0].0, "git");
    assert_eq!(
        calls[0].1,
        vec![
            "-C".to_string(),
            working_dir.display().to_string(),
            "clone".to_string(),
            "github-work:acme/project".to_string(),
        ]
    );
}

#[test]
fn switch_remote_uses_runner_and_rewrites_origin() {
    let runner = Arc::new(FakeCommandRunner::from_outputs(vec![
        output(0, "", ""),
        output(0, "git@github.com:acme/project.git\n", ""),
        output(0, "", ""),
        output(0, "", ""),
        output(0, "", ""),
    ]));
    let (service, dir) = service_with_runner(runner.clone());
    service.register_account(sample_request("work")).unwrap();

    let repo_path = dir.path().join("project");
    fs::create_dir_all(&repo_path).unwrap();

    let report = service
        .switch_remote(RemoteSwitchRequest {
            account_name: "work".to_string(),
            repo_path: repo_path.clone(),
        })
        .unwrap();

    assert_eq!(report.remote_url, "github-work:acme/project");

    let calls = runner.calls();
    assert_eq!(calls[0].1[2], "rev-parse");
    assert_eq!(calls[1].1[2], "remote");
    assert_eq!(
        calls[2].1,
        vec![
            "-C".to_string(),
            repo_path.display().to_string(),
            "remote".to_string(),
            "set-url".to_string(),
            "origin".to_string(),
            "github-work:acme/project".to_string(),
        ]
    );
}

#[test]
fn add_remote_updates_existing_origin_when_needed() {
    let runner = Arc::new(FakeCommandRunner {
        responses: Mutex::new(VecDeque::from([
            Ok(output(0, "", "")),
            Ok(output(0, "", "")),
            Ok(output(0, "", "")),
            Err(io::Error::other("origin already exists")),
            Ok(output(0, "", "")),
        ])),
        calls: Mutex::new(Vec::new()),
    });
    let (service, dir) = service_with_runner(runner.clone());
    service.register_account(sample_request("work")).unwrap();

    let repo_path = dir.path().join("project");
    fs::create_dir_all(&repo_path).unwrap();

    let report = service
        .add_remote(RemoteAddRequest {
            account_name: "work".to_string(),
            repo_url: "github.com/acme/project.git".to_string(),
            repo_path: repo_path.clone(),
        })
        .unwrap();

    assert_eq!(report.remote_url, "github-work:acme/project");

    let calls = runner.calls();
    assert_eq!(calls[3].1[2], "remote");
    assert_eq!(calls[4].1[2], "remote");
    assert_eq!(calls[4].1[3], "set-url");
}

#[test]
fn test_ssh_account_uses_runner_output() {
    let runner = Arc::new(FakeCommandRunner::from_outputs(vec![output(
        1,
        "",
        "Hi octocat! You've successfully authenticated, but GitHub does not provide shell access.\n",
    )]));
    let (service, _dir) = service_with_runner(runner);
    service.register_account(sample_request("work")).unwrap();
    fs::create_dir_all(&service.paths().ssh_dir).unwrap();
    fs::write(service.paths().ssh_dir.join("known_hosts"), "github.com").unwrap();

    let report = service.test_ssh_account("work").unwrap();

    assert!(matches!(report.host_status, HostKeyStatus::Known));
    assert!(report.authenticated);
    assert!(!report.connected_without_shell);
}

#[test]
fn register_account_with_custom_key_path() {
    let (service, _dir) = service_with_temp_paths();
    let mut request = sample_request("custom");
    request.key_path = Some("custom_id_rsa".to_string());

    let report = service.register_account(request).unwrap();

    assert_eq!(report.account.key_path, "custom_id_rsa");
    let config = service.load_config().unwrap();
    assert_eq!(config.accounts[0].key_path, "custom_id_rsa");
}

#[test]
fn clone_repository_defensive_path_resolution() {
    let runner = Arc::new(FakeCommandRunner::from_outputs(vec![
        output(0, "", ""),
        output(0, "", ""),
        output(0, "", ""),
        output(0, "", ""),
        output(0, "", ""),
        output(0, "", ""),
    ]));
    let (service, dir) = service_with_runner(runner.clone());
    service.register_account(sample_request("work")).unwrap();

    let working_dir = dir.path().join("worktrees");

    // Test a trailing slash URL: "https://github.com/acme/project/"
    let report1 = service
        .clone_repository(CloneRequest {
            account_name: "work".to_string(),
            repo_url: "https://github.com/acme/project/".to_string(),
            working_dir: working_dir.clone(),
        })
        .unwrap();
    assert_eq!(report1.repo_path, working_dir.join("project"));

    // Test an empty URL (should resolve to fallback "repo")
    let report2 = service
        .clone_repository(CloneRequest {
            account_name: "work".to_string(),
            repo_url: "".to_string(),
            working_dir: working_dir.clone(),
        })
        .unwrap();
    assert_eq!(report2.repo_path, working_dir.join("repo"));
}
