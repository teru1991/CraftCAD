# G8-STEP1-MFG-HINTS-001 Verification

## Summary
- Implemented `ManufacturingHintsLiteV1` derivation from SSOT `ScrewFeature` params (pilot hole, countersink, depth note text) with deterministic item ordering and SHA-256 hash.
- Added a Step1 integration artifact `FastenerBomWithHintsLiteV1` that aggregates fastener BOM-lite entries and manufacturing hints without mutating existing BOM schemas.
- Added desktop FFI + headless smoke route to compute and report mfg hints hash/count.

## Changed files
- docs/specs/product/contracts/derived-artifacts.md
- docs/specs/product/contracts/master-model.md
- docs/specs/product/contracts/reason-codes.md
- core/Cargo.toml
- core/crates/craftcad_mfg_hints_lite/**
- core/ffi_desktop/Cargo.toml
- core/ffi_desktop/include/craftcad_desktop_ffi.h
- core/ffi_desktop/src/lib.rs
- apps/desktop/src/ffi/craftcad_ffi.h
- apps/desktop/src/main.cpp
- scripts/ci/create_rules_edge_smoke_fixture.py
- scripts/ci/run_all.sh
- docs/specs/system/ffi_contract.md
- docs/status/trace-index.json

## History evidence
- `git status --porcelain`
- `git fetch --all --prune`
- `git switch -c feature/g8-step1-mfg-hints-001`
- `git rev-parse HEAD`
- `git log -n 30 --oneline`
- `git branch -vv`
- `rg -n "hint|instruction|annotation|note|countersink|pilot" core/crates docs/specs -S`
- `rg -n "ScrewParamsV1|spec_name|pitch_mm|edge_offset_mm" core/crates -S`

## Local verification
- `cargo fmt --all` (pass)
- `cargo test -p craftcad_mfg_hints_lite` (pass)
- `cargo test -p craftcad_ffi_desktop` (pass)
- `./scripts/build_desktop.sh` (fails in environment: Qt6Config.cmake missing)
- `python3 scripts/ci/create_rules_edge_smoke_fixture.py build/desktop/rules_edge_smoke_fixture.diycad` (pass)
- `./scripts/run_desktop.sh --smoke-mfg-hints-lite build/desktop/rules_edge_smoke_fixture.diycad` (blocked because desktop binary missing due Qt build failure)
- `./scripts/ci/run_all.sh` (runs and exits non-zero due existing unrelated failures: `rust_test`, `e2e_shelf_flow`)
- `python -m json.tool docs/status/trace-index.json >/dev/null` (pass)

## Self-check
- Allowlist respected (all files under allowed paths).
- No deletions performed.
- JSON lint check passed for `docs/status/trace-index.json`.
