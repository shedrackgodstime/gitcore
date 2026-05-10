use crate::cli::{Cli, Commands, RemoteCommands};
use crate::ui::{confirm, print_result, prompt_input, prompt_password, select_account};
use colored::Colorize;
use gitcore::{Gitcore, HostKeyStatus, UpdateAccountRequest};
use std::fs;
use std::io::{self};
use std::path::PathBuf;

pub fn run(cli: Cli) -> io::Result<()> {
    let service = Gitcore::new();

    match cli.command {
        Commands::Add { name, platform } => {
            handle_add_account(name, platform)?;
        }

        Commands::List => {
            let config = service.load_config().map_err(io::Error::other)?;
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
            let config = service.load_config().map_err(io::Error::other)?;
            if config.accounts.is_empty() {
                println!("{}", "No accounts configured.".yellow());
                if confirm("Would you like to add an account now? [y/N]: ")? {
                    handle_add_account(None, None)?;
                }
                return Ok(());
            }

            let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

            if let Some(url) = repo.clone()
                && let Some(stripped) = url.strip_prefix("git@")
                && let Some(colon_pos) = stripped.find(':')
            {
                let alias = &stripped[..colon_pos];
                if let Ok(account) = service.find_account_by_host_alias(alias) {
                    let report = match service.clone_repository(gitcore::CloneRequest {
                        account_name: account.name.clone(),
                        repo_url: url,
                        working_dir,
                    }) {
                        Ok(report) => report,
                        Err(err) => {
                            println!("{}", "[x] Clone failed".red());
                            println!("   {}", err);
                            return Ok(());
                        }
                    };

                    println!(
                        "{}",
                        format!("  Using account: {} ({})", account.name, account.host_alias)
                            .cyan()
                    );
                    println!("  Cloning: {}", report.remote_url);
                    println!("{}", "[v] Cloned with git config:".green());
                    println!("  user.name  = {}", report.username);
                    println!("  user.email = {}", report.email);
                    return Ok(());
                }
            }

            let Some(choice) = select_account(&config.accounts, "Select account to use:")? else {
                println!("{}", "Invalid selection".red());
                return Ok(());
            };

            let acc = &config.accounts[choice];
            let repo_url =
                repo.unwrap_or_else(|| prompt_input("Enter repository URL: ").unwrap_or_default());
            let report = match service.clone_repository(gitcore::CloneRequest {
                account_name: acc.name.clone(),
                repo_url,
                working_dir,
            }) {
                Ok(report) => report,
                Err(err) => {
                    println!("{}", "[x] Clone failed".red());
                    println!("   {}", err);
                    return Ok(());
                }
            };

            println!("\n{}", "Cloning with account:".yellow());
            println!("  Account: {}", acc.name);
            println!("  Email:   {}", acc.email);
            println!("  Remote:  {}", report.remote_url);
            if report.reused_existing_repo {
                println!(
                    "{}",
                    "Directory already exists. Using existing repo.".yellow()
                );
            }
            println!("{}", "[v] Cloned and git config set:".green());
            println!("  user.name  = {}", report.username);
            println!("  user.email = {}", report.email);
        }

        Commands::Test { name } => {
            let config = service.load_config().map_err(io::Error::other)?;
            if config.accounts.is_empty() {
                println!(
                    "{}",
                    "No accounts configured. Run 'gitcore add' first".red()
                );
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
                let report = match service.test_ssh_account(&acc.name) {
                    Ok(report) => report,
                    Err(err) => {
                        println!("{} {}", "[x] Error:".red(), err);
                        return Ok(());
                    }
                };

                println!(
                    "\n{}",
                    format!(
                        "Testing connection to {}...",
                        report.account.platform.host()
                    )
                    .cyan()
                );

                match report.host_status {
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

                if report.authenticated {
                    println!("{}", "[v] Connection successful!".green());
                    println!("   {}", report.stderr.trim());
                } else if report.connected_without_shell {
                    println!("{}", "[v] Connected (but no shell access)".green());
                } else {
                    println!("{}", "[x] Connection failed".red());
                    let detail = report.stderr.trim();
                    if detail.is_empty() {
                        println!("   exit status: {}", report.status);
                    } else {
                        println!("   {}", detail);
                        if detail.contains("Permission denied") {
                            println!();
                            println!("{}", "  HINT: HINT: If your key has a passphrase, SSH might be failing because".yellow());
                            println!("{}", "     it cannot ask for it in batch mode.".yellow());
                            println!(
                                "{}",
                                "     Try adding your key to the agent first:".yellow()
                            );
                            println!("     ssh-add ~/.ssh/{}", report.account.key_path);
                        }
                    }
                }
            }
        }

        Commands::Remote { subcommand } => match subcommand {
            RemoteCommands::Add { repo_url } => {
                let config = service.load_config().map_err(io::Error::other)?;
                if config.accounts.is_empty() {
                    println!("{}", "No accounts configured.".yellow());
                    if confirm("Would you like to add an account now? [y/N]: ")? {
                        handle_add_account(None, None)?;
                    }
                    return Ok(());
                }

                let repo_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

                if let Some(url) = repo_url.clone()
                    && let Some(stripped) = url.strip_prefix("git@")
                    && let Some(colon_pos) = stripped.find(':')
                {
                    let alias = &stripped[..colon_pos];
                    if let Ok(account) = service.find_account_by_host_alias(alias) {
                        let report = match service.add_remote(gitcore::RemoteAddRequest {
                            account_name: account.name.clone(),
                            repo_url: url.clone(),
                            repo_path: repo_path.clone(),
                        }) {
                            Ok(report) => report,
                            Err(err) => {
                                if let gitcore::GitcoreError::NotGitRepository(ref path) = err {
                                    println!("{}", "[x] Failed to configure origin remote".red());
                                    println!("   {}", err);
                                    println!();
                                    if confirm(&format!(
                                        "Would you like to initialize a new Git repository in {}?",
                                        path.display().to_string().cyan()
                                    ))? {
                                        service.init_git_repo(path).map_err(io::Error::other)?;
                                        match service.add_remote(gitcore::RemoteAddRequest {
                                            account_name: account.name.clone(),
                                            repo_url: url,
                                            repo_path,
                                        }) {
                                            Ok(report) => report,
                                            Err(err) => {
                                                println!(
                                                    "{}",
                                                    "[x] Failed to configure origin remote after initialization"
                                                        .red()
                                                );
                                                println!("   {}", err);
                                                return Ok(());
                                            }
                                        }
                                    } else {
                                        return Ok(());
                                    }
                                } else {
                                    println!("{}", "[x] Failed to configure origin remote".red());
                                    println!("   {}", err);
                                    return Ok(());
                                }
                            }
                        };
                        print_result(&report.remote_url, &report.username, &report.email);
                        return Ok(());
                    }
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
                let report = match service.add_remote(gitcore::RemoteAddRequest {
                    account_name: selected_acc.name.clone(),
                    repo_url: final_repo.clone(),
                    repo_path: repo_path.clone(),
                }) {
                    Ok(report) => report,
                    Err(err) => {
                        if let gitcore::GitcoreError::NotGitRepository(ref path) = err {
                            println!("{}", "[x] Failed to configure origin remote".red());
                            println!("   {}", err);
                            println!();
                            if confirm(&format!(
                                "Would you like to initialize a new Git repository in {}?",
                                path.display().to_string().cyan()
                            ))? {
                                service.init_git_repo(path).map_err(io::Error::other)?;
                                match service.add_remote(gitcore::RemoteAddRequest {
                                    account_name: selected_acc.name.clone(),
                                    repo_url: final_repo,
                                    repo_path,
                                }) {
                                    Ok(report) => report,
                                    Err(err) => {
                                        println!(
                                            "{}",
                                            "[x] Failed to configure origin remote after initialization"
                                                .red()
                                        );
                                        println!("   {}", err);
                                        return Ok(());
                                    }
                                }
                            } else {
                                return Ok(());
                            }
                        } else {
                            println!("{}", "[x] Failed to configure origin remote".red());
                            println!("   {}", err);
                            return Ok(());
                        }
                    }
                };
                print_result(&report.remote_url, &report.username, &report.email);
            }

            RemoteCommands::Switch { account } => {
                let config = service.load_config().map_err(io::Error::other)?;
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
                    let report = match service.switch_remote(gitcore::RemoteSwitchRequest {
                        account_name: acc.name.clone(),
                        repo_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
                    }) {
                        Ok(report) => report,
                        Err(err) => {
                            println!("{}", "[x] Failed to switch remote".red());
                            println!("   {}", err);
                            return Ok(());
                        }
                    };
                    println!("{}", "[v] Remote switched".green());
                    println!("  origin     = {}", report.remote_url);
                    println!("  user.name  = {}", report.username);
                    println!("  user.email = {}", report.email);
                }
            }
        },

        Commands::Backup { file } => {
            let config = service.load_config().map_err(io::Error::other)?;
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

            let mut output_path = file.unwrap_or_else(|| "gitcore_backup.gitcore".to_string());
            if !output_path.contains('.') {
                output_path.push_str(".gitcore");
            }
            println!("\n[*] Collecting keys and encrypting vault...");
            let backup_path = PathBuf::from(&output_path);
            let report = service
                .backup_to_path(&backup_path, &password)
                .map_err(io::Error::other)?;

            for key in &report.included_keys {
                println!("  + {}", key);
            }
            for key in &report.missing_keys {
                println!("  ! Key not found: {}", key.yellow());
            }

            let full_path = std::env::current_dir()?.join(&report.output_path);

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
                let report = service
                    .restore_from_path(&path, None)
                    .map_err(io::Error::other)?;
                println!(
                    "{} Imported {} accounts successfully!",
                    "[v]".green().bold(),
                    report.restored_accounts
                );
                return Ok(());
            }

            println!();
            println!("{}", "Gitcore Vault Restore".cyan().bold());
            println!("{}", "=".repeat(18).cyan());
            let password = prompt_password("  Master Password: ")?;
            println!("\n[*] Unpacking vault...");
            let report = match service.restore_from_path(&path, Some(&password)) {
                Ok(report) => report,
                Err(err) => {
                    println!("\nError: {}", err);
                    return Ok(());
                }
            };

            println!("[*] Restoring SSH keys...");
            for key in &report.restored_keys {
                println!("  + {}", key);
            }

            println!();
            println!("{}", "Success: Vault restored successfully".green().bold());
            println!("Restored: {} account(s)", report.restored_accounts);
            println!();
        }

        Commands::Remove { name } => {
            let config = service.load_config().map_err(io::Error::other)?;
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

                let removed_account = service.remove_account(&n).map_err(io::Error::other)?;

                println!("{}", format!("[v] Account '{}' removed", n).green());
                if confirm("Delete SSH key files too? [y/N]: ")? {
                    match service.delete_account_key_files(&removed_account.key_path) {
                        Ok(report) if report.deleted_paths.is_empty() => {
                            println!("{}", "  No SSH key files were found to delete".yellow());
                        }
                        Ok(report) => {
                            println!("{}", "  Deleted SSH key files:".green());
                            for path in report.deleted_paths {
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

            println!("{}", "SSH Keys".yellow().bold());
            let report = service.audit().map_err(io::Error::other)?;
            for key_audit in &report.key_audits {
                if !key_audit.private_key.exists {
                    println!("  [!] {} - key file not found", key_audit.account.key_path);
                    continue;
                }

                let key_perms = key_audit.private_key.permissions.unwrap_or(0);
                if key_perms == key_audit.private_key.expected_permissions {
                    println!(
                        "  [v] {} - OK ({:o})",
                        key_audit.account.key_path, key_perms
                    );
                } else {
                    println!(
                        "  [x] {} - WRONG ({:o}) should be {:o}",
                        key_audit.account.key_path,
                        key_perms,
                        key_audit.private_key.expected_permissions
                    );
                }

                if key_audit.public_key.exists {
                    let pub_perms = key_audit.public_key.permissions.unwrap_or(0);
                    println!(
                        "      {} (pub) - {:o}",
                        key_audit.account.key_path, pub_perms
                    );
                }
            }
            println!();

            println!("{}", "SSH Config".yellow().bold());
            if report.ssh_config.exists {
                let perms = report.ssh_config.permissions.unwrap_or(0);
                if perms == report.ssh_config.expected_permissions {
                    println!("  [v] config - OK ({:o})", perms);
                } else {
                    println!(
                        "  [x] config - WRONG ({:o}) should be {:o}",
                        perms, report.ssh_config.expected_permissions
                    );
                }
            } else {
                println!("  [!] config - not found");
            }
            println!();

            println!("{}", "Gitcore Config".yellow().bold());
            if report.config_file.exists {
                let perms = report.config_file.permissions.unwrap_or(0);
                if perms == report.config_file.expected_permissions {
                    println!("  [v] config.json - OK ({:o})", perms);
                } else {
                    println!(
                        "  [x] config.json - WRONG ({:o}) should be {:o}",
                        perms, report.config_file.expected_permissions
                    );
                }
            } else {
                println!("  [!] config.json - not found");
            }
            println!();

            if report.issues.is_empty() {
                println!("{}", "All checks passed successfully.".green());
            } else {
                println!("{}", "Issues found:".red());
                for issue in &report.issues {
                    println!("  - {}", issue);
                }
            }
        }

        Commands::Rotate { name } => {
            let config = service.load_config().map_err(io::Error::other)?;
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

                let report = match service.rotate_key(&n, &passphrase) {
                    Ok(report) => report,
                    Err(err) => {
                        println!("Error: Failed to rotate key: {}", err);
                        return Ok(());
                    }
                };
                for path in &report.deleted_paths {
                    println!("  Removed: {}", path.display());
                }

                println!("\n{}", "Success: Key rotated successfully".green().bold());
                println!();
                println!(
                    "{}",
                    "Important: Update your old SSH key on the platform:".yellow()
                );
                println!();
                println!("  {}", report.public_key.cyan());
                println!();
                println!(
                    "  Open: {}",
                    report.account.platform.provider_key_url().cyan()
                );
                println!();
                println!("The old key has been revoked. Only the new key works now.");
            }
        }

        Commands::Update { name } => {
            handle_update_account(name)?;
        }

        Commands::Whoami => {
            let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            match service.detect_account_in_repository(&working_dir) {
                Ok(Some(account)) => {
                    println!();
                    println!("  {} {}", "Active account:".cyan(), account.name.bold());
                    println!("  {}    {}", "Platform:".cyan(), account.platform.host());
                    println!("  {}    {}", "User:".cyan(), account.username);
                    println!("  {}   {}", "Email:".cyan(), account.email);
                    if let Some(gpg) = account.gpg_key_id {
                        println!("  {}     {}", "GPG Key:".cyan(), gpg);
                    }
                    println!();
                }
                Ok(None) => {
                    println!(
                        "{}",
                        "  No managed Gitcore account detected in this directory.".yellow()
                    );
                }
                Err(err) => {
                    println!("{} {}", "[x] Error:".red(), err);
                }
            }
        }
    }

    Ok(())
}

fn handle_update_account(name: Option<String>) -> io::Result<()> {
    let service = Gitcore::new();
    let config = service.load_config().map_err(io::Error::other)?;
    if config.accounts.is_empty() {
        println!("{}", "Error: No accounts to update".red());
        return Ok(());
    }

    let target_name = if let Some(n) = name {
        if config.accounts.iter().any(|a| a.name == n) {
            n
        } else {
            println!("{}", format!("Account '{}' not found", n).red());
            return Ok(());
        }
    } else {
        let Some(choice) = select_account(&config.accounts, "Select account to update:")? else {
            println!("{}", "Invalid selection".red());
            return Ok(());
        };
        config.accounts[choice].name.clone()
    };

    let current = config
        .accounts
        .iter()
        .find(|a| a.name == target_name)
        .unwrap();
    println!(
        "\n{}",
        format!("Updating account: {}", target_name).cyan().bold()
    );
    println!("Leave blank to keep current value.\n");

    let username = prompt_input(&format!("Username [{}]: ", current.username))?;
    let email = prompt_input(&format!("Email [{}]: ", current.email))?;

    let mut update = UpdateAccountRequest::default();
    if !username.is_empty() {
        update.username = Some(username);
    }
    if !email.is_empty() {
        update.email = Some(email);
    }

    if confirm("\nModify GPG signing key? [y/N]: ")? {
        let gpg_keys = gitcore::list_gpg_keys()?;
        if gpg_keys.is_empty() {
            println!("{}", "No GPG secret keys found.".yellow());
            if confirm("Clear current GPG key association? [y/N]: ")? {
                update.gpg_key_id = Some(None);
            }
        } else {
            println!(
                "Current: {}",
                current.gpg_key_id.as_deref().unwrap_or("None")
            );
            match crate::ui::select_gpg_key(&gpg_keys, "Select new GPG key (or ESC to clear):")? {
                Some(idx) => {
                    update.gpg_key_id = Some(Some(gpg_keys[idx].id.clone()));
                }
                None => {
                    if confirm("Clear current GPG key association? [y/N]: ")? {
                        update.gpg_key_id = Some(None);
                    }
                }
            }
        }
    }

    service
        .update_account(&target_name, update)
        .map_err(io::Error::other)?;
    println!("\n{}", "Success: Account updated successfully".green());
    Ok(())
}

fn handle_add_account(name: Option<String>, platform: Option<String>) -> io::Result<()> {
    let service = Gitcore::new();
    let name = if let Some(n) = name {
        n
    } else {
        prompt_input("Enter account name: ")?
    };

    let platform = if let Some(p) = platform {
        match service.parse_platform(&p).ok() {
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
        match service
            .parse_platform(&prompt_input(
                "Enter platform (github/gitlab/codeberg/bitbucket): ",
            )?)
            .ok()
        {
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

    let mut gpg_key_id = None;
    if confirm("\nWould you like to associate a GPG signing key with this account? [y/N]: ")? {
        let gpg_keys = gitcore::list_gpg_keys()?;
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

    let account_request = gitcore::AddAccountRequest {
        name: name.clone(),
        platform,
        username: username.to_string(),
        email: email.to_string(),
        gpg_key_id,
        key_path: None,
    };

    let key_report = service
        .provision_account_keys(&account_request, &passphrase)
        .map_err(io::Error::other)?;

    let registered = match service.register_account(account_request) {
        Ok(registered) => registered,
        Err(err) => {
            eprintln!("{} {}", "[x]".red(), err);
            let _ = service.delete_account_key_files(&key_report.key_path);
            return Ok(());
        }
    };

    println!("\n{}", "Success: Account added successfully".green().bold());
    println!();
    println!("  Name:     {}", name.bold());
    println!("  Platform: {:?}", platform);
    println!(
        "  Use:      git clone git@{}:user/repo.git",
        registered.host_alias
    );
    println!();
    println!("{}", "Next steps:".yellow().bold());
    println!();
    println!("  1. Add your SSH public key to your platform:");
    println!("     {}", key_report.public_key.cyan());
    println!();
    println!("     Open: {}", platform.provider_key_url().cyan());
    println!();
    println!("  2. Test your connection:");
    println!(
        "     Run: {}",
        format!("gitcore test {}", registered.host_alias).cyan()
    );
    println!();
    println!("  3. Start using it:");
    println!(
        "     Clone:  git clone git@{}:username/repo.git",
        registered.host_alias
    );
    println!("     Remote: gitcore remote add");
    println!();
    Ok(())
}
