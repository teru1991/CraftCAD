# G10-8-SUPPORTZIP-EXT-001 Verification

## Summary
- Extended SupportZip payload with a `repro/` bundle that includes redacted SSOT snapshot, derived artifact hashes, optional DirtyPlan, and runtime environment info.
- Preserved existing redaction/consent/limits flow by using the diagnostics security context redactor and existing size checks for each entry.
- Added a dedicated supportzip smoke test and CI step to keep the new payload contract continuously checked.

## Changed files
- `docs/specs/diagnostics/support_zip.md`
- `core/crates/diagnostics/Cargo.toml`
- `core/crates/diagnostics/src/lib.rs`
- `core/crates/diagnostics/src/support_zip.rs`
- `core/crates/diagnostics/tests/support_zip_repro_bundle.rs`
- `scripts/ci/run_all.sh`
- `docs/verification/G10-8-SUPPORTZIP-EXT-001.md`
- `docs/status/trace-index.json`

## History evidence
- `git status --porcelain`
- `git fetch --all --prune`
- `git switch -c feature/g10-8-supportzip-ext-001`
- `git rev-parse HEAD`
- `git log -n 40 --oneline`
- `git branch -vv`

## Discovery evidence
- SupportZip implementation location:
  - `core/crates/diagnostics/src/support_zip.rs`
  - `core/crates/diagnostics/src/cli.rs`
  - `docs/specs/diagnostics/support_zip.md`
- Redaction/consent/privacy rules:
  - `core/crates/security/src/redaction.rs`
  - `docs/specs/diagnostics/privacy.md`
- Existing CI artifact copy path for support zip:
  - `scripts/ci/collect_artifacts.sh` (`build/support_zip` copy)

## Local verification
- `cargo test -p craftcad_diagnostics --manifest-path core/Cargo.toml`
- `cargo test -p craftcad_diagnostics --test support_zip_repro_bundle --manifest-path core/Cargo.toml`
- `cargo test -p craftcad_diagnostics --test support_zip_repro_bundle --manifest-path core/Cargo.toml -- --nocapture`
- `python -m json.tool docs/status/trace-index.json >/dev/null`
- `./scripts/ci/run_all.sh`

## SupportZip content check
- Verified generated ZIP contains:
  - `repro/ssot_snapshot.json`
  - `repro/derived_hashes.json`
  - `repro/env.json`
- Verified `derived_hashes.json` includes keys:
  - `projection_front`, `projection_top`, `projection_side`, `estimate`, `fastener_bom`, `mfg_hints`, `viewpack`
- Verified `ssot_snapshot.json` redaction heuristic by rejecting absolute path markers in test content.

## Self-check
- Allowlist respected (`docs/**`, `core/**`, `tests/**`, `scripts/**`).
- No deletions.
- `trace-index.json` validated with `json.tool`.
