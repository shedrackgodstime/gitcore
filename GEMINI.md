# Project Mandates: Gity Cleanliness & Integrity

This document serves as the foundational law for all development on the Gity project. Any agent or developer working on this codebase must adhere to these standards to maintain the project's current state of clarity and reliability.

## 1. Architectural Integrity
- **Modularity over Monoliths**: Logic must be separated into focused modules (`ssh`, `git`, `ui`, `config`, `models`). No domain logic should ever be added back into `main.rs`.
- **Surgical Edits**: When making changes, modify only what is necessary. Avoid unrelated refactoring or "cleanups" unless specifically tasked with a refactor.
- **Single Source of Truth**: The `Cargo.toml` is the only source of truth for versioning and dependencies. All scripts and code must reference it dynamically.

## 2. Technical Standards (Rust)
- **Zero Warnings**: The project must remain `clippy`-clean. Suppressing warnings with `#[allow(...)]` is a last resort and requires a technical justification.
- **Explicit over Implicit**: Use explicit error handling (`Result`, `?`) and clear type definitions. Avoid "magic strings" or hidden side effects.
- **Dependency Minimalism**: Only add new crates if the functionality is impossible or extremely complex to implement manually. Prefer small, well-audited crates for security tasks.

## 3. Workflow & Verification
- **Empirical Validation**: Before claiming a fix is complete, it must be verified with a test or a manual confirmation in the target environment (e.g., Linux/Windows).
- **Update Documentation**: Every new feature or command change must be reflected immediately in `README.md` and the internal `CHECKLIST.md`.
- **Commit to Objectives**: Feature development should follow the path laid out in `NEXT_OBJECTIVE.md` and `VAULT_OBJECTIVE.md`. Any divergence must be documented.

## 4. UI & UX Consistency
- **Verb-Oriented Commands**: All subcommands must be active, simple verbs (e.g., `add`, `list`, `backup`, `restore`).
- **Informative Feedback**: Use the `colored` crate to provide clear visual cues (green for success, red for errors, yellow for hints). Always provide a "next step" hint when a command fails.

## 5. Security First
- **Credential Protection**: Never implement logic that logs or stores private keys in plain text.
- **Absolute Paths**: Always use absolute paths for system configurations (`~/.ssh/config`) to ensure cross-platform reliability.
- **Permission Enforcement**: Any code that creates files must explicitly set restricted permissions (`0600`) as demonstrated in `src/ssh.rs`.
