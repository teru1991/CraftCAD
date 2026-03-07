# G2-11-DIRTY-DEPENDENCY-TABLE-001 Verification

## Summary
- Added `dirty-deps` SSOT contract to define deterministic invalidation mapping from SSOT change kinds to lite derived artifacts.
- Introduced `craftcad_dirty_deps` crate (v1 table + canonicalization + lookup API).
- Added crate tests and wired them into CI unified entrypoint.
- Compatibility policy is additive-only (extend by adding kinds/targets without redefining existing meaning).

## Changed files
- `docs/specs/product/contracts/dirty-deps.md`
- `docs/specs/product/contracts/derived-artifacts.md`
- `core/Cargo.toml`
- `core/crates/craftcad_dirty_deps/Cargo.toml`
- `core/crates/craftcad_dirty_deps/src/lib.rs`
- `core/crates/craftcad_dirty_deps/tests/default_table.rs`
- `scripts/ci/run_all.sh`
- `docs/verification/G2-11-DIRTY-DEPENDENCY-TABLE-001.md`
- `docs/status/trace-index.json`

## History evidence
- `git status --porcelain` (before): clean.
- `git fetch --all --prune`: success.
- `git switch -c feature/g2-11-dirty-dependency-table-001`: success.
- `git rev-parse HEAD`: `f0bc6db7afe9256f956d49fa6ed8907f29ee69a5`.
- `git log -n 40 --oneline`: reviewed latest baseline history.
- `git branch -vv`: active branch confirmed.

## Discovery evidence
- `rg -n "dirty|invalidate|dependency|regen|recompute" docs/specs core/crates -S`
  - confirmed existing invalidation notes existed but no explicit SSOT dependency table contract.
- `rg -n "projection_lite|estimate_lite|fastener_bom_lite|mfg_hints_lite|viewpack" docs/specs core/crates -S`
  - confirmed current lite artifact set and naming used by this Step1 table.

## Local verification
- `cargo test -p craftcad_dirty_deps` (from `core/`): pass.
- `./scripts/ci/run_all.sh`:
  - `dirty_deps_tests` step passed.
  - existing unrelated failures remained (`rust_test`, `e2e_shelf_flow`).
- `python -m json.tool docs/status/trace-index.json >/dev/null`: pass.

## Self-check
- Allowlist respected (`docs/**`, `core/**`, `scripts/**`, `tests/**`).
- No deletions.
- `trace-index.json` valid.
