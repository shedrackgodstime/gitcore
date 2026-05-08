use colored::Colorize;
use gitcore::{Account, GpgKey};
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

fn select_from_list(items: &[String], title: &str, prompt: &str) -> io::Result<Option<usize>> {
    println!("{}", title.cyan());
    for (i, item) in items.iter().enumerate() {
        println!("  [{}] {}", i + 1, item);
    }

    let input = prompt_input(&format!("\n{} (1-{}): ", prompt, items.len()))?;
    let choice: usize = input.trim().parse().unwrap_or(0);
    if choice == 0 || choice > items.len() {
        Ok(None)
    } else {
        Ok(Some(choice - 1))
    }
}

pub fn select_account(accounts: &[Account], title: &str) -> io::Result<Option<usize>> {
    let items: Vec<String> = accounts
        .iter()
        .map(|acc| format!("{} ({} - {:?})", acc.name, acc.username, acc.platform))
        .collect();
    select_from_list(&items, title, "Select number")
}

pub fn select_file(files: &[String], title: &str) -> io::Result<Option<usize>> {
    select_from_list(files, title, "Select backup file")
}

pub fn select_gpg_key(keys: &[GpgKey], title: &str) -> io::Result<Option<usize>> {
    let items: Vec<String> = keys
        .iter()
        .map(|key| format!("{} ({}) - {}", key.name, key.email, key.id))
        .collect();
    select_from_list(&items, title, "Select GPG key")
}

pub fn print_result(remote: &str, username: &str, email: &str) {
    println!();
    println!("{}", "Success: Operation completed".green().bold());
    println!("  Remote:  {}", remote.green());
    println!("  User:    {}", username);
    println!("  Email:   {}", email);
}
