# Gitcore

**High-assurance Git identity management for multi-tenant developer environments.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/shedrackgodstime/gitcore)](https://github.com/shedrackgodstime/gitcore/releases)

```text
  gitcore list

  Configured Identities
  =====================

  [1] work      (Github)    -> git@github-work:user/repo.git
  [2] personal  (Codeberg)  -> git@codeberg-personal:user/repo.git
```

---

## The Problem: Identity Contamination

Standard Git and OpenSSH configurations are fundamentally designed for a single global identity. For engineers managing disparate personas (Corporate, Personal, Open Source), this often leads to:
*   **Authorship Leakage**: Personal emails appearing in enterprise commits.
*   **Authentication Conflict**: SSH handshake failures due to incorrect key selection.
*   **Credential Sprawl**: Unmanaged, unrotated keys scattered across the filesystem.

Gitcore solves this by introducing a **deterministic identity layer** that cryptographically isolates each account.

---

## Core Capabilities

Gitcore provides a unified engine for managing the entire lifecycle of a Git identity, available as both a high-performance **CLI tool** and a **programmatic Rust SDK**.

*   **Deterministic Isolation**: Every identity is pinned to a unique Ed25519 keypair, preventing cross-contamination during authentication.
*   **Managed SSH Orchestration**: Automatically maintains a non-destructive block in `~/.ssh/config`, enabling seamless usage of standard `git` commands.
*   **Authorship Protection**: Automatically injects correct `user.name`, `user.email`, and GPG signing keys into local repository configurations.
*   **Zero-Friction Context**: Detects the active identity in the current directory with `gitcore whoami`.
*   **Encrypted Portability**: Encapsulates your entire identity environment into a single AES-256-GCM encrypted vault for secure migration.

---

## Installation

### From Crates.io (Recommended)
If you have Rust and Cargo installed, you can install Gitcore directly from [crates.io](https://crates.io/crates/gitcore):

```bash
cargo install gitcore
```

### Script Installation
For systems without a Rust environment:

**Linux & macOS:**
```bash
curl -fsSL https://shedrackgodstime.github.io/gitcore/install | sh
```

**Windows (PowerShell):**
```bash
iwr https://shedrackgodstime.github.io/gitcore/ps | iex
```

### Build from Source
```bash
git clone https://github.com/shedrackgodstime/gitcore.git
cd gitcore
cargo build --release
sudo cp target/release/gitcore /usr/local/bin/gitcore
```

> **Requirements:** OpenSSH, Git.

---

## Quick Start

### 1. Add your accounts

```bash
gitcore add work github
# Prompts for: username, email, SSH passphrase, optional GPG key
```

```bash
gitcore add personal codeberg
```

### 2. Verify the connection

```bash
gitcore test github-work
# [v] Connection successful! Authenticated as shedrackgodstime
```

### 3. Use your identity

```bash
# Clone and automatically configure local git authorship for that account:
gitcore clone work github.com/company/project.git

# Or use the host alias directly:
git clone git@github-work:company/project.git

# Switch an existing repo to a different account:
gitcore remote switch personal
```

---

## Commands

| Command | Description |
|---------|-------------|
| `gitcore add <name> <platform>` | Create a new identity and generate SSH keys |
| `gitcore list` | List configured identities and connection strings |
| `gitcore whoami` | Identify the active Gitcore account in the current repository |
| `gitcore clone [repo]` | Clone a repository with automatic identity injection |
| `gitcore test [host_alias]` | Validate SSH authentication (e.g. `gitcore test github-work`) |
| `gitcore remote add` | Configure an identity-aware remote for a repository |
| `gitcore remote switch` | Transition a repository to a different Gitcore identity |
| `gitcore update <name>` | Modify account metadata (email, GPG key) without re-registration |
| `gitcore export [file]` | Export your entire identity to an encrypted vault |
| `gitcore import [file]` | Restore an entire environment from a vault or JSON |
| `gitcore audit` | Perform security verification on keys and configurations |
| `gitcore rotate <name>` | Regenerate and replace an existing SSH key |
| `gitcore remove <name>` | Decommission an account and its associated keys |

---

## How It Works

Gitcore manages a delimited block within your `~/.ssh/config`:

```ssh-config
# >>> gitcore managed block >>>

Host github-work
  HostName github.com
  User git
  IdentityFile /home/user/.ssh/id_ed25519_work
  AddKeysToAgent yes
  IdentitiesOnly yes

Host codeberg-personal
  HostName codeberg.org
  User git
  IdentityFile /home/user/.ssh/id_ed25519_personal
  AddKeysToAgent yes
  IdentitiesOnly yes

# <<< gitcore managed block <<<
```

When executing `git clone git@github-work:user/repo.git`, OpenSSH automatically selects the correct key. Gitcore uses absolute paths in this configuration to ensure reliability across all environments.

---

## Security Model

- **Key Isolation**: Unique Ed25519 keys per identity minimize the blast radius of a credential leak.
- **`IdentitiesOnly yes`**: Prevents SSH from attempting incorrect keys during the authentication handshake.
- **Permission Enforcement**: Private keys and configurations are strictly enforced at `0600` via native filesystem APIs.
- **Sensitive Input Protection**: Passphrases and master passwords are accepted via hidden terminal echo.
- **Authenticated Encryption**: The vault uses AES-256-GCM, ensuring both confidentiality and cryptographic integrity.

---

## Library Usage

Gitcore is built for integration. You can use the `gitcore` crate to manage Git identities programmatically in your own Rust projects.

```rust
use gitcore::{Gitcore, AddAccountRequest, Platform};

let service = Gitcore::new();
service.register_account(AddAccountRequest {
    name: "work".to_string(),
    platform: Platform::Github,
    username: "octocat".to_string(),
    email: "octocat@example.com".to_string(),
    ..Default::default()
})?;
```

For full library documentation, see [docs/LIBRARY_USAGE.md](docs/LIBRARY_USAGE.md).

## Supported Platforms

| Platform | Status |
|----------|--------|
| Linux | Full |
| macOS | Full |
| Windows | Experimental |

---

## License

MIT - see [LICENSE](LICENSE) for details.

---


Built with ❤️ and Rust &nbsp;·&nbsp; [Report an Issue](https://github.com/shedrackgodstime/gitcore/issues) &nbsp;·&nbsp; [Releases](https://github.com/shedrackgodstime/gitcore/releases)
