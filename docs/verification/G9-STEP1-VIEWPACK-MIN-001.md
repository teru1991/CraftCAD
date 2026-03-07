# G9-STEP1-VIEWPACK-MIN-001 Verification

## Summary
- Implemented `ViewerPackV1` minimal offline pack with deterministic manifest and artifact hashing.
- Wired project save path to auto-generate and embed `viewer_pack_v1.json` from SSOT-derived artifacts.
- Added hash verification and required-artifact checks (`Not generated` / `Corrupt pack` semantics).

## Viewer approach choice
- Chosen approach: **Option B** (headless inspector CLI).
- Evidence:
  - `apps/mobile` exists, but this task prioritizes deterministic CI-verifiable offline checks without Flutter runtime assumptions.
  - Added `craftcad-viewpack-inspect` CLI to report artifact status without recomputation.
  - Touched `apps/mobile/README.md` to align mobile offline contract with `viewer_pack_v1` semantics.

## Changed files
- docs/specs/product/contracts/derived-artifacts.md
- docs/specs/product/contracts/master-model.md
- core/Cargo.toml
- core/crates/craftcad_viewpack/**
- core/crates/craftcad_viewpack_inspect/**
- core/crates/diycad_project/Cargo.toml
- core/crates/diycad_project/src/lib.rs
- core/crates/diycad_ffi/src/lib.rs
- apps/mobile/README.md
- scripts/ci/run_all.sh
- docs/status/trace-index.json

## History evidence
- `git status --porcelain`
- `git fetch --all --prune`
- `git switch -c feature/g9-step1-viewpack-min-001`
- `git rev-parse HEAD`
- `git log -n 30 --oneline`
- `git branch -vv`
- `rg -n "project_file|schema_version|serde_json::to_vec|zip|pack|manifest" core/crates -S`
- `ls -la apps/mobile`
- `rg -n "flutter|dart|mobile" apps/mobile -S`
- `rg -n "projection_lite|estimate_lite|fastener_bom_lite|mfg_hints_lite" core/crates -S`

## Local verification
- `cargo fmt --all` (pass)
- `cargo test -p craftcad_viewpack` (pass)
- `cargo test -p diycad_project -p craftcad_viewpack_inspect` (pass)
- `python3 scripts/ci/create_rules_edge_smoke_fixture.py build/desktop/rules_edge_smoke_fixture.diycad` (pass)
- `cargo run -q -p craftcad_viewpack_inspect --bin craftcad-viewpack-inspect --manifest-path core/Cargo.toml -- build/desktop/rules_edge_smoke_fixture.diycad` (pass; status=NOT_GENERATED for fixture without viewer pack)
- `./scripts/ci/run_all.sh` (runs and exits non-zero due existing unrelated failures: `rust_test`, `e2e_shelf_flow`)
- `python -m json.tool docs/status/trace-index.json >/dev/null` (pass)

## Self-check
- Allowlist respected.
- No deletions performed.
- Offline/no-recompute contract documented and enforced in verifier/inspector.
