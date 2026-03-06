# G7-STEP1-RULES-EDGE-001 Verification

## Summary
- Added `craftcad_rules_engine` with deterministic RuleReport, edge distance fatal evaluation, and preflight gate API.
- Added `craftcad_screw_lite` to evaluate ScrewFeature points from SSOT feature graph params.
- Added desktop FFI hooks and CLI smoke flags for rule report and export preflight blocking check.
- Added CI fixture generation and smoke steps for rules-edge + negative preflight behavior.

## Changed files
- docs/specs/product/contracts/rules-wood.md
- docs/specs/product/contracts/reason-codes.md
- docs/specs/product/contracts/jobs.md
- core/Cargo.toml
- core/crates/craftcad_screw_lite/**
- core/crates/craftcad_rules_engine/**
- core/ffi_desktop/Cargo.toml
- core/ffi_desktop/include/craftcad_desktop_ffi.h
- core/ffi_desktop/src/lib.rs
- apps/desktop/src/ffi/craftcad_ffi.h
- apps/desktop/src/main.cpp
- scripts/ci/create_rules_edge_smoke_fixture.py
- scripts/ci/run_all.sh
- tests/fixtures/rules_edge_smoke_fixture.json
- docs/specs/system/ffi_contract.md
- docs/status/trace-index.json

## History evidence
- `git status --porcelain`
- `git fetch --all --prune`
- `git switch -c feature/g7-step1-rules-edge-001`
- `git rev-parse HEAD`
- `git log -n 30 --oneline`
- `git branch -vv`
- `rg -n "export|print|preflight|validate|gate|ReasonCode" core/crates -S`
- `rg -n "limits|guard|validate|rule" core/crates/docs/specs -S`
- `rg -n "ScrewPoint|eval_screw_points|craftcad_screw_lite" core/crates -S`

## Local verification
- `cargo fmt --all` (pass)
- `cargo test -p craftcad_rules_engine` (pass)
- `cargo test -p craftcad_ffi_desktop` (pass)
- `./scripts/build_desktop.sh` (fails in this environment: Qt6Config.cmake not found)
- `python3 scripts/ci/create_rules_edge_smoke_fixture.py build/desktop/rules_edge_smoke_fixture.diycad` (pass)
- `./scripts/run_desktop.sh --smoke-rules-edge build/desktop/rules_edge_smoke_fixture.diycad` (blocked due missing desktop binary)
- `./scripts/run_desktop.sh --smoke-export-preflight build/desktop/rules_edge_smoke_fixture.diycad` (blocked due missing desktop binary)
- `./scripts/ci/run_all.sh` (completed with existing unrelated failures in suite: rust_test and e2e_shelf_flow)
- `python -m json.tool docs/status/trace-index.json >/dev/null` (pass)

## Self-check
- Allowlist: all edits are within allowed paths.
- No deletion performed.
- `python -m json.tool docs/status/trace-index.json >/dev/null` passed.
