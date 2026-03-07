# G1-STEP2-DESKTOP-RUN-001 Verification

## Summary
- Official desktop runner route is fixed to `bash ./scripts/run_desktop.sh [args...]` for normal and smoke execution.
- No-binary policy is documented: artifacts under `build/` and `target/` are runtime outputs only and must not be committed.
- Runner now provides text-only runtime diagnostics (`--print-env`, Linux `ldd` missing-lib checks, macOS `otool -L` hints, Qt plugin path candidates).

## Changed files
- `scripts/run_desktop.sh`
- `scripts/ci/run_all.sh`
- `apps/desktop/README.md`
- `README.md`
- `docs/status/trace-index.json`
- `docs/verification/G1-STEP2-DESKTOP-RUN-001.md`

## History evidence
- `git status --porcelain` (before changes): clean (no output).
- `git fetch --all --prune`: success.
- `git switch -c feature/g1-step2-desktop-run-001`: created/switch success.
- `git rev-parse HEAD`: `9f792aacbd89d3be739867b675f67f2378dc0c07`.
- `git log -n 30 --oneline`: reviewed recent history.
- `git branch -vv`: confirmed active branch.

## Runtime/dependency preflight evidence
- `file build/desktop/craftcad_desktop`: command unavailable in environment (`bash: command not found: file`).
- `ldd build/desktop/craftcad_desktop`: binary missing before build (`No such file or directory`).
- `otool -L ...`: not available on Linux host.
- `rg -n "LD_LIBRARY_PATH|DYLD_LIBRARY_PATH|QT_PLUGIN_PATH|PATH=" ...`: no previous runtime env handling in desktop scripts/docs before this task.

## Local verification
- `./scripts/build_desktop.sh`
  - Rust release build succeeded for `craftcad_ffi_desktop`.
  - CMake configure failed because Qt6 package config was unavailable (`Qt6Config.cmake` not found).
- `bash ./scripts/run_desktop.sh --print-env`
  - Succeeded and printed `ROOT_DIR`, `BUILD_DIR`, `BIN`, `OS_NAME`, `LD_LIBRARY_PATH`, `DYLD_LIBRARY_PATH`, `QT_PLUGIN_PATH`.
- `bash ./scripts/run_desktop.sh --smoke-view3d tests/fixtures/view3d_smoke_fixture.json`
  - Fails fast with explicit guidance when desktop binary is missing, including build command hint.
- `./scripts/ci/run_all.sh`
  - Runner path updated to `bash ./scripts/run_desktop.sh ...` and includes `desktop_print_env` step.
  - Current environment had existing failing non-desktop checks (`rust_test`, `e2e_shelf_flow`); desktop build is still Qt-dependent.

## Git cleanliness proof
- `git status --porcelain` before task: clean.
- Build/CI generated `.ci_logs/` and `artifacts/` as untracked runtime outputs only.
- No binaries/shared libraries/app bundles/symlinks were added to Git.

## Self-check
- Allowlist respected: modified files are under `docs/**`, `apps/desktop/**`, and `scripts/**` only.
- No file deletions performed.
- No tracked binaries/shared libraries/app bundles/symlinks.
- `python -m json.tool docs/status/trace-index.json`: pass.
