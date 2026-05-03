# Gity

**A secure, zero-friction Git identity manager for developers who juggle multiple accounts.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/shedrackgodstime/gity)](https://github.com/shedrackgodstime/gity/releases)
[![Binary Size](https://img.shields.io/badge/Binary%20Size-815KB-brightgreen.svg)](https://github.com/shedrackgodstime/gity/releases)

```text
  gity list

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

## Why Gity?

Standard Git and OpenSSH configurations are designed for a single global identity. For developers managing multiple accounts (e.g., Work, Personal, Open Source), this "one-size-fits-all" approach is fundamentally insufficient. It frequently results in mismatched authorship metadata (incorrect email in commits) and SSH authentication conflicts (incorrect key selection).

Gity introduces a dedicated **identity layer** that isolates each account. By automating the mapping between local repositories, specific SSH keys, and signing identities, Gity ensures that every operation is performed with the correct persona, with zero manual overhead.

---

## What Gity Does

Gity is a CLI tool written in Rust that acts as a complete Git identity manager. It isolates each account with its own SSH key, automatically manages `~/.ssh/config` for seamless authentication, and provides commands that integrate identity awareness into standard `clone`, `push`, and `remote` workflows.

It also provides a secure, portable vault — a single encrypted file containing your entire configuration and private keys — allowing for instantaneous environment restoration on new machines.

---

## Key Features

| Feature | Description |
|---|---|
| **Isolated SSH Keys** | Generates a unique Ed25519 key per account to prevent identity cross-contamination. |
| **Automated SSH Config** | Manages a dedicated Gity block in `~/.ssh/config`. Existing entries remain untouched. |
| **Encrypted Vault** | Secures your identity in a single `.gity` archive encrypted with AES-256-GCM and Argon2id. |
| **GPG Signing** | Integrated GPG key association. Commit signing is configured automatically on `clone` and `remote switch`. |
| **URL Resolution** | Transparently rewrites GitHub, GitLab, Codeberg, and Bitbucket URLs to use correct host aliases. |
| **Security Audit** | Validates file permissions and identifies potential configuration risks. |
| **Zero Dependencies** | Distributed as a statically linked binary with no external runtime requirements. |

---

## Installation

### Linux & macOS

```bash
curl -fsSL https://shedrackgodstime.github.io/gity/install | sh
```

### Windows (PowerShell)

```powershell
iwr https://shedrackgodstime.github.io/gity/ps | iex
```

### Build from Source

```bash
git clone https://github.com/shedrackgodstime/gity.git
cd gity
cargo build --release
sudo cp target/release/gity /usr/local/bin/gity
```

> **Requirements:** OpenSSH, Git.

---

## Quick Start

### 1. Add your accounts

```bash
gity add work github
# Prompts for: username, email, SSH passphrase, optional GPG key
```

```bash
gity add personal codeberg
```

### 2. Verify the connection

```bash
gity test github-work
# [v] Connection successful! Authenticated as shedrackgodstime
```

### 3. Use your identity

```bash
# Clone and automatically configure local git authorship for that account:
gity clone work github.com/company/project.git

# Or use the host alias directly:
git clone git@github-work:company/project.git

# Switch an existing repo to a different account:
gity remote switch personal
```

---

## Commands

| Command | Description |
|---------|-------------|
| `gity add <name> <platform>` | Create a new identity and generate SSH keys |
| `gity list` | List configured identities and connection strings |
| `gity clone [repo]` | Clone a repository with automatic identity injection |
| `gity test [host_alias]` | Validate SSH authentication (e.g. `gity test github-work`) |
| `gity remote add` | Configure an identity-aware remote for a repository |
| `gity remote switch` | Transition a repository to a different Gity identity |
| `gity export [file]` | Export your entire identity to an encrypted vault |
| `gity import [file]` | Restore an entire environment from a vault or JSON |
| `gity audit` | Perform security verification on keys and configurations |
| `gity rotate <name>` | Regenerate and replace an existing SSH key |
| `gity remove <name>` | Decommission an account and its associated keys |

---

## How It Works

Gity manages a delimited block within your `~/.ssh/config`:

```ssh-config
# >>> gity managed block >>>

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

# <<< gity managed block <<<
```

When executing `git clone git@github-work:user/repo.git`, OpenSSH automatically selects the correct key. Gity uses absolute paths in this configuration to ensure reliability across all environments.

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


Built with ❤️ and Rust &nbsp;·&nbsp; [Report an Issue](https://github.com/shedrackgodstime/gity/issues) &nbsp;·&nbsp; [Releases](https://github.com/shedrackgodstime/gity/releases)
