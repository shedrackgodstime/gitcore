# Design Proposal: Gitcore Secure Gatekeeper

## 1. The Core Problem: Exfiltration Vulnerability
Standard Git/SSH workflows rely on "File-Backed Keys." These are plaintext (or easily decryptable) files stored in `~/.ssh/`. 

**The Threat:**
In the modern development ecosystem, supply chain attacks (e.g., malicious `npm` or `pip` packages) are rampant. When a developer runs `npm install`, the script runs with the same permissions as the user. 
*   **Malware's First Goal:** Scan `~/.ssh/` for private keys.
*   **The Outcome:** Your identity is stolen, and your corporate/personal repositories are compromised. To the OS, there is no difference between a legitimate `git push` and a malicious script reading the key file.

---

## 2. Proposed Solution: `gitcore` as a Gatekeeper
Instead of being a "Configuration Manager," `gitcore` evolves into a "Secure Executor."

### Concept: Ephemeral Identity Injection
1.  **Zero-Persistence:** Move private keys out of `~/.ssh/` entirely. They are stored encrypted within the `gitcore` vault.
2.  **`gitcore push/pull/fetch`:** Users stop running `git push`. They run `gitcore push`.
3.  **Just-in-Time Decryption:** 
    *   `gitcore` retrieves a "Master Key" from the **OS Secret Store** (Gnome Keyring / macOS Keychain).
    *   This triggers a native system password/biometric popup (preventing silent background theft).
    *   The SSH key is decrypted into **memory only**.
    *   `gitcore` executes the `git` command, injecting the key through a temporary SSH agent.
    *   The key vanishes the moment the command finishes.

---

## 3. The Trade-offs (The Developer Friction)

### A. IDE & Tooling Breakage
Most developers use VS Code, IntelliJ, or GUI clients.
*   **Problem:** These tools expect to find keys in `~/.ssh/`. If `gitcore` hides them, the IDE's "Push" button will break.
*   **Maintenance Burden:** Supporting these tools would require building a background daemon or a custom "SSH Proxy" that the IDEs can talk to.

### B. User Friction (Password Fatigue)
*   **Problem:** If the OS Secret Store isn't configured for long-term "unlock" sessions, the user might be prompted for a password on every single `pull` or `fetch`.
*   **UX Risk:** Developers hate friction. If it's too annoying, they will revert to insecure plaintext keys.

### C. Platform Complexity
*   **Problem:** Handling memory-only keys and system keychains is drastically different across Linux, macOS, and Windows (OpenSSH vs. Pageant).
*   **Maintenance Burden:** This significantly increases the complexity of the `gitcore` codebase and its bug surface.

---

## 4. Alternative Solutions Considered

### Tiered Security Model
1.  **Standard Mode (Current):** Manage `~/.ssh/config`, keep keys in files. (High compatibility, High risk).
2.  **Vault Mode (Proposed):** `gitcore push` gatekeeper. Keys stay in the vault. (Low compatibility, High security).
3.  **Hardware Mode:** Support for FIDO2/YubiKey. (High security, requires hardware purchase). *Note: Currently deprioritized based on user feedback.*

---

## 5. Peer Review Goals
*   Is the "Memory-Only" injection via `gitcore push` a viable replacement for `git push`?
*   Should `gitcore` provide a "Compatibility Layer" (like a persistent SSH Agent) to avoid breaking IDEs?
*   Is the dependency on OS-native secret stores (via the `keyring` crate) acceptable for a Rust-based CLI?
