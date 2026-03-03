# S16-CACHE-LRU-004 Verification

## Summary
- Added deterministic cache core (canonical JSON hashing + sha256 key) and bounded epoch-based LRU store with safe eviction.

## Changed Files
- Added: core/crates/cache/Cargo.toml
- Added: core/crates/cache/src/lib.rs
- Added: core/crates/cache/src/reasons.rs
- Added: core/crates/cache/src/key.rs
- Added: core/crates/cache/src/policy.rs
- Added: core/crates/cache/src/store.rs
- Added: core/crates/cache/tests/cache_lru_determinism_invalidation.rs
- Updated: core/Cargo.toml (workspace members)
- Added: docs/verification/S16-CACHE-LRU-004.md

## Preflight Evidence
- git status (pre-change):
```text
On branch work
nothing to commit, working tree clean
```
- git fetch --all --prune: executed
- git checkout -b feature/s16-cache-lru-004: executed
- scripts/ci/run_all.sh: executed, baseline rust_fmt failed before this task diff

- git status:
```text
On branch feature/s16-cache-lru-004
Changes not staged for commit:
  modified:   core/Cargo.toml
Untracked files:
  core/crates/cache/
```

- git log -n 20 --oneline:
```text
ee195c0 S16: Add Perf SSOT budgets schema and CI lint checks
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
* feature/s16-cache-lru-004 ee195c0 S16: Add Perf SSOT budgets schema and CI lint checks
  work                      ee195c0 S16: Add Perf SSOT budgets schema and CI lint checks
```

## Test Evidence
- `cargo test -p craftcad-cache` => PASS (5 tests)

## Contract Guarantees
- CacheKey: deterministic (canonical JSON) + changes when ssot/inputs/options/schema change
- LRU: bounded by total bytes and entry count; eviction is silent by default (warnings are drainable)
- No unsafe, thread-safe Mutex

## Safety / Allowlist Self-check
- Only allowed paths modified: YES
- No deletions: YES
