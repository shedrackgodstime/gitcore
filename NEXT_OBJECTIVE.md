# Next Objective

## Goal

Refactor `gity` from a working MVP into a more stable, maintainable CLI tool without changing user-facing behavior.

The current implementation works, but too much logic is concentrated in `src/main.rs`. For a tool that should be trusted long-term, the next step is structural cleanup, better separation of concerns, and stronger testability.

## Why This Is Next

Right now the project is functionally promising, but the codebase shape is still fragile:

- `src/main.rs` is too large and mixes domain logic, command orchestration, terminal prompts, config handling, git/ssh execution, and tests
- interactive account selection and prompt flows are repeated in several commands
- error handling is inconsistent across commands
- config loading is too forgiving for a serious tool
- current tests only cover a small pure function

If more features are added before cleanup, the maintenance cost and regression risk will rise quickly.

## Primary Objective

Restructure the codebase into focused modules while preserving the current CLI behavior and passing the existing quality gate:

```bash
cargo fmt --all --check
cargo test
cargo check --workspace
cargo clippy --all-targets --all-features -- -D warnings
```

## Proposed Target Structure

This structure is the intended direction, not a rigid requirement:

```text
src/
  main.rs
  cli.rs
  models.rs
  config.rs
  ssh.rs
  git.rs
  ui.rs
```

### Suggested Responsibilities

- `main.rs`
  - program entry point
  - command dispatch only

- `cli.rs`
  - clap command definitions
  - subcommand enums

- `models.rs`
  - `Account`
  - `Platform`
  - `GityConfig`
  - validation helpers closely tied to the domain

- `config.rs`
  - config path resolution
  - load/save config
  - config validation behavior

- `ssh.rs`
  - SSH key generation
  - SSH config block generation/update
  - key deletion helpers
  - SSH connection test helpers

- `git.rs`
  - git command execution helpers
  - remote add/switch helpers
  - clone-related helpers
  - repo-local git config setup

- `ui.rs`
  - prompt helpers
  - account selection helper
  - yes/no confirmation helper
  - shared output formatting helpers

## Non-Goals

These are not the main target of the next pass:

- adding new end-user features
- redesigning the CLI surface
- changing documented command semantics
- optimizing for clever abstractions over clarity

## Success Criteria

The refactor should be considered successful if:

- behavior remains the same from a user perspective
- `src/main.rs` becomes small and easy to scan
- repeated input-selection-confirmation flows are centralized
- command helpers return consistent errors
- config read failures and malformed config behavior are made explicit or intentionally handled
- more logic becomes testable without interactive stdin/stdout
- the full cargo quality gate remains green

## Immediate Refactor Priorities

1. Move clap command definitions out of `main.rs`
2. Move domain models and platform helpers into a dedicated module
3. Extract config load/save/update behavior
4. Extract SSH helpers
5. Extract git helpers
6. Centralize repeated terminal prompt/selection logic
7. Add tests for config behavior and account edge cases

## Expected Outcome

After this pass, `gity` should still feel like the same tool to the user, but the internals should be much easier to extend, review, and trust.
