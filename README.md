# Gity - Git Account Manager

Manage multiple Git accounts safely using separate SSH keys for each account.

## Why Gity?

If you have multiple GitHub/Codeberg/GitLab accounts (e.g., work + personal) and want to:
- Keep them isolated and secure
- Avoid constantly switching SSH keys
- Have a clean workflow

Gity creates separate SSH keys for each account and auto-configures your `~/.ssh/config` so you can use different accounts without conflict.
It manages only a dedicated `gity` block inside `~/.ssh/config`, leaving your other SSH entries intact.

## Installation

```bash
# Quick install (Linux/macOS)
curl -fsSL https://shedrackgodstime.github.io/gity/install | sh

# Quick install (Windows)
irm https://shedrackgodstime.github.io/gity/ps | iex

# Or build from source
cargo build --release
sudo cp target/release/gity /usr/local/bin/gity
```

## Quick Start

### 1. Add an Account

```bash
# Add a GitHub account
gity add work github
# Enter your email when prompted

# Add a Codeberg account
gity add mycodeberg codeberg

# Add GitLab
gity add personal gitlab
```

This will:
- Generate an SSH key: `~/.ssh/id_ed25519_<name>`
- Update `~/.ssh/config` with the host alias
- Show you the public key to add to your git provider

### 2. Add Public Key to Git Provider

The output shows the public key. Add it to:
- **GitHub**: https://github.com/settings/keys
- **GitLab**: https://gitlab.com/-/profile/keys
- **Codeberg**: https://codeberg.org/user/keys
- **Bitbucket**: https://bitbucket.org/account/settings/ssh-keys/

### 3. Verify Connection

```bash
gity test work
```

Should show: `✓ Connection successful!`

### 4. Clone Using Your Account

```bash
# Instead of:
git clone git@github.com:username/repo.git

# Use the host alias from gity list:
git clone git@github-work:username/repo.git
```

## Commands

| Command | Description |
|---------|-------------|
| `gity add <name> <platform>` | Add a new account (platform: github, gitlab, codeberg, bitbucket) |
| `gity list` | List all accounts with usage instructions |
| `gity test [name]` | Test SSH connection to verify key is working |
| `gity remote add` | Interactive: create remote URL for a specific account |
| `gity remote switch` | Switch the current repo `origin` remote to a different configured account |
| `gity export` | Export config to JSON (for backup or moving to another PC) |
| `gity import < file.json` | Import config from file or stdin |
| `gity remove <name>` | Remove an account from gity config, with an optional prompt to delete its SSH key files |

## Usage Examples

### Clone a repo with specific account

```bash
# After adding "work" account:
git clone git@github-work:company/project.git
```

### Switch repo to different account

```bash
# In your repo directory:
gity remote switch personal
```

### Export/Import for another PC

```bash
# On PC 1 - export your setup
gity export > gity-config.json

# Copy SSH keys manually
cp ~/.ssh/id_ed25519_work* /backup/
cp ~/.ssh/id_ed25519_personal* /backup/

# On PC 2 - import config
gity import < gity-config.json

# Copy SSH keys
cp /backup/* ~/.ssh/
chmod 600 ~/.ssh/id_ed25519_*
```

## How It Works

Gity manages a block in your SSH config like this:

```
# >>> gity managed block >>>
Host github-work
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519_work
  AddKeysToAgent yes
  IdentitiesOnly yes

Host codeberg-codeberg
  HostName codeberg.org
  User git
  IdentityFile ~/.ssh/id_ed25519_codeberg
  AddKeysToAgent yes
  IdentitiesOnly yes
# <<< gity managed block <<<
```

Then you use `git@github-work:user/repo` instead of `git@github.com:user/repo` - SSH automatically picks the right key.

## Security Notes

- SSH keys are stored in `~/.ssh/` with 600 permissions
- Each key is isolated per account
- Gity updates only its managed block in `~/.ssh/config`
- Use `gity test` to verify before pushing

## Requirements

- Rust (to build)
- OpenSSH
- Git

## Development

Run the local quality gate before committing changes:

```bash
cargo fmt --all --check
cargo test
cargo check --workspace
cargo clippy --all-targets --all-features -- -D warnings
```

## License

MIT
