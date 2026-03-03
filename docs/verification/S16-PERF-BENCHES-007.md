# S16-PERF-BENCHES-007 Verification

## Summary
- Added bench executables that emit PerfReport JSON artifacts for manual performance comparison (non-CI).

## Changed Files
- Added: benches/perf_common.rs
- Added: benches/io_import_export.rs
- Added: benches/diycad_open_save.rs
- Added: benches/render_frame.rs
- Added: scripts/bench/run_perf_benches.sh
- Updated: core/crates/perf/Cargo.toml (bench targets with harness=false)
- Updated: docs/status/trace-index.json
- Added: docs/verification/S16-PERF-BENCHES-007.md

## Preflight Evidence
- git status (before branch had leftover artifact dir):
```text
On branch feature/s16-perf-dataset-smoke-006
Untracked files:
  tests/perf/artifacts/
```
- git fetch --all --prune: executed
- git checkout -b feature/s16-perf-benches-007: executed
- scripts/ci/run_all.sh: executed; baseline rust_fmt failed, others passed including perf_smoke

- git status:
```text
On branch feature/s16-perf-benches-007
Changes not staged for commit:
  modified:   benches/diycad_open_save.rs
  modified:   benches/io_import_export.rs
  modified:   core/crates/perf/Cargo.toml
Untracked files:
  benches/perf_common.rs
  benches/render_frame.rs
  scripts/bench/
```

- git log -n 20 --oneline:
```text
1dd3a5a S16: add heavy dataset perf smoke gate with report artifacts
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
```

- git branch -vv:
```text
* feature/s16-perf-benches-007       1dd3a5a S16: add heavy dataset perf smoke gate with report artifacts
  feature/s16-perf-dataset-smoke-006 1dd3a5a S16: add heavy dataset perf smoke gate with report artifacts
  work                               38db3b5 Add deterministic cache crate, render IR pipeline, and Perf SSOT (budgets + lint)
```

## Run Evidence
- `cargo bench --manifest-path core/Cargo.toml -p craftcad_perf --features perf --bench io_import_export` => PASS
- `scripts/bench/run_perf_benches.sh` => PASS
- `ls benches/artifacts` produced:
  - bench_io_import_export_heavy_sample_v1.json
  - bench_diycad_open_save_heavy_sample_v1.json
  - bench_render_frame_heavy_sample_v1.json

## Artifact Notes
- Output dir: benches/artifacts/
- Files:
  - bench_io_import_export_heavy_sample_v1.json
  - bench_diycad_open_save_heavy_sample_v1.json
  - bench_render_frame_heavy_sample_v1.json
- Format has fixed `dataset_id` and `determinism_tag`, plus `report.spans` for direct comparison.

## Safety / Allowlist Self-check
- Only allowed paths modified: YES
- No deletions: YES
