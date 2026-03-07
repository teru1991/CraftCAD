# G2-12-DIRTY-ENGINE-001 Verification

## Summary
- Added Dirty Engine contract (`dirty-engine.md`) to fix deterministic input/output behavior for dirty planning.
- Implemented `craftcad_dirty_engine` to compute deterministic `DirtyPlanV1` from `ChangeKind` sets using `craftcad_dirty_deps` SSOT mapping.
- Added unit tests for empty set, duplicate handling, output ordering, reason ordering/uniqueness, and representative screw mapping.
- Added CI step to run dirty engine tests.

## Changed files
- `docs/specs/product/contracts/dirty-engine.md`
- `core/Cargo.toml`
- `core/crates/craftcad_dirty_engine/Cargo.toml`
- `core/crates/craftcad_dirty_engine/src/lib.rs`
- `core/crates/craftcad_dirty_engine/tests/dirty_plan.rs`
- `scripts/ci/run_all.sh`
- `docs/verification/G2-12-DIRTY-ENGINE-001.md`
- `docs/status/trace-index.json`

## History evidence
- `git status --porcelain` (before): clean.
- `git fetch --all --prune`: success.
- `git switch -c feature/g2-12-dirty-engine-001`: success.
- `git rev-parse HEAD`: `bd565323079b4e7bbcc2bdbc5065f7fb0893db35`.
- `git log -n 40 --oneline`: reviewed baseline.
- `git branch -vv`: active branch confirmed.

## Discovery evidence
- `rg -n "craftcad_dirty_deps|DirtyDepsV1|ChangeKind|ArtifactKind" core/crates docs/specs -S`
  - confirmed dirty deps crate and contract already existed and was ready as mapping SSOT for dirty engine.

## Local verification
- `cargo test -p craftcad_dirty_engine` (from `core/`): pass.
- `./scripts/ci/run_all.sh`:
  - `dirty_engine_tests` step passed.
  - existing unrelated failures remained (`rust_test`, `e2e_shelf_flow`).
- `python -m json.tool docs/status/trace-index.json >/dev/null`: pass.

## Self-check
- Allowlist respected (`docs/**`, `core/**`, `scripts/**`, `tests/**`).
- No deletions.
- `trace-index.json` valid.
