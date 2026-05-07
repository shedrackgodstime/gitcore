use crate::cli::{Cli, Commands, RemoteCommands};
use crate::config::{load_config, save_config};
use crate::git::{
    convert_to_host, ensure_git_repository, run_git, run_git_remote_add, set_git_config,
};
use crate::models::{
    Account, GitcoreConfig, Platform, Vault, VaultKey, is_valid_account_name, validate_accounts,
};
use crate::ssh::{
    HostKeyStatus, check_host_key, delete_account_keys, generate_ssh_key, get_ssh_dir,
    provider_key_url, test_ssh_connection, update_ssh_config,
};
use crate::ui::{confirm, print_result, prompt_input, prompt_password, select_account};
use crate::vault::{decrypt_vault, encrypt_vault};
use colored::Colorize;
use std::fs;
use std::io::{self};
use std::path::PathBuf;
use std::process::Command;

pub fn run(cli: Cli) -> io::Result<()> {
    match cli.command {
        Commands::Add { name, platform } => {
            handle_add_account(name, platform)?;
        }

        Commands::List => {
            let config = load_config();
            if config.accounts.is_empty() {
                println!("{}", "No accounts configured.".yellow());
                if confirm("Would you like to add your first account now? [y/N]: ")? {
                    handle_add_account(None, None)?;
                }
                return Ok(());
            }

            println!();
            println!("{}", "Configured Git Accounts".cyan().bold());
            println!("{}", "=".repeat(23).cyan());
            println!();

            for (i, acc) in config.accounts.iter().enumerate() {
                println!("[{}] {}", i + 1, acc.name.bold());
                println!("   Platform: {:?}", acc.platform);
                println!("   Host:     {}", acc.host_alias);
                println!("   Key:      {}", acc.key_path);
                println!("   User:     {}", acc.username);
                println!("   Email:    {}", acc.email);
                if let Some(gpg_id) = &acc.gpg_key_id {
                    println!("   GPG:      {}", gpg_id.cyan());
                }
                println!(
                    "   Use:      git clone git@{}:user/repo.git",
                    acc.host_alias
                );
                println!();
            }
        }

        Commands::Clone { repo } => {
            let config = load_config();
            if config.accounts.is_empty() {
                println!("{}", "No accounts configured.".yellow());
                if confirm("Would you like to add an account now? [y/N]: ")? {
                    handle_add_account(None, None)?;
                }
                return Ok(());
            }

            let preloaded: Option<(String, &Account)> = if let Some(url) = repo.clone() {
                if let Some(stripped) = url.strip_prefix("git@") {
                    if let Some(colon_pos) = stripped.find(':') {
                        let alias = &stripped[..colon_pos];
                        if let Some(found) = config.accounts.iter().find(|a| a.host_alias == alias)
                        {
                            Some((url, found))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some((url, acc)) = preloaded {
                let converted = convert_to_host(&url, &acc.host_alias);
                println!(
                    "{}",
                    format!("  Using account: {} ({})", acc.name, acc.host_alias).cyan()
                );
                println!("  Cloning: {}", converted);

                let clone_dir = std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(
                        url.split('/')
                            .next_back()
                            .unwrap_or("repo")
                            .trim_end_matches(".git"),
                    );

                if clone_dir.exists() {
                    if !ensure_git_repository(&clone_dir) {
                        println!("{}", "[!] Directory exists but not a git repository.".red());
                        return Ok(());
                    }
                } else if let Err(err) = run_git(&["clone", &converted]) {
                    println!("{}", "[x] Clone failed".red());
                    println!("   {}", err);
                    return Ok(());
                }

                std::env::set_current_dir(&clone_dir).ok();
                if let Err(err) =
                    set_git_config(&acc.username, &acc.email, acc.gpg_key_id.as_deref())
                {
                    println!(
                        "{}",
                        "Error: Clone succeeded, but git config update failed".red()
                    );
                    println!("   {}", err);
                    return Ok(());
                }

                println!("{}", "[v] Cloned with git config:".green());
                println!("  user.name  = {}", acc.username);
                println!("  user.email = {}", acc.email);
                return Ok(());
            }

            let Some(choice) = select_account(&config.accounts, "Select account to use:")? else {
                println!("{}", "Invalid selection".red());
                return Ok(());
            };

            let acc = &config.accounts[choice];
            let repo_url =
                repo.unwrap_or_else(|| prompt_input("Enter repository URL: ").unwrap_or_default());

            let converted = convert_to_host(&repo_url, &acc.host_alias);

            println!("\n{}", "Cloning with account:".yellow());
            println!("  Account: {}", acc.name);
            println!("  Email:   {}", acc.email);
            println!("  Remote:  {}", converted);
            println!();

            let clone_dir = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(
                    repo_url
                        .split('/')
                        .next_back()
                        .unwrap_or("repo")
                        .trim_end_matches(".git"),
                );

            if clone_dir.exists() {
                if !ensure_git_repository(&clone_dir) {
                    println!("{}", "[!] Directory exists but not a git repository.".red());
                    return Ok(());
                }
                println!(
                    "{}",
                    "Directory already exists. Using existing repo.".yellow()
                );
                if let Err(err) = run_git_remote_add(&converted) {
                    println!("{}", "[x] Failed to configure origin remote".red());
                    println!("   {}", err);
                    return Ok(());
                }
            } else {
                println!("{}", "Running: git clone".cyan());
                if let Err(err) = run_git(&["clone", &converted]) {
                    println!("{}", "[x] Clone failed".red());
                    println!("   {}", err);
                    return Ok(());
                }
            }

            let target_dir = if clone_dir.exists() {
                clone_dir.clone()
            } else {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            };

            std::env::set_current_dir(&target_dir).ok();

            if let Err(err) = set_git_config(&acc.username, &acc.email, acc.gpg_key_id.as_deref()) {
                println!(
                    "{}",
                    "Error: Clone succeeded, but git config update failed".red()
                );
                println!("   {}", err);
                return Ok(());
            }

            println!("{}", "[v] Cloned and git config set:".green());
            println!("  user.name  = {}", acc.username);
            println!("  user.email = {}", acc.email);
        }

        Commands::Test { name } => {
            let config = load_config();
            if config.accounts.is_empty() {
                println!("{}", "No accounts configured. Run 'gitcore add' first".red());
                return Ok(());
            }

            let target = if let Some(n) = name {
                config
                    .accounts
                    .iter()
                    .find(|a| a.name == n || a.host_alias.starts_with(&n))
            } else {
                match select_account(&config.accounts, "Select account to test:")? {
                    Some(choice) => Some(&config.accounts[choice]),
                    None => None,
                }
            };

            if let Some(acc) = target {
                println!(
                    "\n{}",
                    format!("Testing connection to {}...", acc.platform.host()).cyan()
                );

                let host_status = check_host_key(acc.platform.host());
                match host_status {
                    HostKeyStatus::Known => {
                        println!("{}", "  [v] Host key is known".green());
                    }
                    HostKeyStatus::New => {
                        println!(
                            "{}",
                            "  [!] New host key - will be added to known_hosts".yellow()
                        );
                    }
                    HostKeyStatus::Unknown => {
                        println!("{}", "  [!] No known_hosts file - will be created".yellow());
                    }
                }

                let output = test_ssh_connection(&acc.host_alias);

                match output {
                    Ok(out) => {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        if stderr.contains("successfully authenticated") || stderr.contains("Hi") {
                            println!("{}", "[v] Connection successful!".green());
                            println!("   {}", stderr.trim());
                        } else if out.status.success() {
                            println!("{}", "[v] Connected (but no shell access)".green());
                        } else {
                            println!("{}", "[x] Connection failed".red());
                            let detail = stderr.trim();
                            if detail.is_empty() {
                                println!("   exit status: {}", out.status);
                            } else {
                                println!("   {}", detail);
                                if detail.contains("Permission denied") {
                                    println!();
                                    println!("{}", "  HINT: HINT: If your key has a passphrase, SSH might be failing because".yellow());
                                    println!(
                                        "{}",
                                        "     it cannot ask for it in batch mode.".yellow()
                                    );
                                    println!(
                                        "{}",
                                        "     Try adding your key to the agent first:".yellow()
                                    );
                                    println!("     ssh-add ~/.ssh/{}", acc.key_path);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("{} {}", "[x] Error:".red(), e);
                    }
                }
            }
        }

        Commands::Remote { subcommand } => match subcommand {
            RemoteCommands::Add { repo_url } => {
                let git_check = Command::new("git")
                    .args(["rev-parse", "--git-dir"])
                    .output();
                if git_check.is_err()
                    || !git_check
                        .as_ref()
                        .map(|o| o.status.success())
                        .unwrap_or(false)
                {
                    println!(
                        "{}",
                        "[!] Not a git repository. Run 'git init' first.".red()
                    );
                    return Ok(());
                }

                let config = load_config();
                if config.accounts.is_empty() {
                    println!("{}", "No accounts configured.".yellow());
                    if confirm("Would you like to add an account now? [y/N]: ")? {
                        handle_add_account(None, None)?;
                    }
                    return Ok(());
                }

                let preloaded: Option<(String, &Account)> = if let Some(url) = repo_url.clone() {
                    if let Some(stripped) = url.strip_prefix("git@") {
                        if let Some(colon_pos) = stripped.find(':') {
                            let alias = &stripped[..colon_pos];
                            if let Some(found) =
                                config.accounts.iter().find(|a| a.host_alias == alias)
                            {
                                Some((url, found))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some((url, acc)) = preloaded {
                    let converted = convert_to_host(&url, &acc.host_alias);
                    if let Err(err) =
                        set_git_config(&acc.username, &acc.email, acc.gpg_key_id.as_deref())
                    {
                        println!("{}", "Error: Failed to set git config".red());
                        println!("   {}", err);
                        return Ok(());
                    }

                    if let Err(err) = run_git_remote_add(&converted) {
                        println!("{}", "[x] Failed to configure origin remote".red());
                        println!("   {}", err);
                        return Ok(());
                    }
                    print_result(&converted, &acc.username, &acc.email);
                    return Ok(());
                }

                let manual_url = repo_url.clone();

                let Some(choice) = select_account(&config.accounts, "Select an account to use:")?
                else {
                    println!("{}", "Invalid selection".red());
                    return Ok(());
                };

                let selected_acc = &config.accounts[choice];
                let final_repo = manual_url
                    .unwrap_or_else(|| prompt_input("Enter repository URL: ").unwrap_or_default());

                let converted = convert_to_host(&final_repo, &selected_acc.host_alias);
                if let Err(err) = set_git_config(
                    &selected_acc.username,
                    &selected_acc.email,
                    selected_acc.gpg_key_id.as_deref(),
                ) {
                    println!("{}", "Error: Failed to set git config".red());
                    println!("   {}", err);
                    return Ok(());
                }

                if let Err(err) = run_git_remote_add(&converted) {
                    println!("{}", "[x] Failed to configure origin remote".red());
                    println!("   {}", err);
                    return Ok(());
                }
                print_result(&converted, &selected_acc.username, &selected_acc.email);
            }

            RemoteCommands::Switch { account } => {
                let git_check = Command::new("git")
                    .args(["rev-parse", "--git-dir"])
                    .output();
                if git_check.is_err()
                    || !git_check
                        .as_ref()
                        .map(|o| o.status.success())
                        .unwrap_or(false)
                {
                    println!("{}", "[!] Not a git repository.".red());
                    return Ok(());
                }

                let config = load_config();
                if config.accounts.is_empty() {
                    println!("{}", "No accounts configured".red());
                    return Ok(());
                }

                let target = if let Some(name) = account {
                    config
                        .accounts
                        .iter()
                        .find(|a| a.name == name || a.host_alias.starts_with(&name))
                } else {
                    match select_account(&config.accounts, "Select new account:")? {
                        Some(choice) => Some(&config.accounts[choice]),
                        None => None,
                    }
                };

                if let Some(acc) = target {
                    let remote_output = Command::new("git")
                        .args(["remote", "get-url", "origin"])
                        .output();

                    let current_remote = match remote_output {
                        Ok(out) if out.status.success() => {
                            String::from_utf8_lossy(&out.stdout).trim().to_string()
                        }
                        _ => {
                            println!("{}", "[x] Could not read origin remote".red());
                            return Ok(());
                        }
                    };

                    let converted = convert_to_host(&current_remote, &acc.host_alias);
                    if let Err(err) = run_git(&["remote", "set-url", "origin", &converted]) {
                        println!("{}", "[x] Failed to switch remote".red());
                        println!("   {}", err);
                        return Ok(());
                    }

                    if let Err(err) =
                        set_git_config(&acc.username, &acc.email, acc.gpg_key_id.as_deref())
                    {
                        println!(
                            "{}",
                            "Error: Clone succeeded, but git config update failed".red()
                        );
                        println!("   {}", err);
                        return Ok(());
                    }

                    println!("{}", "[v] Remote switched".green());
                    println!("  origin     = {}", converted);
                    println!("  user.name  = {}", acc.username);
                    println!("  user.email = {}", acc.email);
                }
            }
        },

        Commands::Backup { file } => {
            let config = load_config();
            if config.accounts.is_empty() {
                println!("{}", "Error: No accounts found to backup.".red());
                return Ok(());
            }

            println!();
            println!("{}", "Gitcore Vault Backup".cyan().bold());
            println!("{}", "=".repeat(17).cyan());
            println!("This will create a secure, encrypted archive of your Gitcore");
            println!("configuration and all associated private SSH keys.");
            println!();

            let password = prompt_password("  Master Password: ")?;
            if password.is_empty() {
                println!("{}", "Error: Password cannot be empty.".red());
                return Ok(());
            }

            let confirm_password = prompt_password("  Confirm Password: ")?;
            if password != confirm_password {
                println!("{}", "Error: Passwords do not match.".red());
                return Ok(());
            }

            println!("\n[*] Collecting keys...");
            let ssh_dir = get_ssh_dir();
            let mut vault_keys = Vec::new();

            for acc in &config.accounts {
                let priv_path = ssh_dir.join(&acc.key_path);
                let pub_path = ssh_dir.join(format!("{}.pub", acc.key_path));

                if priv_path.exists() {
                    let priv_content = fs::read_to_string(&priv_path)?;
                    let pub_content = if pub_path.exists() {
                        fs::read_to_string(&pub_path)?
                    } else {
                        String::new()
                    };

                    vault_keys.push(VaultKey {
                        filename: acc.key_path.clone(),
                        private_content: priv_content,
                        public_content: pub_content,
                    });
                    println!("  + {}", acc.key_path);
                } else {
                    println!("  ! Key not found: {}", acc.key_path.yellow());
                }
            }

            let vault = Vault {
                config,
                keys: vault_keys,
            };

            println!("[*] Encrypting vault...");
            let encrypted_data = encrypt_vault(vault, &password)?;

            let mut output_path = file.unwrap_or_else(|| "gitcore_backup.gitcore".to_string());
            if !output_path.contains('.') {
                output_path.push_str(".gitcore");
            }
            fs::write(&output_path, encrypted_data)?;

            let full_path = std::env::current_dir()?.join(&output_path);

            println!();
            println!("{}", "Success: Vault created".green().bold());
            println!("Path: {}", full_path.display());
            println!("Note: Keep this file and your password in a safe place.");
            println!();
        }

        Commands::Restore { file } => {
            let input_path = if let Some(path) = file {
                path
            } else {
                // Smart Picker logic
                let mut backups = Vec::new();
                if let Ok(entries) = fs::read_dir(".") {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            let name = path.file_name().unwrap_or_default().to_string_lossy();
                            if name.ends_with(".gitcore") || name.ends_with(".json") {
                                backups.push(name.to_string());
                            }
                        }
                    }
                }

                if backups.is_empty() {
                    prompt_input("Enter path to backup file: ")?
                } else if backups.len() == 1 {
                    println!("[*] Found backup file: {}", backups[0].cyan().bold());
                    backups[0].clone()
                } else {
                    println!();
                    match crate::ui::select_file(&backups, "Select a backup to restore:")? {
                        Some(idx) => backups[idx].clone(),
                        None => {
                            println!("{}", "Cancelled".yellow());
                            return Ok(());
                        }
                    }
                }
            };

            let mut path = PathBuf::from(&input_path);

            if !path.exists() && !input_path.contains('.') {
                let with_ext = format!("{}.gitcore", input_path);
                let path_with_ext = PathBuf::from(&with_ext);
                if path_with_ext.exists() {
                    path = path_with_ext;
                }
            }

            if !path.exists() {
                println!("\nError: File not found: {}", path.display());
                return Ok(());
            }

            let final_path_str = path.to_string_lossy().to_string();

            if final_path_str.ends_with(".json") {
                println!("\n[*] Importing legacy JSON config...");
                let content = fs::read_to_string(&path)?;
                let config: GitcoreConfig = serde_json::from_str(&content)
                    .map_err(|e| io::Error::other(format!("Invalid JSON: {}", e)))?;

                validate_accounts(&config.accounts).map_err(io::Error::other)?;
                save_config(&config)?;
                update_ssh_config(&config.accounts)?;
                println!(
                    "{} Imported {} accounts successfully!",
                    "[v]".green().bold(),
                    config.accounts.len()
                );
                return Ok(());
            }

            println!();
            println!("{}", "Gitcore Vault Restore".cyan().bold());
            println!("{}", "=".repeat(18).cyan());
            let password = prompt_password("  Master Password: ")?;

            let data = fs::read(&path)?;
            let vault = match decrypt_vault(&data, &password) {
                Ok(v) => v,
                Err(e) => {
                    println!("\nError: {}", e);
                    return Ok(());
                }
            };

            println!("\n[*] Unpacking vault...");
            validate_accounts(&vault.config.accounts).map_err(io::Error::other)?;

            save_config(&vault.config)?;

            println!("[*] Restoring SSH keys...");
            let ssh_dir = get_ssh_dir();
            fs::create_dir_all(&ssh_dir)?;

            for vkey in vault.keys {
                let priv_path = ssh_dir.join(&vkey.filename);
                let pub_path = ssh_dir.join(format!("{}.pub", vkey.filename));

                fs::write(&priv_path, &vkey.private_content)?;
                if !vkey.public_content.is_empty() {
                    fs::write(&pub_path, &vkey.public_content)?;
                }

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&priv_path, fs::Permissions::from_mode(0o600));
                    if pub_path.exists() {
                        let _ = fs::set_permissions(&pub_path, fs::Permissions::from_mode(0o644));
                    }
                }
                println!("  + {}", vkey.filename);
            }

            update_ssh_config(&vault.config.accounts)?;

            println!();
            println!("{}", "Success: Vault restored successfully".green().bold());
            println!("Restored: {} account(s)", vault.config.accounts.len());
            println!();
        }

        Commands::Remove { name } => {
            let mut config = load_config();
            if config.accounts.is_empty() {
                println!("{}", "No accounts to remove".red());
                return Ok(());
            }

            let target_name = if let Some(n) = name {
                if config.accounts.iter().any(|a| a.name == n) {
                    Some(n)
                } else {
                    println!("{}", format!("Account '{}' not found", n).red());
                    return Ok(());
                }
            } else {
                let Some(choice) = select_account(&config.accounts, "Select account to remove:")?
                else {
                    println!("{}", "Invalid selection".red());
                    return Ok(());
                };
                Some(config.accounts[choice].name.clone())
            };

            if let Some(n) = target_name {
                if !confirm(&format!(
                    "{} Are you sure? This will remove the account from gitcore [y/N]: ",
                    "[!]".yellow()
                ))? {
                    println!("{}", "Cancelled".yellow());
                    return Ok(());
                }

                config.accounts.retain(|a| a.name != n);
                save_config(&config)?;
                update_ssh_config(&config.accounts)?;

                println!("{}", format!("[v] Account '{}' removed", n).green());
                if confirm("Delete SSH key files too? [y/N]: ")? {
                    match delete_account_keys(&n) {
                        Ok(paths) if paths.is_empty() => {
                            println!("{}", "  No SSH key files were found to delete".yellow());
                        }
                        Ok(paths) => {
                            println!("{}", "  Deleted SSH key files:".green());
                            for path in paths {
                                println!("  - {}", path.display());
                            }
                        }
                        Err(err) => {
                            println!("{}", "  Failed to delete SSH key files".red());
                            println!("   {}", err);
                        }
                    }
                } else {
                    println!("{}", "  SSH key files were left untouched".yellow());
                }
            }
        }

        Commands::Audit => {
            println!();
            println!("{}", "Security Audit".cyan().bold());
            println!("{}", "=".repeat(14).cyan());
            println!();

            let ssh_dir = get_ssh_dir();
            let config_path = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("~/.config"))
                .join("gitcore")
                .join("config.json");

            println!("{}", "SSH Keys".yellow().bold());
            let config = load_config();
            for acc in &config.accounts {
                let key_path = ssh_dir.join(&acc.key_path);
                let pub_key_path = ssh_dir.join(format!("{}.pub", acc.key_path));

                let key_perms = if key_path.exists() {
                    get_permissions(&key_path)
                } else {
                    println!("  [!] {} - key file not found", acc.key_path);
                    continue;
                };

                if key_perms == 0o600 {
                    println!("  [v] {} - OK ({:o})", acc.key_path, key_perms);
                } else {
                    println!(
                        "  [x] {} - WRONG ({:o}) should be 600",
                        acc.key_path, key_perms
                    );
                }

                if pub_key_path.exists() {
                    let pub_perms = get_permissions(&pub_key_path);
                    println!("      {} (pub) - {:o}", acc.key_path, pub_perms);
                }
            }
            println!();

            println!("{}", "SSH Config".yellow().bold());
            let ssh_config_path = ssh_dir.join("config");
            if ssh_config_path.exists() {
                let perms = get_permissions(&ssh_config_path);
                if perms == 0o600 {
                    println!("  [v] config - OK ({:o})", perms);
                } else {
                    println!("  [x] config - WRONG ({:o}) should be 600", perms);
                }
            } else {
                println!("  [!] config - not found");
            }
            println!();

            println!("{}", "Gitcore Config".yellow().bold());
            if config_path.exists() {
                let perms = get_permissions(&config_path);
                if perms == 0o600 {
                    println!("  [v] config.json - OK ({:o})", perms);
                } else {
                    println!("  [x] config.json - WRONG ({:o}) should be 600", perms);
                }
            } else {
                println!("  [!] config.json - not found");
            }
            println!();

            let issues = check_issues(&config, &ssh_dir);
            if issues.is_empty() {
                println!("{}", "All checks passed successfully.".green());
            } else {
                println!("{}", "Issues found:".red());
                for issue in &issues {
                    println!("  - {}", issue);
                }
            }
        }

        Commands::Rotate { name } => {
            let config = load_config();
            if config.accounts.is_empty() {
                println!("{}", "Error: No accounts to rotate".red());
                return Ok(());
            }

            let target_name = if let Some(n) = name {
                if config.accounts.iter().any(|a| a.name == n) {
                    Some(n)
                } else {
                    println!("{}", format!("Account '{}' not found", n).red());
                    return Ok(());
                }
            } else {
                let Some(choice) = select_account(&config.accounts, "Select account to rotate:")?
                else {
                    println!("{}", "Invalid selection".red());
                    return Ok(());
                };
                Some(config.accounts[choice].name.clone())
            };

            if let Some(n) = target_name {
                let acc_idx = config.accounts.iter().position(|a| a.name == n).unwrap();
                let acc = &config.accounts[acc_idx];

                if !confirm(&format!(
                    "{} This will delete the old SSH key and generate a new one. Continue? [y/N]: ",
                    "Warning:".yellow()
                ))? {
                    println!("{}", "Cancelled".yellow());
                    return Ok(());
                }

                println!("{}", "[*] Generating new SSH key...".cyan());

                let passphrase = prompt_password(
                    "Enter passphrase for new SSH key (leave empty for no protection): ",
                )?;

                match delete_account_keys(&n) {
                    Ok(paths) => {
                        for path in paths {
                            println!("  Removed: {}", path.display());
                        }
                    }
                    Err(e) => {
                        println!("Error: Failed to delete old key: {}", e);
                    }
                }

                let pub_key = generate_ssh_key(&acc.key_path, &acc.email, &passphrase)?;

                println!("\n{}", "Success: Key rotated successfully".green().bold());
                println!();
                println!(
                    "{}",
                    "Important: Update your old SSH key on the platform:".yellow()
                );
                println!();
                println!("  {}", pub_key.cyan());
                println!();
                println!("  Open: {}", provider_key_url(&acc.platform).cyan());
                println!();
                println!("The old key has been revoked. Only the new key works now.");
            }
        }
    }

    Ok(())
}

fn handle_add_account(name: Option<String>, platform: Option<String>) -> io::Result<()> {
    let name = if let Some(n) = name {
        n
    } else {
        prompt_input("Enter account name: ")?
    };

    let platform = if let Some(p) = platform {
        match Platform::from_str(&p) {
            Some(pl) => pl,
            None => {
                eprintln!(
                    "{}",
                    "Invalid platform. Use: github, gitlab, codeberg, bitbucket".red()
                );
                return Ok(());
            }
        }
    } else {
        match Platform::from_str(&prompt_input(
            "Enter platform (github/gitlab/codeberg/bitbucket): ",
        )?) {
            Some(pl) => pl,
            None => {
                eprintln!(
                    "{}",
                    "Invalid platform. Use: github, gitlab, codeberg, bitbucket".red()
                );
                return Ok(());
            }
        }
    };

    let username = prompt_input("Enter your git username (for commits): ")?;
    let email = prompt_input("Enter your email (for SSH key + commits): ")?;
    let passphrase =
        prompt_password("Enter passphrase for SSH key (leave empty for no protection): ")?;

    let key_path = format!("id_ed25519_{}", name);
    let host_alias = format!("{}-{}", platform.host().split('.').next().unwrap(), name);

    let mut config = load_config();

    if !is_valid_account_name(&name) {
        eprintln!(
            "{} Account name must use only letters, numbers, '-' or '_'.",
            "[x]".red()
        );
        return Ok(());
    }

    if username.is_empty() {
        eprintln!("{}", "[x] Username cannot be empty.".red());
        return Ok(());
    }

    if email.is_empty() {
        eprintln!("{}", "[x] Email cannot be empty.".red());
        return Ok(());
    }

    if config
        .accounts
        .iter()
        .any(|a| a.name.eq_ignore_ascii_case(&name))
    {
        eprintln!(
            "{} Account '{}' already exists. Use 'gitcore remove {}' first.",
            "[x]".red(),
            name,
            name
        );
        return Ok(());
    }

    if config
        .accounts
        .iter()
        .any(|a| a.host_alias.eq_ignore_ascii_case(&host_alias))
    {
        eprintln!(
            "{} Host alias '{}' already exists. Choose a different account name.",
            "[x]".red(),
            host_alias
        );
        return Ok(());
    }

    let pub_key = generate_ssh_key(&key_path, &email, &passphrase)?;

    let mut gpg_key_id = None;
    if confirm("\nWould you like to associate a GPG signing key with this account? [y/N]: ")? {
        let gpg_keys = crate::gpg::list_gpg_keys()?;
        if gpg_keys.is_empty() {
            println!("{}", "No GPG secret keys found on your system.".yellow());
            println!("You can generate one later with: gpg --full-generate-key");
        } else {
            match crate::ui::select_gpg_key(&gpg_keys, "Select a GPG key to use for signing:")? {
                Some(idx) => {
                    gpg_key_id = Some(gpg_keys[idx].id.clone());
                    println!("Associated GPG key: {}", gpg_keys[idx].id.cyan());
                }
                None => {
                    println!("{}", "No GPG key associated.".yellow());
                }
            }
        }
    }

    let account = Account {
        name: name.clone(),
        platform: platform.clone(),
        key_path,
        host_alias: host_alias.clone(),
        username: username.to_string(),
        email: email.to_string(),
        gpg_key_id,
    };

    config.accounts.push(account);
    save_config(&config)?;
    update_ssh_config(&config.accounts)?;

    println!("\n{}", "Success: Account added successfully".green().bold());
    println!();
    println!("  Name:     {}", name.bold());
    println!("  Platform: {:?}", platform);
    println!("  Use:      git clone git@{}:user/repo.git", host_alias);
    println!();
    println!("{}", "Next steps:".yellow().bold());
    println!();
    println!("  1. Add your SSH public key to your platform:");
    println!("     {}", pub_key.cyan());
    println!();
    println!("     Open: {}", provider_key_url(&platform).cyan());
    println!();
    println!("  2. Test your connection:");
    println!("     Run: {}", format!("gitcore test {}", host_alias).cyan());
    println!();
    println!("  3. Start using it:");
    println!(
        "     Clone:  git clone git@{}:username/repo.git",
        host_alias
    );
    println!("     Remote: gitcore remote add");
    println!();
    Ok(())
}

fn get_permissions(path: &PathBuf) -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o777)
            .unwrap_or(0)
    }
    #[cfg(not(unix))]
    {
        0
    }
}

#[allow(clippy::ptr_arg)]
fn check_issues(config: &GitcoreConfig, ssh_dir: &PathBuf) -> Vec<String> {
    let mut issues = Vec::new();

    for acc in &config.accounts {
        let key_path = ssh_dir.join(&acc.key_path);
        if !key_path.exists() {
            issues.push(format!("SSH key missing: {}", acc.key_path));
        }
    }

    let ssh_config = ssh_dir.join("config");
    if !ssh_config.exists() {
        issues.push("SSH config file missing".to_string());
    }

    issues
}
