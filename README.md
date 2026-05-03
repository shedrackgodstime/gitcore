<div align="center">

# Gity

**A secure, zero-friction Git identity manager for developers who juggle multiple accounts.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/shedrackgodstime/gity)](https://github.com/shedrackgodstime/gity/releases)
[![Binary Size](https://img.shields.io/badge/Binary%20Size-815KB-brightgreen.svg)](https://github.com/shedrackgodstime/gity/releases)

```
  gity list

  Configured Git Accounts
  =======================

  [1] work (github.com)
      Host:     github-work
      User:     john_doe | john@company.com
      Signing:  [v] GPG (8B92A1C3)
      Use:      git clone git@github-work:user/repo.git

  [2] personal (codeberg.org)
      Host:     codeberg-personal
      User:     john_home | john@proton.me
      Use:      git clone git@codeberg-personal:user/repo.git
```

</div>

---

## The Problem

Every developer with more than one Git account has felt this pain. You have a work GitHub account and a personal one. You want to push to a personal project but your machine is configured for work. You end up:

- Typing `git push` and seeing it go under the wrong username.
- Manually exporting and importing SSH keys between machines.
- Keeping a sticky note with the exact `ssh-config` block to copy-paste when setting up a new machine.
- Accidentally committing to a corporate repository with your personal email.

The root cause is that the standard `ssh-agent` and `~/.ssh/config` approach works well for *one* identity but becomes genuinely complex when you're managing two or more.

**Gity solves this entirely.**

---

## What Gity Does

Gity is a CLI tool written in Rust that acts as a **complete Git identity manager**. It stores each identity (account) with its own isolated SSH key, automatically wires up `~/.ssh/config` so SSH picks the right key without you ever thinking about it, and provides commands that bring identity awareness directly into your everyday `clone`, `push`, and `remote` workflows.

It also gives you an **encrypted portable vault** — a single file that contains your entire Git identity (config + SSH keys), password-protected with AES-256-GCM, so restoring a new machine takes seconds, not an hour.

---

## Key Features

| Feature | Description |
|---|---|
| 🔑 **Isolated SSH Keys** | Generates a unique `Ed25519` SSH key per account. No shared keys, no conflicts. |
| ⚙️ **Auto SSH Config** | Writes and manages a dedicated block in `~/.ssh/config`. Your other SSH entries are never touched. |
| 🔒 **Encrypted Vault** | Backs up your entire identity to a single `.gity` file encrypted with AES-256-GCM + Argon2 key derivation. |
| ✍️ **GPG Signing** | Optionally associates a GPG key with an account. Commit signing is configured automatically on `clone` and `remote switch`. |
| 🌐 **Smart URL Resolution** | Understands GitHub, GitLab, Codeberg, and Bitbucket URLs in any format (HTTPS, SSH, shorthand) and rewrites them to use the correct host alias. |
| 🛡️ **Security Audit** | Scans your SSH key file permissions and SSH config for common misconfigurations. |
| 🎯 **Interactive & Self-Healing** | If you run a command with no accounts configured, Gity guides you through adding your first one on the spot. |
| 📦 **Single Binary, No Runtime** | Ships as a single static binary with zero runtime dependencies. |

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
git clone git@github.com:shedrackgodstime/gity.git
cd gity
cargo build --release
sudo cp target/release/gity /usr/local/bin/gity
```

> **Requirements:** OpenSSH, Git. That's it.

---

## Quick Start

### 1. Add your accounts

```bash
gity add work github
# Prompts for: username, email, SSH passphrase (hidden), optional GPG key
```

```bash
gity add personal codeberg
```

Each `add` command:
- Generates a unique `~/.ssh/id_ed25519_<name>` key pair.
- Updates `~/.ssh/config` with a dedicated host alias.
- Displays the public key and a direct link to add it to your provider.

### 2. Verify the connection

```bash
gity test work
# => [v] SSH connection to github.com: authenticated as john_doe
```

### 3. Use your identity

```bash
# Clone and automatically configure local git user for that account:
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
| `gity add <name> <platform>` | Add a new account and generate its SSH key |
| `gity list` | List all configured accounts with their connection strings |
| `gity clone [repo]` | Clone a repo and automatically configure local git identity |
| `gity test [name]` | Test SSH authentication for an account |
| `gity remote add` | Add an identity-aware remote to an existing repo |
| `gity remote switch` | Switch a repo's `origin` remote to a different account |
| `gity backup [file]` | Create an AES-256-GCM encrypted vault of your entire identity |
| `gity restore [file]` | Restore config + SSH keys from a vault |
| `gity audit` | Audit SSH key permissions and config for security issues |
| `gity rotate <name>` | Regenerate the SSH key for an account |
| `gity remove <name>` | Remove an account and optionally delete its SSH keys |

---

## How It Works

Gity owns a clearly delimited block inside your `~/.ssh/config`:

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

When you run `git clone git@github-work:company/project.git`, OpenSSH reads the config and automatically uses `id_ed25519_work`. You never touch it again. All your other SSH entries (servers, VPSes, etc.) are completely unaffected.

---

## Portable Identity Vault

The vault feature is designed for the moment you get a new machine.

```bash
# On your old machine — one command backs up everything:
gity backup my_identity
# Creates 'my_identity.gity' — an AES-256-GCM encrypted archive.

# On your new machine — one command restores everything:
gity restore my_identity
# Decrypts the vault, restores SSH keys with correct file permissions,
# rebuilds ~/.ssh/config, and you're ready to push within seconds.
```

The vault format:
- Encrypted with **AES-256-GCM** (authenticated encryption).
- Key derived with **Argon2id** (memory-hard, phishing-resistant).
- Password is always input with hidden terminal echo for security.

---

## Security Model

Gity is built with a security-first mindset:

- **Key Isolation**: Each account gets its own `Ed25519` key. A leak of one key never compromises another.
- **`IdentitiesOnly yes`**: Every SSH host alias explicitly disables key negotiation, ensuring SSH only ever offers the correct key to the correct host.
- **File Permissions**: SSH private keys are always written and enforced as `0600`. The SSH config is enforced as `0600`. This is done natively via Rust's `fs::set_permissions` — never via an external `chmod` call.
- **Hidden Input**: All passphrases and passwords are accepted with hidden terminal echo (using `rpassword`). They are never logged or stored.
- **Authenticated Encryption**: The vault uses AES-256-GCM, which provides both confidentiality and integrity. A tampered vault is rejected before any data is written.
- **`gity audit`**: Proactively scans your setup and reports any keys or configs with incorrect permissions.

---

## Architecture

Gity is built in Rust and follows a strict **"Modularity over Monoliths"** principle. The codebase is split into clean, single-responsibility modules:

```
src/
├── main.rs      # Entry point
├── cli.rs       # Command definitions (Clap derive)
├── app.rs       # Command orchestration
├── models.rs    # Account structs + validation logic
├── config.rs    # Config load/save (JSON, ~/.config/gity/)
├── ssh.rs       # SSH key generation + config management
├── git.rs       # Git operations + URL resolution
├── gpg.rs       # GPG key discovery
├── vault.rs     # AES-256-GCM encryption/decryption
└── ui.rs        # Terminal interaction helpers
```

The project maintains a comprehensive unit test suite covering URL parsing, account validation, config serialization, and encryption cycles.

---

## Supported Platforms

| Platform | Status |
|----------|--------|
| Linux | ✅ Fully supported |
| macOS | ✅ Fully supported |
| Windows | ⚠️ Experimental |

---

## License

MIT — see [LICENSE](LICENSE) for details.

---

<div align="center">

Built with ❤️ and Rust &nbsp;·&nbsp; [Report an Issue](https://github.com/shedrackgodstime/gity/issues) &nbsp;·&nbsp; [Releases](https://github.com/shedrackgodstime/gity/releases)

</div>
