# Verification: G4-STEP1-PROJECTION-001

## Summary
Implemented Step1 projection-lite pipeline from SSOT Part AABB to deterministic 2D sheet hashes.
- Added `craftcad_projection_lite` core crate (`front/top/side`, AABB→rectangle outline, canonical hash).
- Added desktop smoke CLI `--smoke-projection-lite` backed by new Rust FFI `craftcad_projection_lite_hashes`.
- Added CI desktop smoke step for projection-lite (Qt-available branch) reusing existing fixture.
- Extended product artifact/testing docs with Step1 projection-lite and determinism expectations.

## Changed files
- docs/specs/product/contracts/derived-artifacts.md
- docs/specs/product/testing/e2e-flows.md
- core/Cargo.toml
- core/crates/craftcad_projection_lite/Cargo.toml
- core/crates/craftcad_projection_lite/src/lib.rs
- core/crates/craftcad_projection_lite/tests/projection_lite.rs
- core/ffi_desktop/Cargo.toml
- core/ffi_desktop/include/craftcad_desktop_ffi.h
- core/ffi_desktop/src/lib.rs
- apps/desktop/src/ffi/craftcad_ffi.h
- apps/desktop/src/main.cpp
- scripts/ci/run_all.sh
- docs/specs/system/ffi_contract.md
- docs/status/trace-index.json (tasks["G4-STEP1-PROJECTION-001"] only)

## History evidence
### Preflight
- `git status --porcelain`

```text
<empty>
```

- `git rev-parse HEAD`

```text
1ea12249de982faaffb7b0c921392b203825a3ee
```

- `git log -n 30 --oneline`

```text
1ea1224 feat(desktop): add minimal 3D viewport + PartId selection + smoke flag (G3-STEP1-3D-VIEW-001)
cf55a36 core(ssot): persist Part/Material/FeatureGraph v1 in project file (G2-STEP1-SSOT-SAVE-001)
8d635bb build(desktop): unify release build route and CI to use scripts/build_desktop.sh (G1-STEP1-DESKTOP-BUILD-001)
...
```

- `git branch -vv`

```text
* feature/g4-step1-projection-001    1ea1224 feat(desktop): add minimal 3D viewport + PartId selection + smoke flag (G3-STEP1-3D-VIEW-001)
  feature/g3-step1-3d-view-001       1ea1224 feat(desktop): add minimal 3D viewport + PartId selection + smoke flag (G3-STEP1-3D-VIEW-001)
  feature/g2-step1-ssot-save-001     cf55a36 core(ssot): persist Part/Material/FeatureGraph v1 in project file (G2-STEP1-SSOT-SAVE-001)
```

### Discovery scans and chosen integration points
Executed scans:
- `rg -n "drawing|sheet|pdf|export_drawing|drawing_model|drawing_style" core/crates -S`
- `rg -n "projection|projector|orthographic|isometric|viewSpec|camera" core/crates -S`
- `rg -n -- "--smoke|smoke_" apps/desktop/src`

Chosen integration points:
- Projection core: new crate under `core/crates/` to keep deterministic logic in Rust and test in isolation.
- Desktop smoke: `apps/desktop/src/main.cpp` already had `--smoke-view3d`, so `--smoke-projection-lite` was added in same path.
- FFI bridge: `core/ffi_desktop/src/lib.rs` is existing ABI surface for desktop smoke data; added projection hash endpoint there.

## Local verification
- Core projection tests:

```bash
cargo test -p craftcad_projection_lite
```

Result:

```text
pass (3 tests: determinism with shuffled input, rectangle closure, NaN/Inf deterministic handling)
```

- FFI tests:

```bash
cargo test -p craftcad_ffi_desktop
```

Result:

```text
pass (view3d + projection hash determinism tests + ffi contract lint + json roundtrip)
```

- Desktop build route:

```bash
./scripts/build_desktop.sh
```

Result:

```text
Rust release build succeeded; CMake configure failed due missing Qt6Config.cmake in this environment.
```

- Projection smoke (fixture + desktop smoke flag):

```bash
python3 scripts/ci/create_view3d_smoke_fixture.py build/desktop/view3d_smoke_fixture.diycad
./scripts/run_desktop.sh --smoke-projection-lite build/desktop/view3d_smoke_fixture.diycad
```

Result:

```text
fixture generated; run failed with "Desktop binary not found" because desktop build did not complete (Qt missing).
```

- CI script:

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
- Determinism path is fixed in Rust (`sort_by_key(part_id)` + canonical JSON hash).
- Projection-lite intentionally uses AABB rectangle outlines only (full projection/mesh to later tasks).
