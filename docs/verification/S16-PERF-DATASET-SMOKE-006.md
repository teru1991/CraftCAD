# S16-PERF-DATASET-SMOKE-006 Verification

## Summary
- Added heavy dataset SSOT entry + deterministic dataset file, and introduced perf smoke test that always emits PerfReport artifacts and runs budgets check.

## Changed Files
- Updated: tests/datasets/manifest.json
- Added: tests/datasets/heavy_sample_v1/README.md
- Added: tests/datasets/heavy_sample_v1/heavy_sample_v1.json
- Added: tests/perf/mod.rs
- Updated: tests/perf/perf_smoke.rs
- Added: core/crates/perf/tests/perf_smoke.rs
- Updated: core/crates/perf/src/budgets.rs
- Updated: core/crates/perf/Cargo.toml
- Added: scripts/ci/perf_smoke.sh
- Updated: scripts/ci/run_all.sh
- Updated: docs/specs/perf/budgets.json
- Added: tests/golden/inputs/perf/heavy_sample_v1.json
- Added: tests/golden/expected/heavy_sample_v1/warnings.json
- Updated: docs/status/trace-index.json
- Added: docs/verification/S16-PERF-DATASET-SMOKE-006.md

## Preflight Evidence
- git status (pre-change):
```text
On branch work
nothing to commit, working tree clean
```
- git fetch --all --prune: executed
- git checkout -b feature/s16-perf-dataset-smoke-006: executed
- scripts/ci/run_all.sh: executed, baseline rust_fmt failed before this task diff

- git status:
```text
On branch feature/s16-perf-dataset-smoke-006
Changes not staged for commit:
  modified:   core/crates/perf/Cargo.toml
  modified:   core/crates/perf/src/budgets.rs
  modified:   docs/specs/perf/budgets.json
  modified:   docs/status/trace-index.json
  modified:   scripts/ci/run_all.sh
  modified:   tests/datasets/manifest.json
  modified:   tests/perf/perf_smoke.rs
Untracked files:
  core/crates/perf/tests/
  scripts/ci/perf_smoke.sh
  tests/datasets/heavy_sample_v1/
  tests/golden/expected/heavy_sample_v1/
  tests/golden/inputs/perf/
  tests/perf/artifacts/
  tests/perf/mod.rs
```

- git log -n 20 --oneline:
```text
38db3b5 Add deterministic cache crate, render IR pipeline, and Perf SSOT (budgets + lint)
298cb0b Merge pull request #79 from teru1991/codex/expand-golden-datasets-for-step-3-thjhms
609d9fe Merge branch 'main' into codex/expand-golden-datasets-for-step-3-thjhms
70a5816 S15-STEP5: add 10x determinism gate with shared dataset runner
2d283fe Merge pull request #78 from teru1991/codex/expand-golden-datasets-for-step-3-qvdofx
a42e8f1 Merge branch 'main' into codex/expand-golden-datasets-for-step-3-qvdofx
98154eb S15-STEP4: add binary-free N-2 compat assets and harness
8cb360f Merge pull request #77 from teru1991/codex/expand-golden-datasets-for-step-3
e0ffc2c S15-STEP3: expand binary-free golden smoke datasets
6588498 Merge pull request #76 from teru1991/codex/complete-datasets-manifest-as-ssot-fu2rz6
732a663 Merge branch 'main' into codex/complete-datasets-manifest-as-ssot-fu2rz6
129c909 S15-STEP2: include input_sha in golden mismatch repro context
2ee8c19 Merge pull request #75 from teru1991/codex/complete-datasets-manifest-as-ssot-5kutls
768cb95 S15-STEP1: add datasets manifest SSOT schema and lint gate
8bf61bd Merge pull request #74 from teru1991/codex/finalize-sprint14-integration-and-documentation
13f93bb Sprint14 Step8: finalize survival runbook and PR gate evidence
87027ac Merge pull request #73 from teru1991/codex/add-pr-gated-testing-framework
acac9d2 test: add PR-gated golden/compat/fuzz/determinism/e2e (binary-free)
94ae6f9 Merge pull request #72 from teru1991/codex/complete-tools/migrate-for-cli-safety-features
f7c5f4a Sprint14 Step6: complete diycad-migrate CLI with deterministic batch summary
```

- git branch -vv:
```text
* feature/s16-perf-dataset-smoke-006 38db3b5 Add deterministic cache crate, render IR pipeline, and Perf SSOT (budgets + lint)
  work                               38db3b5 Add deterministic cache crate, render IR pipeline, and Perf SSOT (budgets + lint)
```

## Test Evidence
- `cargo test --manifest-path core/Cargo.toml -p craftcad_perf --features perf --test perf_smoke` => PASS
- `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize --test spec_ssot_lint` => PASS
- `scripts/ci/perf_smoke.sh` => PASS

## Repro / Artifact Notes
- Report path: tests/perf/artifacts/perf_report_heavy_sample_v1.json
- Determinism: fixed dataset + fixed span names + no randomness

## Negative Checks
- budgets dataset mismatch:
  - changed `docs/specs/perf/budgets.json` dataset_id to `heavy_sample_v1_mismatch_tmp`
  - `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize --test spec_ssot_lint ssot_perf_budgets_json_is_valid_and_consistent` failed as expected
  - restored dataset_id to `heavy_sample_v1`
- dataset corruption:
  - temporarily wrote invalid JSON into `tests/datasets/heavy_sample_v1/heavy_sample_v1.json`
  - `cargo test --manifest-path core/Cargo.toml -p craftcad_perf --features perf --test perf_smoke` failed as expected
  - restored original dataset JSON

## Safety / Allowlist Self-check
- Only allowed paths modified: YES
- No deletions: YES
