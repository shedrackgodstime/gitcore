use crate::models::Account;
use colored::Colorize;
use std::io::{self, Write};

pub fn prompt_input(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub fn confirm(prompt: &str) -> io::Result<bool> {
    let answer = prompt_input(prompt)?;
    Ok(answer.eq_ignore_ascii_case("y"))
}

pub fn prompt_password(prompt: &str) -> io::Result<String> {
    let password = rpassword::prompt_password(prompt)?;
    Ok(password)
}

pub fn select_account(accounts: &[Account], title: &str) -> io::Result<Option<usize>> {
    println!("{}", title.cyan());
    for (i, acc) in accounts.iter().enumerate() {
        println!(
            "  [{}] {} ({} - {:?})",
            i + 1,
            acc.name,
            acc.username,
            acc.platform
        );
    }

    let input = prompt_input(&format!("\nEnter number (1-{}): ", accounts.len()))?;
    let choice: usize = input.trim().parse().unwrap_or(0);
    if choice == 0 || choice > accounts.len() {
        Ok(None)
    } else {
        Ok(Some(choice - 1))
    }
}

pub fn select_file(files: &[String], title: &str) -> io::Result<Option<usize>> {
    println!("{}", title.cyan());
    for (i, file) in files.iter().enumerate() {
        println!("  [{}] {}", i + 1, file);
    }

    let input = prompt_input(&format!("\nSelect backup file (1-{}): ", files.len()))?;
    let choice: usize = input.trim().parse().unwrap_or(0);
    if choice == 0 || choice > files.len() {
        Ok(None)
    } else {
        Ok(Some(choice - 1))
    }
}

pub fn select_gpg_key(keys: &[crate::gpg::GpgKey], title: &str) -> io::Result<Option<usize>> {
    println!("{}", title.cyan());
    for (i, key) in keys.iter().enumerate() {
        println!("  [{}] {} ({}) - {}", i + 1, key.name, key.email, key.id);
    }

    let input = prompt_input(&format!("\nSelect GPG key (1-{}): ", keys.len()))?;
    let choice: usize = input.trim().parse().unwrap_or(0);
    if choice == 0 || choice > keys.len() {
        Ok(None)
    } else {
        Ok(Some(choice - 1))
    }
}

pub fn print_result(remote: &str, username: &str, email: &str) {
    println!();
    println!("{}", "Success: Operation completed".green().bold());
    println!("  Remote:  {}", remote.green());
    println!("  User:    {}", username);
    println!("  Email:   {}", email);
}
