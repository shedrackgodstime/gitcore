# Gity Checklist

## Core Safety
- [x] Make SSH config updates safe and non-destructive
- [x] Preserve non-gity entries in `~/.ssh/config`
- [x] Avoid duplicate or conflicting host aliases

## Feature Completion
- [x] Implement `gity backup` (encrypted vault with keys)
- [x] Implement `gity restore` (handles .gity vaults and legacy JSON)
- [x] Support restoring from a file

## Behavior Fixes
- [x] Align `remove` behavior with the README
- [x] Verify clone flow handles existing directories correctly
- [x] Verify remote add/switch handles existing remotes clearly
- [x] Improve command failure reporting for `git`, `ssh`, and `ssh-keygen`

## Validation
- [x] Reject empty or invalid account names
- [x] Validate platform/account combinations before writing config
- [x] Detect duplicate account names and duplicate host aliases
- [x] Handle missing `~/.ssh` directory cleanly

## Testing
- [x] Add tests for `convert_to_host`
- [x] Add tests for config load/save behavior
- [x] Add tests for account add/remove edge cases

## Documentation
- [x] Update README to match actual behavior
- [x] Add notes about how `~/.ssh/config` is managed
- [x] Add development and testing instructions
