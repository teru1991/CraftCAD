# G1-7-DESKTOP-DIST-LAYOUT-001 Verification

## Summary
- Added a fixed Desktop resource contract and dist layout so both dev and dist runs share one discovery rule.
- Added Desktop resource locator + `--smoke-resources` preflight that emits one-line JSON with `RESOURCE_ROOT_NOT_FOUND` / `RESOURCE_MISSING` on failure.
- Added build-time resources sync into `build/desktop/resources` and runner-side `CRAFTCAD_RESOURCE_ROOT` defaulting for dist-like dev runs.
- Wired CI desktop branch to execute resources smoke and JSON validation when Qt6 is available.

## Changed files
- `docs/specs/product/contracts/resources.md`
- `apps/desktop/CMakeLists.txt`
- `apps/desktop/src/resources/resource_locator.h`
- `apps/desktop/src/resources/resource_locator.cpp`
- `apps/desktop/src/main.cpp`
- `apps/desktop/resources/templates/.keep`
- `apps/desktop/resources/samples/.keep`
- `apps/desktop/resources/fonts/.keep`
- `apps/desktop/resources/icons/.keep`
- `scripts/build_desktop.sh`
- `scripts/run_desktop.sh`
- `scripts/ci/run_all.sh`
- `tests/desktop/resources_preflight_smoke.sh`
- `docs/verification/G1-7-DESKTOP-DIST-LAYOUT-001.md`
- `docs/status/trace-index.json`

## History evidence
- `git status --porcelain` (before): clean.
- `git fetch --all --prune`: success.
- `git switch -c feature/g1-7-desktop-dist-layout-001`: success.
- `git rev-parse HEAD`: `ba782c3d71fd029b43545365b7a858157b9a470f`.
- `git log -n 40 --oneline`: reviewed recent baseline.
- `git branch -vv`: branch alignment confirmed.

## Resource inventory evidence
- `find apps/desktop -maxdepth 3 -type d -name assets -o -name resources -o -name templates -o -name samples`
  - before change, only `apps/desktop/i18n/resources` existed.
- `rg -n "template|sample|font|icon|resource" apps/desktop/src core/crates -S`
  - prior desktop sample paths included source-relative references such as `apps/desktop/assets/samples` and no unified dist resource locator.

## Local verification
- `python -m json.tool docs/status/trace-index.json >/dev/null`: pass.
- `./scripts/build_desktop.sh`:
  - Rust release FFI build passed.
  - CMake configure failed in this environment because Qt6 package config was unavailable.
- `bash ./scripts/run_desktop.sh --smoke-resources`:
  - desktop binary missing in this environment (expected without successful Qt build), script failed fast with build hint.
- `./scripts/ci/run_all.sh`:
  - existing unrelated failures remained (`rust_test`, `e2e_shelf_flow`).
  - desktop gate branch skipped with explicit Qt-not-detected reason in this environment.

## Self-check
- Allowlist respected (`docs/**`, `apps/desktop/**`, `scripts/**`, `tests/**`).
- No deletions.
- `python -m json.tool docs/status/trace-index.json` passes.
