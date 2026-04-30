use crate::cli::{Cli, Commands, RemoteCommands};
use crate::config::{load_config, save_config};
use crate::git::{
    convert_to_host, ensure_git_repository, run_git, run_git_remote_add, set_git_config,
};
use crate::models::{is_valid_account_name, validate_accounts, Account, GityConfig, Platform};
use crate::ssh::{
    check_host_key, delete_account_keys, generate_ssh_key, get_ssh_dir, provider_key_url,
    test_ssh_connection, update_ssh_config, HostKeyStatus,
};
use crate::ui::{confirm, print_result, prompt_input, select_account};
use colored::Colorize;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::Command;

pub fn run(cli: Cli) -> io::Result<()> {
    match cli.command {
        Commands::Add { name, platform } => {
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
                prompt_input("Enter passphrase for SSH key (leave empty for no protection): ")?;

            let key_path = format!("id_ed25519_{}", name);
            let host_alias = format!("{}-{}", platform.host().split('.').next().unwrap(), name);

            let mut config = load_config();

            if !is_valid_account_name(&name) {
                eprintln!(
                    "{} Account name must use only letters, numbers, '-' or '_'.",
                    "✗".red()
                );
                return Ok(());
            }

            if username.is_empty() {
                eprintln!("{}", "✗ Username cannot be empty.".red());
                return Ok(());
            }

            if email.is_empty() {
                eprintln!("{}", "✗ Email cannot be empty.".red());
                return Ok(());
            }

            if config
                .accounts
                .iter()
                .any(|a| a.name.eq_ignore_ascii_case(&name))
            {
                eprintln!(
                    "{} Account '{}' already exists. Use 'gity remove {}' first.",
                    "✗".red(),
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
                    "✗".red(),
                    host_alias
                );
                return Ok(());
            }

            let pub_key = generate_ssh_key(&key_path, &email, &passphrase)?;

            let account = Account {
                name: name.clone(),
                platform: platform.clone(),
                key_path,
                host_alias: host_alias.clone(),
                username: username.to_string(),
                email: email.to_string(),
            };

            config.accounts.push(account);
            save_config(&config)?;
            update_ssh_config(&config.accounts)?;

            println!();
            println!(
                "{}",
                "┌─────────────────────────────────────────────────┐".cyan()
            );
            println!(
                "{}",
                "│           Account Added Successfully!           │".cyan()
            );
            println!(
                "{}",
                "└─────────────────────────────────────────────────┘".cyan()
            );
            println!();
            println!("  Name:     {}", name.bold());
            println!("  Platform: {:?}", platform);
            println!("  Use:      git clone git@{}:user/repo.git", host_alias);
            println!();
            println!("{}", "─".repeat(51));
            println!();
            println!("{}", "  1. ADD SSH KEY TO YOUR PLATFORM".yellow().bold());
            println!();
            println!("  {}", pub_key);
            println!();
            println!("  Open: {}", provider_key_url(&platform).cyan());
            println!();
            println!("{}", "─".repeat(51));
            println!();
            println!("{}", "  2. TEST CONNECTION".yellow().bold());
            println!();
            println!("  Run: {}", format!("gity test {}", host_alias).cyan());
            println!();
            println!("{}", "─".repeat(51));
            println!();
            println!("{}", "  3. USAGE".yellow().bold());
            println!();
            println!("  Clone:  git clone git@{}:username/repo.git", host_alias);
            println!("  Remote: gity remote add");
            println!();
        }

        Commands::List => {
            let config = load_config();
            if config.accounts.is_empty() {
                println!(
                    "{}",
                    "No accounts configured. Run 'gity add <name> <platform>'".yellow()
                );
                return Ok(());
            }

            println!(
                "{}",
                "╔═══════════════════════════════════════════════════╗".cyan()
            );
            println!(
                "{}",
                "║           Configured Git Accounts                 ║".cyan()
            );
            println!(
                "{}",
                "╚═══════════════════════════════════════════════════╝".cyan()
            );
            println!();

            for (i, acc) in config.accounts.iter().enumerate() {
                println!("{} [{}] {}", "●".green(), i + 1, acc.name.bold());
                println!("   Platform: {:?}", acc.platform);
                println!("   Host:     {}", acc.host_alias);
                println!("   Key:      ~/.ssh/{}", acc.key_path);
                println!("   User:     {}", acc.username);
                println!("   Email:    {}", acc.email);
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
                println!("{}", "No accounts. Run 'gity add' first".red());
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
                        println!("{}", "⚠ Directory exists but not a git repository.".red());
                        return Ok(());
                    }
                } else if let Err(err) = run_git(&["clone", &converted]) {
                    println!("{}", "✗ Clone failed".red());
                    println!("   {}", err);
                    return Ok(());
                }

                std::env::set_current_dir(&clone_dir).ok();
                if let Err(err) = set_git_config(&acc.username, &acc.email) {
                    println!(
                        "{}",
                        "✗ Clone succeeded, but git config update failed".red()
                    );
                    println!("   {}", err);
                    return Ok(());
                }
                println!("{}", "✓ Cloned with git config:".green());
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
                    println!("{}", "⚠ Directory exists but not a git repository.".red());
                    return Ok(());
                }
                println!(
                    "{}",
                    "Directory already exists. Using existing repo.".yellow()
                );
                if let Err(err) = run_git_remote_add(&converted) {
                    println!("{}", "✗ Failed to configure origin remote".red());
                    println!("   {}", err);
                    return Ok(());
                }
            } else {
                println!("{}", "Running: git clone".cyan());
                if let Err(err) = run_git(&["clone", &converted]) {
                    println!("{}", "✗ Clone failed".red());
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

            if let Err(err) = set_git_config(&acc.username, &acc.email) {
                println!(
                    "{}",
                    "✗ Clone succeeded, but git config update failed".red()
                );
                println!("   {}", err);
                return Ok(());
            }
            println!("{}", "✓ Cloned and git config set:".green());
            println!("  user.name  = {}", acc.username);
            println!("  user.email = {}", acc.email);
        }

        Commands::Test { name } => {
            let config = load_config();
            if config.accounts.is_empty() {
                println!("{}", "No accounts configured. Run 'gity add' first".red());
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
                        println!("{}", "  ✓ Host key is known".green());
                    }
                    HostKeyStatus::New => {
                        println!(
                            "{}",
                            "  ⚠ New host key - will be added to known_hosts".yellow()
                        );
                    }
                    HostKeyStatus::Unknown => {
                        println!("{}", "  ⚠ No known_hosts file - will be created".yellow());
                    }
                }

                let output = test_ssh_connection(&acc.host_alias);

                match output {
                    Ok(out) => {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        if stderr.contains("successfully authenticated") || stderr.contains("Hi") {
                            println!("{}", "✓ Connection successful!".green());
                            println!("   {}", stderr.trim());
                        } else if out.status.success() {
                            println!("{}", "✓ Connected (but no shell access)".green());
                        } else {
                            println!("{}", "✗ Connection failed".red());
                            let detail = stderr.trim();
                            if detail.is_empty() {
                                println!("   exit status: {}", out.status);
                            } else {
                                println!("   {}", detail);
                            }
                        }
                    }
                    Err(e) => {
                        println!("{} {}", "✗ Error:".red(), e);
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
                    println!("{}", "⚠ Not a git repository. Run 'git init' first.".red());
                    return Ok(());
                }

                let config = load_config();
                if config.accounts.is_empty() {
                    println!("{}", "No accounts. Add one first with 'gity add'".red());
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
                    if let Err(err) = set_git_config(&acc.username, &acc.email) {
                        println!("{}", "✗ Failed to set git config".red());
                        println!("   {}", err);
                        return Ok(());
                    }
                    if let Err(err) = run_git_remote_add(&converted) {
                        println!("{}", "✗ Failed to configure origin remote".red());
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
                if let Err(err) = set_git_config(&selected_acc.username, &selected_acc.email) {
                    println!("{}", "✗ Failed to set git config".red());
                    println!("   {}", err);
                    return Ok(());
                }
                if let Err(err) = run_git_remote_add(&converted) {
                    println!("{}", "✗ Failed to configure origin remote".red());
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
                    println!("{}", "⚠ Not a git repository.".red());
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
                            println!("{}", "✗ Could not read origin remote".red());
                            return Ok(());
                        }
                    };

                    let converted = convert_to_host(&current_remote, &acc.host_alias);
                    if let Err(err) = run_git(&["remote", "set-url", "origin", &converted]) {
                        println!("{}", "✗ Failed to switch remote".red());
                        println!("   {}", err);
                        return Ok(());
                    }

                    if let Err(err) = set_git_config(&acc.username, &acc.email) {
                        println!(
                            "{}",
                            "✗ Remote switched, but git config update failed".red()
                        );
                        println!("   {}", err);
                        return Ok(());
                    }

                    println!("{}", "✓ Remote switched".green());
                    println!("  origin     = {}", converted);
                    println!("  user.name  = {}", acc.username);
                    println!("  user.email = {}", acc.email);
                }
            }
        },

        Commands::Export => {
            let config = load_config();
            let content = serde_json::to_string_pretty(&config)?;
            println!("{}", content);
        }

        Commands::Import { file } => {
            let input = if let Some(path) = file {
                fs::read_to_string(path)?
            } else {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer
            };

            let config: GityConfig = match serde_json::from_str(&input) {
                Ok(cfg) => cfg,
                Err(err) => {
                    eprintln!("{} Invalid JSON: {}", "✗".red(), err);
                    return Ok(());
                }
            };

            if let Err(err) = validate_accounts(&config.accounts) {
                eprintln!("{} Import rejected: {}", "✗".red(), err);
                return Ok(());
            }

            save_config(&config)?;
            update_ssh_config(&config.accounts)?;
            println!(
                "{} Imported {} account(s)",
                "✓".green(),
                config.accounts.len()
            );
            println!(
                "{}",
                "  Note: import restores config only; SSH key files must exist separately."
                    .yellow()
            );
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
                    "{} Are you sure? This will remove the account from gity [y/N]: ",
                    "⚠".yellow()
                ))? {
                    println!("{}", "Cancelled".yellow());
                    return Ok(());
                }

                config.accounts.retain(|a| a.name != n);
                save_config(&config)?;
                update_ssh_config(&config.accounts)?;

                println!("{}", format!("✓ Account '{}' removed", n).green());
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
            println!("{}", "🔍 Security Audit".cyan().bold());
            println!("{}", "═".repeat(40));
            println!();

            let ssh_dir = get_ssh_dir();
            let config_path = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("~/.config"))
                .join("gity")
                .join("config.json");

            println!("{}", "📁 SSH Keys".yellow().bold());
            let config = load_config();
            for acc in &config.accounts {
                let key_path = ssh_dir.join(&acc.key_path);
                let pub_key_path = ssh_dir.join(format!("{}.pub", acc.key_path));

                let key_perms = if key_path.exists() {
                    get_permissions(&key_path)
                } else {
                    println!("  ⚠ {} - key file not found", acc.key_path);
                    continue;
                };

                if key_perms == 0o600 {
                    println!("  ✓ {} - OK ({:o})", acc.key_path, key_perms);
                } else {
                    println!(
                        "  ✗ {} - WRONG ({:o}) should be 600",
                        acc.key_path, key_perms
                    );
                }

                if pub_key_path.exists() {
                    let pub_perms = get_permissions(&pub_key_path);
                    println!("    {} (pub) - {:o}", acc.key_path, pub_perms);
                }
            }
            println!();

            println!("{}", "📁 SSH Config".yellow().bold());
            let ssh_config_path = ssh_dir.join("config");
            if ssh_config_path.exists() {
                let perms = get_permissions(&ssh_config_path);
                if perms == 0o600 {
                    println!("  ✓ config - OK ({:o})", perms);
                } else {
                    println!("  ✗ config - WRONG ({:o}) should be 600", perms);
                }
            } else {
                println!("  ⚠ config - not found");
            }
            println!();

            println!("{}", "📁 Gity Config".yellow().bold());
            if config_path.exists() {
                let perms = get_permissions(&config_path);
                if perms == 0o600 {
                    println!("  ✓ config.json - OK ({:o})", perms);
                } else {
                    println!("  ✗ config.json - WRONG ({:o}) should be 600", perms);
                }
            } else {
                println!("  ⚠ config.json - not found");
            }
            println!();

            let issues = check_issues(&config, &ssh_dir);
            if issues.is_empty() {
                println!("{}", "✅ All checks passed!".green());
            } else {
                println!("{}", "⚠ Issues found:".yellow());
                for issue in &issues {
                    println!("  - {}", issue);
                }
            }
        }

        Commands::Rotate { name } => {
            let config = load_config();
            if config.accounts.is_empty() {
                println!("{}", "No accounts to rotate".red());
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
                    "⚠".yellow()
                ))? {
                    println!("{}", "Cancelled".yellow());
                    return Ok(());
                }

                println!("{}", "Generating new SSH key...".cyan());

                let passphrase = prompt_input(
                    "Enter passphrase for new SSH key (leave empty for no protection): ",
                )?;

                match delete_account_keys(&n) {
                    Ok(paths) => {
                        for path in paths {
                            println!("  Deleted: {}", path.display());
                        }
                    }
                    Err(e) => {
                        println!("{} Failed to delete old key: {}", "✗".red(), e);
                    }
                }

                let pub_key = generate_ssh_key(&acc.key_path, &acc.email, &passphrase)?;

                println!();
                println!(
                    "{}",
                    "┌─────────────────────────────────────────────────┐".cyan()
                );
                println!(
                    "{}",
                    "│           Key Rotated Successfully!              │".cyan()
                );
                println!(
                    "{}",
                    "└─────────────────────────────────────────────────┘".cyan()
                );
                println!();
                println!(
                    "{}",
                    "  IMPORTANT: Update your old SSH key on the platform:"
                        .yellow()
                        .bold()
                );
                println!();
                println!("  {}", pub_key);
                println!();
                println!("  Open: {}", provider_key_url(&acc.platform).cyan());
                println!();
                println!(
                    "{}",
                    "  The old key has been revoked. Only the new key works now.".yellow()
                );
            }
        }
    }

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
fn check_issues(config: &GityConfig, ssh_dir: &PathBuf) -> Vec<String> {
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
