# S16-PERF-GATE-INTEGRATION-008 Verification

## Summary
- Integrated Sprint16 perf system into mandatory PR gate (`perf_smoke`) with SSOT-driven fatality path on main/release.
- Added artifact collection script to guarantee reproducible reports.

## Changed Files
- Updated: tests/perf/perf_smoke.rs (fatality decision + CI context embedding)
- Added: scripts/ci/collect_artifacts.sh
- Updated: scripts/ci/run_all.sh (always collect artifacts at end)
- Updated: core/serialize/tests/spec_ssot_lint.rs (policy sanity check)
- Updated: core/crates/perf/src/budgets.rs (policy parsing + accessor)
- Updated: docs/specs/perf/budgets.json (policy contract clarity in notes; budget values unchanged)
- Updated: docs/status/trace-index.json
- Added: docs/verification/S16-PERF-GATE-INTEGRATION-008.md

## Preflight Evidence
- git status:
```text
On branch feature/s16-perf-gate-integration-008
Changes not staged for commit:
  modified:   core/crates/perf/src/budgets.rs
  modified:   core/serialize/tests/spec_ssot_lint.rs
  modified:   docs/specs/perf/budgets.json
  modified:   docs/status/trace-index.json
  modified:   scripts/ci/run_all.sh
  modified:   tests/perf/perf_smoke.rs
Untracked files:
  .ci_logs/
  artifacts/
  scripts/ci/collect_artifacts.sh
  tests/perf/artifacts/
```
- git log -n 30 --oneline: captured in terminal evidence.
- git branch -vv:
```text
* feature/s16-perf-gate-integration-008 5a9e376 Add deterministic cache crate, render IR, Perf SSOT, perf benches, datasets and associated tests
  work                                  5a9e376 Add deterministic cache crate, render IR, Perf SSOT, perf benches, datasets and associated tests
```

## Test Evidence
- `cargo test --manifest-path core/Cargo.toml -p craftcad_perf --features perf --test perf_smoke` => PASS
- `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize --test spec_ssot_lint ssot_perf_budgets_json_is_valid_and_consistent` => PASS
- `scripts/ci/collect_artifacts.sh artifacts && find artifacts -maxdepth 3 -type f` => PASS
- `scripts/ci/run_all.sh` => FAIL on existing `rust_fmt`, but `perf_smoke` PASS and artifacts collected.

### run_all.sh log excerpt (including perf_smoke)
```text
==> rust_fmt
[FAIL] rust_fmt (exit 1)
...
==> perf_smoke
[PASS] perf_smoke
[collect_artifacts] copied: tests/perf/artifacts -> /workspace/CraftCAD/artifacts/tests_perf_artifacts
[collect_artifacts] skip (missing): benches/artifacts
[collect_artifacts] copied: docs/specs/perf/budgets.json -> /workspace/CraftCAD/artifacts/budgets.json
[collect_artifacts] copied: tests/datasets/manifest.json -> /workspace/CraftCAD/artifacts/datasets_manifest.json
[collect_artifacts] done. OUT_DIR=/workspace/CraftCAD/artifacts
```

## Behavior Matrix
- PR (`GITHUB_EVENT_NAME=pull_request`): budgets exceed -> WARN (report emitted)
- main/release (`GITHUB_REF_NAME=main` or `release/*`): budgets exceed -> FAIL (report path printed)
- local: budgets exceed -> WARN (report emitted)

## Safety / Allowlist Self-check
- Only allowed paths modified: YES
- No deletions: YES
