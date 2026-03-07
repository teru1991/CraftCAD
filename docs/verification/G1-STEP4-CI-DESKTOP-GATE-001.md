# G1-STEP4-CI-DESKTOP-GATE-001 Verification

## Summary
- `scripts/ci/run_all.sh` now treats Desktop as an explicit CI gate with two deterministic branches:
  - Qt6 available: run desktop build + fixed smoke list + per-smoke JSON validation.
  - Qt6 unavailable: emit reasoned `[SKIP]` and continue CI.
- `run_all.sh` remains CI SSOT for desktop decision/execution logic.
- On any failing step, CI now bundles failure artifacts into `build/ci_artifacts/<step>.tar.gz`.

## Changed files
- `scripts/ci/run_all.sh`
- `scripts/ci/validate_smoke_json.py`
- `apps/desktop/src/main.cpp`
- `.github/workflows/ci.yml`
- `docs/specs/product/testing/e2e-flows.md`
- `docs/verification/G1-STEP4-CI-DESKTOP-GATE-001.md`
- `docs/status/trace-index.json`

## History evidence
- `git status --porcelain`: clean before edits.
- `git fetch --all --prune`: success.
- `git switch -c feature/g1-step4-ci-desktop-gate-001`: branch created.
- `git rev-parse HEAD`: `3f450535e461247b9551eb9ac956b78d10399ac4`.
- `git log -n 30 --oneline`: reviewed recent series.
- `git branch -vv`: verified active branch.

## Workflow scan evidence
- `ls -la .github/workflows`: existing `ci.yml`, `rust-ci.yml`, `security.yml` confirmed.
- `rg -n "run_all\.sh|cargo test|cmake|qt" .github/workflows -S`: `ci.yml` already called unified CI entrypoint; artifact upload updated for `build/ci_artifacts/**`.
- `sed -n '1,240p' scripts/ci/run_all.sh` + `rg -n "Qt|qt|desktop|cmake|SKIP" scripts/ci/run_all.sh -S`: desktop branch/skip behavior verified pre/post update.

## Local verification
- `bash -n scripts/ci/run_all.sh`: pass.
- `python3 -m py_compile scripts/ci/validate_smoke_json.py`: pass.
- `./scripts/ci/run_all.sh`:
  - Existing unrelated failures observed (`rust_test`, `e2e_shelf_flow`).
  - New failure artifact bundles printed:
    - `build/ci_artifacts/rust_test.tar.gz`
    - `build/ci_artifacts/e2e_shelf_flow.tar.gz`
  - Desktop gate path in this environment:
    - `==> desktop_gate`
    - `[SKIP] Qt6 tooling not detected; desktop build+smoke gate skipped`

## CI verification plan
- CI workflow uploads `.ci_logs/` and `build/ci_artifacts/**` using `actions/upload-artifact` with `if: always()`.
- When Qt6 is available on runner, desktop build/smokes + JSON validation are enforced by `run_all.sh`.

## Self-check
- Allowlist respected (`docs/**`, `scripts/**`, `.github/workflows/**`, `apps/desktop/**`).
- No deletions.
- `python -m json.tool docs/status/trace-index.json` must pass.
