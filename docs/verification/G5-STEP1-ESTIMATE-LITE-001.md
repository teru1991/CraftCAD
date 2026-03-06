# Verification: G5-STEP1-ESTIMATE-LITE-001

## Summary
Implemented Step1 EstimateLite derived artifact over SSOT:
- New deterministic estimate core computes per-material totals from Part manufacturing outlines.
- Added desktop headless smoke flag `--smoke-estimate-lite` using Rust FFI endpoint.
- Added CI smoke hook (Qt-available branch) and deterministic tests.

## Changed files
- docs/specs/product/feature-scope.md
- docs/specs/product/contracts/derived-artifacts.md
- core/Cargo.toml
- core/crates/craftcad_estimate_lite/Cargo.toml
- core/crates/craftcad_estimate_lite/src/lib.rs
- core/crates/craftcad_estimate_lite/tests/estimate_lite.rs
- core/ffi_desktop/Cargo.toml
- core/ffi_desktop/include/craftcad_desktop_ffi.h
- core/ffi_desktop/src/lib.rs
- apps/desktop/src/ffi/craftcad_ffi.h
- apps/desktop/src/main.cpp
- scripts/ci/run_all.sh
- docs/specs/system/ffi_contract.md
- docs/status/trace-index.json (tasks["G5-STEP1-ESTIMATE-LITE-001"] only)

## History evidence
### Preflight
- `git status --porcelain`

```text
<empty>
```

- `git rev-parse HEAD`

```text
2e4850dc75ff5e84f41e660ba1225c9632409480
```

- `git log -n 30 --oneline`

```text
2e4850d feat(drawing): add projection-lite (AABB->2D) + deterministic hashes + smoke flag (G4-STEP1-PROJECTION-001)
1ea1224 feat(desktop): add minimal 3D viewport + PartId selection + smoke flag (G3-STEP1-3D-VIEW-001)
cf55a36 core(ssot): persist Part/Material/FeatureGraph v1 in project file (G2-STEP1-SSOT-SAVE-001)
...
```

- `git branch -vv`

```text
* feature/g5-step1-estimate-lite-001 2e4850d feat(drawing): add projection-lite (AABB->2D) + deterministic hashes + smoke flag (G4-STEP1-PROJECTION-001)
  feature/g4-step1-projection-001    2e4850d feat(drawing): add projection-lite (AABB->2D) + deterministic hashes + smoke flag (G4-STEP1-PROJECTION-001)
  feature/g3-step1-3d-view-001       1ea1224 feat(desktop): add minimal 3D viewport + PartId selection + smoke flag (G3-STEP1-3D-VIEW-001)
```

### Discovery findings
Executed scans:
- `rg -n "BOM|bill of materials|part_bom|estimate|cutlist|material" core/crates docs/specs -S`
- `rg -n "manufacturing_outline_2d|ManufacturingOutline2dV1" core/crates -S`
- `rg -n -- "--smoke" apps/desktop/src`

Chosen integration points:
- SSOT source and outline fields are in `craftcad_ssot` and loaded via `diycad_project`.
- Existing desktop smoke entrypoint in `apps/desktop/src/main.cpp` is reused for `--smoke-estimate-lite`.
- FFI bridge in `core/ffi_desktop/src/lib.rs` already resolves project→ssot and is extended with estimate-lite endpoint.

## Local verification
- Estimate core tests:

```bash
cargo test -p craftcad_estimate_lite
```

Result:

```text
pass (4 tests: determinism/order, quantity effect, missing outline behavior, NaN/Inf handling)
```

- Desktop FFI tests:

```bash
cargo test -p craftcad_ffi_desktop
```

Result:

```text
pass (includes estimate hash determinism unit + ffi contract lint + existing tests)
```

- Desktop build:

```bash
./scripts/build_desktop.sh
```

Result:

```text
Rust release build succeeded; CMake configure failed due missing Qt6Config.cmake in this environment.
```

- Desktop estimate smoke:

```bash
python3 scripts/ci/create_view3d_smoke_fixture.py build/desktop/view3d_smoke_fixture.diycad
./scripts/run_desktop.sh --smoke-estimate-lite build/desktop/view3d_smoke_fixture.diycad
```

Result:

```text
fixture generated; run failed with "Desktop binary not found" because desktop build did not complete (Qt missing).
```

- CI run:

```bash
./scripts/ci/run_all.sh
```

Result summary:

```text
desktop section skipped due missing Qt6 dev package; existing rust_fmt failure remains (encountered diff marker).
```

## Self-check
- Allowlist respected: YES
- No deletions: YES
- json.tool trace-index: PASS

## Notes
- Determinism is fixed via stable material ordering (`material_id`) and canonical JSON hash.
- Step1 estimate area uses manufacturing bbox area with missing outline treated as 0 while preserving parts_count.
