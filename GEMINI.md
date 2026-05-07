# Strategic Plan: Transition to Published Rust Crate

This document outlines the roadmap for transitioning the project from a local tool ("Gitcore") to a published Rust crate.

## Phase 1: Brand Migration
The objective is to move away from the "Gitcore" name to avoid conflicts and prepare the repository for a public launch.

- **Finalize Identity:** Confirm the new name (e.g., Gitcore) by verifying availability on crates.io and GitHub.
- **Repository Refactor:** Rename the GitHub repository to trigger automatic redirects.
- **Internal Rebranding:** Perform a global search-and-replace in the codebase to update the binary name, help strings, file paths (like `~/.gitcore` to `~/.gitcore`), and documentation.

## Phase 2: Distribution Readiness (Binary)
The objective is to prepare the package specifically for the Rust ecosystem's package manager.

- **Metadata Validation:** Update `Cargo.toml` with the required publishing fields (description, license, keywords, and homepage).
- **Binary Specification:** Explicitly define the `[[bin]]` target in the configuration to ensure `cargo install` works out of the box for users.
- **Clean-room Testing:** Run a local `cargo package` command to verify that the crate builds successfully from its packaged state without local environment dependencies.

## Phase 3: Initial Publication
The objective is to "claim" the name and provide the first public installation path.

- **Crates.io Release:** Execute the initial publish of the Binary-only version.
- **Installation Verification:** Confirm that the tool can be installed via `cargo install` across different operating systems (Linux/macOS).
- **Update Installers:** Update curl and powershell installation scripts to point to the new repository and binary name.

## Phase 4: Architecture Evolution (Lib + Bin)
The objective is to transform the tool into a developer platform, allowing others to import your logic.

- **Modular Decoupling:** Separate the core "business logic" (encryption, SSH parsing, account management) from the "CLI interface" (argument parsing and terminal printing).
- **Library Exposure:** Create a `src/lib.rs` to expose the core logic as a public API.
- **Dual-Purpose Configuration:** Update `Cargo.toml` to define both a `[lib]` and a `[[bin]]`, allowing your own CLI to consume the library while others do the same.
- **Version Bump:** Publish the updated version to Crates.io, officially moving from a "tool" to a "crate ecosystem."
