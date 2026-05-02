# Gity Vault: Secure Portability Objective

## Vision
Transform Gity from a local configuration helper into a secure, cross-platform git identity vault. This allows users to move their entire setup (config + actual SSH keys) between machines with a single encrypted file and zero manual steps.

## Consistent Command Structure
To maintain a polished and intuitive CLI, we will transition from technical terms (import/export) to action-oriented verbs.

| Command | Action | Description |
| :--- | :--- | :--- |
| `gity backup [file]` | **Vault Creation** | Encrypts config + all private keys into a `.gity` file. |
| `gity restore [file]` | **Vault Deployment** | Decrypts a `.gity` file and installs keys/config locally. |

## Technical Implementation Details
- **Encryption**: AES-256-GCM (Industry standard authenticated encryption).
- **Key Derivation**: Argon2id or PBKDF2 to "stretch" the Master Password into a high-entropy encryption key, protecting against brute-force attacks.
- **Zero Dependencies**: All cryptographic logic will be statically linked into the Rust binary. The user does not need `gpg`, `openssl`, or any other external tools installed.
- **Path Translation**: Automatic conversion between Unix (`/home/user/.ssh`) and Windows (`C:\Users\user\.ssh`) paths during restoration to ensure cross-platform compatibility.
- **Security**: Private key data is decrypted only in memory and written directly to the target SSH folder with restricted permissions (`0600` on Unix, restricted ACLs on Windows).

## Detailed Workflows

### 1. The Backup Flow
1. User runs `gity backup identity.gity`.
2. Gity prompts for a **Master Password**.
3. Gity reads the local `config.json` to identify all managed accounts.
4. Gity reads the actual bytes of each private key file associated with those accounts.
5. All data (JSON config + Key bytes) is bundled and encrypted using the derived key.
6. A single, secure `.gity` vault file is generated.

### 2. The Restore Flow
1. User runs `gity restore identity.gity` on a new machine.
2. Gity prompts for the **Master Password**.
3. Gity decrypts the vault content in memory and validates integrity.
4. Gity detects the current Operating System.
5. Gity writes the private keys to the local `~/.ssh` folder (creating it if necessary).
6. Gity automatically sets strict file permissions (e.g., `chmod 600`).
7. Gity updates the local `config.json` and injects the managed block into `~/.ssh/config`.
8. **Result**: The user is ready to `git clone` or `git push` immediately with no manual configuration.

## Success Criteria
- The `backup` and `restore` commands replace the need for manual `cp` or `tar` operations.
- A vault exported on Linux can be successfully restored on Windows (and vice-versa).
- The Master Password is the only requirement for a full recovery.
- No sensitive data is stored in the vault in unencrypted form.
