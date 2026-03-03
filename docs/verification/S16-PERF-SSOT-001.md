# S16-PERF-SSOT-001 Verification

## Summary
- Added Perf SSOT (budgets + policies + docs) and wired SSOT lint to validate them.

## Changed Files
- Added: docs/specs/perf/budgets.schema.json
- Added: docs/specs/perf/budgets.json
- Added: docs/specs/perf/profiling.md
- Added: docs/specs/perf/job_queue.md
- Added: docs/specs/perf/cache_policy.md
- Added: docs/specs/perf/lod_policy.md
- Added: docs/specs/perf/memory_policy.md
- Updated: core/serialize/tests/spec_ssot_lint.rs
- Updated: docs/status/trace-index.json
- Added: docs/verification/S16-PERF-SSOT-001.md

## Spec Alignment (SSOT)
- 大分類17 性能・安定化 / B17-01 ベンチマークSSOT / B17-02 プロファイル運用 / B17-03 規約化に対応  [oai_citation:3‡DIY向け木工・レザークラフトCAD＋木取り図（ネスティング）搭載ツール中分類.txt](sediment://file_000000004dfc720bbb2ae92cce7eb77a)

## Preflight Evidence
- git status (before changes):
```text
On branch work
nothing to commit, working tree clean
```
- git fetch --all --prune: executed
- git checkout -b feature/s16-perf-ssot-001: executed
- scripts/ci/run_all.sh (if possible): executed, rust_fmt failed in current baseline before this task diff

- git status (current):
```text
On branch feature/s16-perf-ssot-001
Changes not staged for commit:
  modified:   core/serialize/tests/spec_ssot_lint.rs
  modified:   docs/specs/perf/budgets.json
  modified:   docs/specs/perf/budgets.schema.json
  modified:   docs/specs/perf/profiling.md
Untracked files:
  docs/specs/perf/cache_policy.md
  docs/specs/perf/job_queue.md
  docs/specs/perf/lod_policy.md
  docs/specs/perf/memory_policy.md
```

- git log -n 20 --oneline:
```text
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
50f3cbb Merge pull request #71 from teru1991/codex/implement-recovery-crate-with-autosave
```

- git branch -vv:
```text
* feature/s16-perf-ssot-001 298cb0b Merge pull request #79 from teru1991/codex/expand-golden-datasets-for-step-3-thjhms
  work                      298cb0b Merge pull request #79 from teru1991/codex/expand-golden-datasets-for-step-3-thjhms
```

## Test Evidence
- `cargo test -p craftcad_serialize --test spec_ssot_lint` => PASS (10 passed)
- Negative checks:
  - dataset_id mismatch fails lint (confirmed, reverted)
    - changed budgets dataset_id to `nonexistent_dataset_for_negative_check`
    - `cargo test -p craftcad_serialize --test spec_ssot_lint ssot_perf_budgets_json_is_valid_and_consistent` failed with:
      - `budgets.json dataset_id not found in tests/datasets/manifest.json`
  - schema invalid fails lint (confirmed, reverted)
    - corrupted `budgets.schema.json`
    - same test failed with:
      - `budgets.schema.json must be valid JSON`

## Safety / Allowlist Self-check
- Only allowed paths modified: YES
- No deletions: YES
- Determinism/limits not impacted: YES (SSOT only)
