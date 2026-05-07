# Gitcore

**A secure, zero-friction Git identity manager for developers who juggle multiple accounts.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/shedrackgodstime/gitcore)](https://github.com/shedrackgodstime/gitcore/releases)
[![Binary Size](https://img.shields.io/badge/Binary%20Size-815KB-brightgreen.svg)](https://github.com/shedrackgodstime/gitcore/releases)

```text
  gitcore list

  Configured Git Accounts
  =======================

  [1] work
      Platform: Github
      Host:     github-work
      User:     shedrackgodstime
      Email:    shedrackgodstime@gmail.com
      GPG:      8B92A1C3
      Use:      git clone git@github-work:user/repo.git

  [2] personal
      Platform: Codeberg
      Host:     codeberg-personal
      User:     godstime_dev
      Email:    godstime@proton.me
      Use:      git clone git@codeberg-personal:user/repo.git
```

---

## Why Gitcore?

Standard Git and OpenSSH configurations are designed for a single global identity. For developers managing multiple accounts (e.g., Work, Personal, Open Source), this "one-size-fits-all" approach is fundamentally insufficient. It frequently results in mismatched authorship metadata (incorrect email in commits) and SSH authentication conflicts (incorrect key selection).

Gitcore introduces a dedicated **identity layer** that isolates each account. By automating the mapping between local repositories, specific SSH keys, and signing identities, Gitcore ensures that every operation is performed with the correct persona, with zero manual overhead.

---

## What Gitcore Does

Gitcore is a CLI tool written in Rust that acts as a complete Git identity manager. It isolates each account with its own SSH key, automatically manages `~/.ssh/config` for seamless authentication, and provides commands that integrate identity awareness into standard `clone`, `push`, and `remote` workflows.

It also provides a secure, portable vault — a single encrypted file containing your entire configuration and private keys — allowing for instantaneous environment restoration on new machines.

---

## Key Features

| Feature | Description |
|---|---|
| **Isolated SSH Keys** | Generates a unique Ed25519 key per account to prevent identity cross-contamination. |
| **Automated SSH Config** | Manages a dedicated Gitcore block in `~/.ssh/config`. Existing entries remain untouched. |
| **Encrypted Vault** | Secures your identity in a single `.gitcore` archive encrypted with AES-256-GCM and Argon2id. |
| **GPG Signing** | Integrated GPG key association. Commit signing is configured automatically on `clone` and `remote switch`. |
| **URL Resolution** | Transparently rewrites GitHub, GitLab, Codeberg, and Bitbucket URLs to use correct host aliases. |
| **Security Audit** | Validates file permissions and identifies potential configuration risks. |
| **Zero Dependencies** | Distributed as a statically linked binary with no external runtime requirements. |

---

## Installation

### Linux & macOS

```bash
curl -fsSL https://shedrackgodstime.github.io/gitcore/install | sh
```

### Windows

```powershell
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
| `gitcore clone [repo]` | Clone a repository with automatic identity injection |
| `gitcore test [host_alias]` | Validate SSH authentication (e.g. `gitcore test github-work`) |
| `gitcore remote add` | Configure an identity-aware remote for a repository |
| `gitcore remote switch` | Transition a repository to a different Gitcore identity |
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
