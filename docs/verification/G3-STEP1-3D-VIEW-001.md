# Verification: G3-STEP1-3D-VIEW-001

## Summary
Implemented Step1 desktop 3D view scope:
- Rust FFI exports deterministic Part AABB boxes derived from project `ssot_v1`.
- Desktop adds a minimal `QOpenGLWidget` 3D viewport (wireframe box rendering, orbit/pan/zoom, click selection).
- Selected PartId is shown in the desktop status area.
- Added smoke CLI `--smoke-view3d <project>` to validate FFI + SSOT linkage in CI.

## Changed files
- docs/specs/product/feature-scope.md
- docs/specs/product/contracts/derived-artifacts.md
- docs/specs/system/ffi_contract.md
- core/ffi_desktop/Cargo.toml
- core/ffi_desktop/include/craftcad_desktop_ffi.h
- core/ffi_desktop/src/lib.rs
- apps/desktop/CMakeLists.txt
- apps/desktop/src/ffi/craftcad_ffi.h
- apps/desktop/src/main.cpp
- apps/desktop/src/view3d_widget.h
- apps/desktop/src/view3d_widget.cpp
- scripts/ci/run_all.sh
- scripts/ci/create_view3d_smoke_fixture.py
- docs/status/trace-index.json (tasks["G3-STEP1-3D-VIEW-001"] only)

## History evidence
### Preflight
- `git status --porcelain`

```text
<empty>
```

- `git rev-parse HEAD`

```text
cf55a36b2d662af23e17975506f478fc28ff5c43
```

- `git log -n 30 --oneline`

```text
cf55a36 core(ssot): persist Part/Material/FeatureGraph v1 in project file (G2-STEP1-SSOT-SAVE-001)
8d635bb build(desktop): unify release build route and CI to use scripts/build_desktop.sh (G1-STEP1-DESKTOP-BUILD-001)
eb110ea Merge pull request #104 from teru1991/codex/add-implementation-readiness-ssot
...
```

- `git branch -vv`

```text
* feature/g3-step1-3d-view-001       cf55a36 core(ssot): persist Part/Material/FeatureGraph v1 in project file (G2-STEP1-SSOT-SAVE-001)
  feature/g2-step1-ssot-save-001     cf55a36 core(ssot): persist Part/Material/FeatureGraph v1 in project file (G2-STEP1-SSOT-SAVE-001)
  feature/g1-step1-desktop-build-001 8d635bb build(desktop): unify release build route and CI to use scripts/build_desktop.sh (G1-STEP1-DESKTOP-BUILD-001)
```

### Discovery scans and chosen integration points
Executed:
- `rg -n "QOpenGLWidget|QOpenGLFunctions|QMatrix4x4|QOpenGL" apps/desktop/src -S`
- `rg -n "MainWindow|Dock|Inspector|Canvas" apps/desktop/src -S`
- `rg -n "extern \"C\"|ffi|FFI|craftcad_ffi_desktop" core/ffi_desktop -S`
- `rg -n "ssot_v1|SsotV1|craftcad_ssot" core/crates -S`

Findings used:
- Desktop shell and docking are in `apps/desktop/src/main.cpp`; integrated 3D dock and smoke flag there.
- FFI surface and symbol registry are in `core/ffi_desktop/src/lib.rs` and C headers (`core/ffi_desktop/include/...`, `apps/desktop/src/ffi/...`).
- SSOT availability comes from `diycad_project` (`project.ssot_v1`), enabling direct deterministic AABB artifact export.

## Local verification
- Build desktop route:

```bash
./scripts/build_desktop.sh
```

Result:

```text
Rust FFI release build: OK
CMake configure: FAIL (Qt6Config.cmake not found in this environment)
```

- Smoke fixture generation + smoke run:

```bash
python3 scripts/ci/create_view3d_smoke_fixture.py build/desktop/view3d_smoke_fixture.diycad
./scripts/run_desktop.sh --smoke-view3d build/desktop/view3d_smoke_fixture.diycad
```

Result:

```text
fixture generated: build/desktop/view3d_smoke_fixture.diycad
run_desktop failed before launch (desktop binary missing due Qt build failure)
```

- Rust FFI tests:

```bash
cargo test -p craftcad_ffi_desktop
```

Result:

```text
pass (unit tests + ffi_contract_lint + ffi_json_roundtrip)
```

- CI script:

```bash
./scripts/ci/run_all.sh
```

Result summary:

```text
desktop section skipped due missing Qt6 dev package
existing rust_fmt failure remains: "error: encountered diff marker"
```

## Self-check
- Allowlist respected: YES
- No deletions: YES
- json.tool trace-index: PASS

## Notes
- Step1 3D artifact is intentionally minimal (PartId + AABB + deterministic color).
- Selection contract is click→ray/AABB hit test→selected PartId display; mesh/picking complexity deferred to later tasks.
