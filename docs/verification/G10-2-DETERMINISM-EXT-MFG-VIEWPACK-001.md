# G10-2-DETERMINISM-EXT-MFG-VIEWPACK-001 Verification

## Summary
- Extended determinism harness coverage to include `mfg_hints_lite` and `viewpack_v1` hashes in addition to existing projection/estimate/fastener checks.
- Added richer mismatch failure bundle output under `build/determinism_failures/<run_id>/` containing input SSOT, per-run artifact JSON bytes, hashes, and environment metadata.
- Updated determinism gate docs to make expanded artifact coverage and failure-bundle contract explicit.

## Changed files
- `docs/specs/product/testing/e2e-flows.md`
- `docs/specs/determinism/wizard_policy.md`
- `core/crates/craftcad_determinism_harness/Cargo.toml`
- `core/crates/craftcad_determinism_harness/src/lib.rs`
- `core/crates/craftcad_determinism_harness/src/main.rs`
- `core/crates/craftcad_determinism_harness/tests/cli.rs`
- `scripts/ci/run_all.sh`
- `docs/verification/G10-2-DETERMINISM-EXT-MFG-VIEWPACK-001.md`
- `docs/status/trace-index.json`

## History evidence
- `git status --porcelain`
- `git fetch --all --prune`
- `git switch -c feature/g10-2-determinism-ext-mfg-viewpack-001`
- `git rev-parse HEAD`
- `git log -n 40 --oneline`
- `git branch -vv`

## Discovery evidence
- Harness location and current scope:
  - `rg -n "determinism|harness|ProjectionLite|EstimateLite|FastenerBom|hash" core/crates -S`
- mfg/viewpack hash producers:
  - `rg -n "compute_mfg_hints_lite|hints_hash_hex|ViewerPack|viewpack|manifest|ssot_hash" core/crates -S`
- Failure bundle path wiring:
  - `rg -n "determinism_failures|failure bundle|build/determinism_failures" core/crates scripts -S`

## Local verification
- `cargo test -p craftcad_determinism_harness --manifest-path core/Cargo.toml`
- `cargo run -q -p craftcad_determinism_harness --bin craftcad-determinism-check --manifest-path core/Cargo.toml`
  - Confirmed JSON summary includes `mfg_hints_hash` and `viewpack_hash`.
- `python -m json.tool docs/status/trace-index.json >/dev/null`
- `./scripts/ci/run_all.sh`

## Self-check
- Allowlist respected (`docs/**`, `core/**`, `tests/**`, `scripts/**`).
- No deletions.
- `trace-index.json` is valid JSON.
