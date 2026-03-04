# Verification: S18-SEC-INTEGRATE-003

## Goal
Wire `security` into diycad_format/io/diagnostics/tools-migrate: limits preflight, path traversal blocking, SVG external ref handling, mandatory redaction, consent gating.

## Changed files
- core/crates/diycad_format/** (security preflight + zip path validation integration)
- core/crates/io/** (SSOT security limits preflight wiring)
- core/crates/io_svg/** (SVG external ref blocking via `security::Sandbox`)
- core/crates/diagnostics/** (SecurityCtx + support zip consent/limits/redaction enforcement)
- tools/migrate/** (security import-bytes guard)
- tests/security/integration_limits.rs
- tests/e2e/support_zip_is_safe.rs

## History evidence
- `git log --oneline -n 30`:
  - `728fc40 S18: implement security core crate with SSOT-driven guards`
  - `4b66703 Merge pull request #92 from teru1991/codex/establish-ssot-for-security-specifications`
  - `7e808d2 S18: lock security SSOT and enforce lint checks`
- `git log -- core/crates/diycad_format | head`:
  - `commit acac9d22f4214c00a92326601dcacbaaeece0d7c`
  - `Author: teru1991 <48640151+teru1991@users.noreply.github.com>`
- `git log -- core/crates/diagnostics | head`:
  - `commit 728fc404e434d95ddb5a425a3095e258fc6bfe5e`
  - `Author: Codex <codex@openai.com>`

## Commands executed
- `cargo test -p diycad_format -p craftcad_io -p craftcad_io_svg -p craftcad_diagnostics`
- `cargo test --manifest-path tools/migrate/Cargo.toml`
- `cargo clippy -p diycad_format -p craftcad_io -p craftcad_io_svg -p craftcad_diagnostics -- -D warnings` (failed due pre-existing clippy issues outside this task's scope)
- `scripts/ci/run_all.sh` (failed at rust_fmt step due existing repository-wide formatting/conflict state)

## Assertions
- Oversized import is rejected without panic via security preflight in io/diycad_format.
- Zip path traversal and absolute/device paths are blocked via sandbox path normalization.
- SVG external refs are rejected via `Sandbox::handle_svg_external_refs` policy `Reject`.
- SupportZip applies mandatory redaction and consent defaults (OFF) with size/path guards.
