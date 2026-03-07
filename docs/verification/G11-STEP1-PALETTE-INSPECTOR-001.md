# G11-STEP1-PALETTE-INSPECTOR-001 Verification

## Summary
- Added Desktop command palette (`Ctrl+K`/`/`) with searchable execution for Open/Save and smoke commands.
- Added selection-linked Inspector dock that reads Part fields via FFI and persists Name/Quantity edits to `ssot_v1`.
- Added headless smoke `--smoke-inspector-edit` and CI wiring.

## Chosen implementation
- Reused existing `View3dWidget::selectedPartChanged` as the selection event source.
- Implemented persistence through `craftcad_ffi_desktop` mutation APIs so GUI and smoke use the same write path.

## Changed files
- docs/specs/product/feature-scope.md
- docs/specs/product/contracts/jobs.md
- docs/specs/system/ffi_contract.md
- core/ffi_desktop/src/lib.rs
- core/ffi_desktop/include/craftcad_desktop_ffi.h
- apps/desktop/src/ffi/craftcad_ffi.h
- apps/desktop/src/main.cpp
- scripts/ci/run_all.sh
- tests/fixtures/inspector_smoke_expected.json
- docs/status/trace-index.json

## History evidence
- `git status --porcelain`
- `git rev-parse HEAD`
- `git log -n 5 --oneline`
- `git branch -vv`

## Local verification
- `cargo test -p craftcad_ffi_desktop` ✅
- `./scripts/build_desktop.sh` ⚠️ failed at CMake configure because Qt6 development package is unavailable in this environment (`Qt6Config.cmake` not found).
- `./scripts/run_desktop.sh --smoke-inspector-edit build/desktop/rules_edge_smoke_fixture.diycad` ⚠️ skipped by missing desktop binary (build blocked by missing Qt6).
- `./scripts/ci/run_all.sh` ⚠️ completed with existing unrelated suite failures (`rust_fmt`, `rust_test`, `e2e_shelf_flow`) and environment-dependent desktop skips.

## Self-check
- Allowlist respected (docs/core/apps/desktop/tests/scripts only).
- No file deletions.
- `python -m json.tool docs/status/trace-index.json >/dev/null` executed.
