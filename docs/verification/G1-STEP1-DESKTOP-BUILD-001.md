# Verification: G1-STEP1-DESKTOP-BUILD-001

## Summary
This task unifies the **official desktop build/run route**:
- Rust FFI (`craftcad_ffi_desktop`) is built in **release**.
- CMake desktop app is built in **release** and links against `core/target/release`.
- CI uses the same `scripts/build_desktop.sh` route to avoid divergence.

## Changed files
- scripts/build_desktop.sh
- scripts/run_desktop.sh (new)
- apps/desktop/README.md
- README.md
- scripts/ci/run_all.sh
- docs/status/trace-index.json (tasks["G1-STEP1-DESKTOP-BUILD-001"] only)

## History evidence
### Preflight
- git status --porcelain:

```text
<empty>
```

- git rev-parse HEAD:

```text
eb110ea0136b1994bc0a2dfa4f61414d8d83d3a8
```

- git log -n 30 --oneline:

```text
eb110ea Merge pull request #104 from teru1991/codex/add-implementation-readiness-ssot
17e0feb docs(product): add implementation readiness SSOT for final pillars (P00-PREP-SSOT-001)
c98b2a2 Merge pull request #103 from teru1991/codex/establish-ux-ssot-for-sprint-20-mabxir
9b8f524 Merge branch 'main' into codex/establish-ux-ssot-for-sprint-20-mabxir
686f59e S20-STEP5: integrate job UX controller with modes/error/onboarding
...
```

- git branch -vv:

```text
* feature/g1-step1-desktop-build-001 eb110ea Merge pull request #104 from teru1991/codex/add-implementation-readiness-ssot
  work                               eb110ea Merge pull request #104 from teru1991/codex/add-implementation-readiness-ssot
```

### Build-route scan
- grep FFI_LIB_DIR / craftcad_ffi_desktop references:

```text
apps/desktop/CMakeLists.txt:45:if (NOT DEFINED FFI_LIB_DIR)
apps/desktop/CMakeLists.txt:46:  set(FFI_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../core/target/debug")
apps/desktop/CMakeLists.txt:51:  PATHS ${FFI_LIB_DIR}
apps/desktop/CMakeLists.txt:56:  message(FATAL_ERROR "craftcad_ffi_desktop library not found. Build Rust FFI first and set -DFFI_LIB_DIR=<path>.")
apps/desktop/README.md:27:cmake -S apps/desktop -B build/desktop -DFFI_LIB_DIR=$(pwd)/core/target/debug
scripts/build_desktop.sh:10:  cargo build -p craftcad_ffi_desktop
scripts/ci/run_all.sh:66:    run_step rust_ffi_desktop "${ROOT_DIR}/core" cargo build -p craftcad_ffi_desktop
scripts/ci/run_all.sh:68:    run_step rust_ffi_build "${ROOT_DIR}/core" cargo build -p craftcad_ffi_desktop
```

## Local verification
- Desktop build:

```bash
./scripts/build_desktop.sh
```

Result:

```text
[craftcad] building Rust desktop FFI (release)…
...
Finished `release` profile [optimized] target(s) in 4m 05s
[craftcad] configuring CMake (release)…
CMake Error at CMakeLists.txt:8 (find_package):
  ... Could not find a package configuration file provided by "Qt6" ...
```

- Desktop run:

```bash
./scripts/run_desktop.sh --help
```

Result:

```text
[craftcad] Desktop binary not found: /workspace/CraftCAD/build/desktop/craftcad_desktop
[craftcad] Build first: /workspace/CraftCAD/scripts/build_desktop.sh
```

- CI script (note: desktop build may skip if Qt missing):

```bash
./scripts/ci/run_all.sh
```

Result summary:

```text
rust_fmt failed (existing formatting issue: "error: encountered diff marker")
most gates passed; desktop section skipped due missing Qt6 dev package:
[SKIP] Qt6 development package not available; skipping desktop CMake build
```

## Self-check
- Allowlist respected: YES
- No deletions: YES
- json.tool trace-index: PASS
- scripts executable bit set: PASS

## Notes
- Why release→release: prevents CMake linking against debug default path and avoids mismatch.
- Why CI uses scripts/build_desktop.sh: single source of truth for build steps.

