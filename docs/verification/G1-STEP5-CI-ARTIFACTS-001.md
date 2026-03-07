# G1-STEP5-CI-ARTIFACTS-001 Verification

## Summary
- CI step failures now always collect reproducible bundles under `build/ci_artifacts/<step_name>/` via `scripts/ci/collect_artifacts.sh`.
- Collected targets are unified (stdout/stderr + step log + SupportZip best-effort + `e2e_failures` + `determinism_failures` + smoke JSON best-effort).
- `scripts/ci/artifacts_index.py` always generates `build/ci_artifacts/index.json` so artifact contents are discoverable immediately.
- Workflow upload path keeps `build/ci_artifacts/**` under `if: always()` for success/failure parity.

## Changed files
- `scripts/ci/run_all.sh`
- `scripts/ci/collect_artifacts.sh`
- `scripts/ci/artifacts_index.py`
- `.github/workflows/ci.yml`
- `docs/specs/product/testing/e2e-flows.md`
- `docs/verification/G1-STEP5-CI-ARTIFACTS-001.md`
- `docs/status/trace-index.json`

## History evidence
- `git status --porcelain` (before): clean.
- `git fetch --all --prune`: success.
- `git switch -c feature/g1-step5-ci-artifacts-001`: branch created.
- `git rev-parse HEAD`: `8ea3fc334d095308c3ac66e512c801f9a9ae2c3e`.
- `git log -n 30 --oneline`: recent history reviewed.
- `git branch -vv`: branch state confirmed.

## Existing behavior scan evidence
- `rg -n "ci_artifacts|artifact|upload|SupportZip|e2e_failures|determinism_failures" -S scripts .github docs core`
  confirmed prior mixed artifact handling and workflow upload location.

## Local verification
- `bash -n scripts/ci/run_all.sh`: pass.
- `bash -n scripts/ci/collect_artifacts.sh`: pass.
- `python3 -m py_compile scripts/ci/artifacts_index.py scripts/ci/validate_smoke_json.py`: pass.
- `./scripts/ci/run_all.sh`:
  - existing unrelated failures remained (`rust_test`, `e2e_shelf_flow`).
  - each failure emitted collector output and wrote into:
    - `build/ci_artifacts/rust_test/`
    - `build/ci_artifacts/e2e_shelf_flow/`
  - desktop gate still gave explicit skip reason in this environment.
- `python -m json.tool build/ci_artifacts/index.json >/dev/null`: pass.
- `python -m json.tool docs/status/trace-index.json >/dev/null`: pass.

## CI verification plan
- `actions/upload-artifact` uploads `.ci_logs/` and `build/ci_artifacts/**` with `if: always()`.
- Artifact index contract: inspect `build/ci_artifacts/index.json` first, then drill into per-step folders.

## Self-check
- Allowlist respected (`docs/**`, `scripts/**`, `.github/workflows/**`).
- No deletions.
- Repro-oriented collection is best-effort and non-fatal for missing paths.
