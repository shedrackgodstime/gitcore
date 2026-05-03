use std::io;
use std::process::Command;

pub struct GpgKey {
    pub id: String,
    pub name: String,
    pub email: String,
}

pub fn list_gpg_keys() -> io::Result<Vec<GpgKey>> {
    let output = match Command::new("gpg")
        .args(["--list-secret-keys", "--keyid-format", "LONG"])
        .output()
    {
        Ok(out) => out,
        Err(_) => return Ok(Vec::new()), // GPG not installed or not in PATH
    };

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let content = String::from_utf8_lossy(&output.stdout);
    let mut keys = Vec::new();

    // Simple parser for gpg output
    // Looking for: sec   rsa4096/ABCDEF1234567890 2023-01-01 [SC]
    // Followed by: uid           [ultimate] Name <email@example.com>

    let mut current_id = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("sec") {
            if let Some(pos) = line.find('/') {
                let parts: Vec<&str> = line[pos + 1..].split_whitespace().collect();
                if !parts.is_empty() {
                    current_id = parts[0].to_string();
                }
            }
        } else if line.starts_with("uid") && !current_id.is_empty() {
            let identity = match line.find(']').map(|pos| line[pos + 1..].trim()) {
                Some(id) => id,
                None => continue,
            };

            if let (Some(email_start), Some(email_end)) = (identity.find('<'), identity.find('>')) {
                let name = identity[..email_start].trim().to_string();
                let email = identity[email_start + 1..email_end].trim().to_string();

                keys.push(GpgKey {
                    id: current_id.clone(),
                    name,
                    email,
                });
                current_id = String::new();
            }
        }
    }

    Ok(keys)
}
