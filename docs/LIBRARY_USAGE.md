# Integrating the Gitcore Identity Engine

Gitcore is architected as a decoupled, high-assurance identity engine. While most users interact with it via the CLI, the underlying logic is exposed as a comprehensive Rust SDK, allowing developers to embed deterministic Git identity management into their own tools, CI/CD pipelines, and automation scripts.

## Installation

Add `gitcore` to your `Cargo.toml` dependencies:

```toml
[dependencies]
gitcore = "1.3.0"
```

## Core Integration Flow

The primary interface is the [`Gitcore`] service, which orchestrates identity registration, SSH configuration, and repository-level metadata injection.

```rust
use gitcore::{Gitcore, AddAccountRequest, Platform};

fn main() -> Result<(), gitcore::GitcoreError> {
    // 1. Initialize the identity engine
    let service = Gitcore::new();

    // 2. Define a new account
    let request = AddAccountRequest {
        name: "work".to_string(),
        platform: Platform::Github,
        username: "octocat".to_string(),
        email: "octocat@example.com".to_string(),
        gpg_key_id: None,
        key_path: None, // Uses default naming: id_ed25519_work
    };

    // 3. Provision keys (generates ssh-keygen pair)
    let key_report = service.provision_account_keys(&request, "secret-passphrase")?;
    println!("Public key to upload: {}", key_report.public_key);

    // 4. Register the account
    service.register_account(request)?;

    Ok(())
}
```

## Repository Operations

Once an account is registered, you can use it to clone or manage repositories.

```rust
use gitcore::{Gitcore, CloneRequest};
use std::path::PathBuf;

fn clone_project(service: &Gitcore) -> Result<(), gitcore::GitcoreError> {
    service.clone_repository(CloneRequest {
        account_name: "work".to_string(),
        repo_url: "github.com/acme/project.git".to_string(),
        working_dir: PathBuf::from("./projects"),
    })?;
    Ok(())
}
```

## Customizing Paths

For testing or specialized environments, you can specify custom configuration and SSH directories:

```rust
use gitcore::{Gitcore, GitcorePaths};
use std::path::PathBuf;

let paths = GitcorePaths {
    config_path: PathBuf::from("/tmp/gitcore/config.json"),
    ssh_dir: PathBuf::from("/tmp/.ssh"),
};
let service = Gitcore::with_paths(paths);
```

## Updating Identities

You can update metadata (like email or GPG key) without removing the account:

```rust
use gitcore::{Gitcore, UpdateAccountRequest};

service.update_account("work", UpdateAccountRequest {
    email: Some("new-email@example.com".to_string()),
    ..Default::default()
})?;
```

## Context Detection

Identify which managed account is active in a local directory:

```rust
if let Some(account) = service.detect_account_in_repository(std::path::Path::new("."))? {
    println!("Active identity: {}", account.name);
}
```

## Error Handling

Gitcore uses the [`GitcoreError`] enum for all library errors. It leverages `thiserror` to provide descriptive error messages and easy programmatic inspection.

```rust
match service.list_accounts() {
    Ok(accounts) => println!("Managed accounts: {}", accounts.len()),
    Err(gitcore::GitcoreError::AccountNotFound(name)) => {
        eprintln!("Account '{}' does not exist", name);
    }
    Err(e) => eprintln!("Library error: {}", e),
}
```
