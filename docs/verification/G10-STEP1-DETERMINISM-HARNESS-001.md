# G10-STEP1-DETERMINISM-HARNESS-001 Verification

## Summary
- Added a deterministic harness gate for lite artifacts (`ProjectionLite`, `EstimateLite`, `FastenerBOMLite`).
- Harness runs repeated hash calculations on identical SSOT input and fails on drift.
- Added CI step to run the harness for every run.

## Changed files
- docs/specs/product/contracts/jobs.md
- docs/specs/product/testing/e2e-flows.md
- docs/specs/determinism/wizard_policy.md
- core/Cargo.toml
- core/crates/craftcad_mfg_hints_lite/src/lib.rs
- core/crates/craftcad_determinism_harness/**
- scripts/ci/run_all.sh
- docs/status/trace-index.json

## History evidence
- `git status --porcelain`
- `git fetch --all --prune`
- `git switch -c feature/g10-step1-determinism-harness-001`
- `git rev-parse HEAD`
- `git log -n 30 --oneline`
- `git branch -vv`
- `ls -la docs/specs/determinism`
- `rg -n "determinism|seed|stable sort|epsilon|round6" docs/specs core/crates tests -S`
- `rg -n "sheet_hash_hex|estimate_hash_hex|bom_hash_hex|hints_hash_hex" core/crates -S`

## Local verification
- `cargo fmt --all` (pass)
- `cargo test -p craftcad_determinism_harness` (pass)
- `cargo run -q -p craftcad_determinism_harness --bin craftcad-determinism-check` (pass)
- `./scripts/ci/run_all.sh` (runs with determinism step pass; exits non-zero due existing unrelated failures `rust_test`, `e2e_shelf_flow`)
- `python -m json.tool docs/status/trace-index.json >/dev/null` (pass)

## Self-check
- Allowlist respected.
- No deletion performed.
- Flakiness avoidance: fixed N=3 repeated checks, deterministic fixture UUIDs, stable sort paths.
