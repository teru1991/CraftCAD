# Rust Quality Runbook

## Purpose
Maintain zero compile/clippy/rustdoc warnings in the UCEL Rust workspace and fail fast in CI when quality regresses.

## Local commands
- macOS/Linux:
  - `./scripts/rust_quality.sh`
- Windows PowerShell:
  - `.\scripts\rust_quality.ps1`

Both scripts run:
1. `cargo fmt --all -- --check`
2. `RUSTFLAGS="-D warnings" cargo check --workspace --all-targets`
3. `cargo test --workspace --all-targets`
4. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
5. `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`

## Common lint fixes
- `unwrap` / `expect`:
  - Do not add new `unwrap`/`expect` in production paths.
  - Return `Result` and propagate with context.
- `unused_*`:
  - Remove unused imports/variables; if intentional, encode behavior via `cfg` or underscore-prefixed local binding.
- `dead_code`:
  - Remove unreachable or unused code where possible.
  - If API must remain for compatibility, deprecate and keep it referenced by tests/usage.
- clippy style lints:
  - Prefer deriving standard traits (`Default`, etc.) when available.

## Exception policy for `#[allow(...)]`
Use `#[allow(...)]` only as last resort:
1. Narrowest scope possible (statement/block/function).
2. Add reason comment describing safety/compatibility rationale.
3. Add a test/guard proving invariant so unsafe regression fails loudly.
