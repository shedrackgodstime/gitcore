# Gity Security Considerations

## 1. SSH Key Passphrase Protection

**Current:** Keys are generated without passphrase (`-N ""`)

**Risk:** If private key is stolen, attacker can use it immediately without authentication

**Solution:** Prompt user for passphrase when generating keys
- Empty passphrase = no protection (not recommended)
- Passphrase encrypts the private key
- Even if key is stolen, attacker needs passphrase to use it

**Implementation:**
```rust
// Prompt for passphrase (can be empty)
// If provided, use: ssh-keygen -t ed25519 -f key -N "passphrase" -C "email"
```

---

## 2. File Permissions

### SSH Keys (`~/.ssh/id_ed25519_*`)
- **Required:** `600` (owner read/write only)
- **Risk:** If `644` or world-readable, any user on system can read the key

### SSH Config (`~/.ssh/config`)
- **Required:** `600`
- **Risk:** Exposure of host aliases and key paths

### Gity Config (`~/.config/gity/config.json`)
- **Required:** `600`
- **Risk:** Exposure of account names, emails, host aliases

**Implementation:**
- After key generation, run `chmod 600` on private key
- When updating SSH config, ensure permissions are correct
- Add `gity audit` command to check all permissions

---

## 3. SSH Agent Integration

**Current:** Config has `AddKeysToAgent yes`

**Enhancement:**
- Use `ssh-agent` to hold decrypted keys in memory
- Keys only decrypted in memory, never written to disk unprotected
- Keys auto-added when needed

---

## 4. GPG Commit Signing (Future)

**Different from SSH keys:**
- SSH key = authentication (who you are)
- GPG key = commit signing (who wrote this code)

**Benefits:**
- Even if SSH key stolen, attacker cannot forge signed commits
- Signed commits verified by git (Tamper-proof)
- GitHub/GitLab show "Verified" badge

**Implementation:** Add command to generate GPG key and configure git to sign commits

---

## 5. Host Key Verification

**Risk:** Man-in-the-Middle (MITM) attack - attacker intercepts connection

**Solution:**
- Verify `known_hosts` before connecting
- Warn if host key changed
- Support `StrictHostKeyChecking`

---

## 6. Audit Command

**Purpose:** Security health check

**Features:**
- Check all key file permissions
- Check SSH config permissions
- Check gity config permissions
- Verify keys have passphrases (optional warning)
- Check for weak keys (deprecated algorithms like RSA-1024)
- Report security issues

**Example Output:**
```
gity audit

🔍 Security Audit
=================

SSH Keys:
  ✓ ~/.ssh/id_ed25519_kriz (600)
  ✓ ~/.ssh/id_ed25519_personal (600)

SSH Config:
  ✓ ~/.ssh/config (600)

Gity Config:
  ⚠ ~/.config/gity/config.json (644) - FIX: chmod 600

Keys with passphrases:
  ✓ kriz - protected
  ✗ personal - NOT protected (no passphrase)

Recommendation: Run 'chmod 600 ~/.config/gity/config.json'
```

---

## 7. Key Rotation

**Purpose:** Regenerate keys periodically or after suspected compromise

**Features:**
- `gity rotate <account>` - regenerate SSH key
- Prompt to update remote (show new public key)
- Remove old key from known systems

---

## 8. Multi-Factor Authentication (Future)

**For git providers:**
- SSH key = factor 1 (what you have)
- GPG signing = factor 2 (what you signed)

---

## Security Priority Checklist

| Priority | Feature | Status |
|----------|---------|--------|
| 1 | Passphrase on keys | Planned |
| 2 | File permission enforcement | Planned |
| 3 | Audit command | Planned |
| 4 | SSH agent integration | Existing |
| 5 | Host key verification | Future |
| 6 | GPG signing | Future |
| 7 | Key rotation | Future |

---

## Best Practices for Users

1. **Always use passphrase** on SSH keys
2. **Never share** private keys
3. **Use separate keys** for each account
4. **Rotate keys** periodically
5. **Use audit command** regularly
6. **Enable GPG signing** for important repos
7. **Don't commit keys** to repositories
8. **Use ssh-agent** for convenience + security