# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.0] - 2026-05-08

### Added
- **Flexible Key Paths**: Accounts can now specify custom SSH key filenames via `key_path` in `AddAccountRequest`.
- **Account Update**: Added `Gitcore::update_account` and `gitcore update` command to modify account metadata without re-registration.
- **Context Detection**: Added `Gitcore::detect_account_in_repository` and `gitcore whoami` to identify the active managed account in a local repository.
- **Library Documentation**: Comprehensive doc comments and runnable examples added to the public API in `src/lib.rs`.
- **Library Usage Guide**: New documentation in `docs/LIBRARY_USAGE.md` explaining how to embed Gitcore as a library.
- **Integration Tests**: End-to-end test suite for account lifecycles and vault operations.

### Changed
- **Rebranding**: Finalized the migration from `gity` to `gitcore`.
- **Architecture**: Completed the "Fat Library, Thin CLI" split. The core logic is now 100% programmatically usable without terminal IO.
- **Error Handling**: Standardized on `GitcoreError` throughout the library, replacing raw `io::Error` at high-level boundaries.
- **Configuration**: Improved config recovery logic and enforced strict permissions (0o600) on all sensitive files.

## [1.2.0] - 2026-04-20
- Initial public release of `gitcore`.
- Support for GitHub, GitLab, Codeberg, and Bitbucket.
- Secure encrypted vaults for configuration backups.
- Automatic SSH host alias generation.
