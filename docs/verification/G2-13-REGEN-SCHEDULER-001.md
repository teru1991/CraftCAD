# G2-13-REGEN-SCHEDULER-001 Verification

## Summary
- Added Artifact Store contract (`artifact-store.md`) and updated Jobs contract to define REGEN job behavior with DirtyPlan input and all-or-nothing writes.
- Implemented `craftcad_artifact_store` with deterministic canonicalization (last-wins + sorted by ArtifactKind).
- Added `craftcad_regen_scheduler` to execute DirtyPlan-driven regeneration in Sync/Job modes for lite artifacts and return deterministic store updates.
- Integrated optional `artifact_store_v1` into project save/load (`artifact_store_v1.json`) with backward-compatible optional loading.
- Wired CI to run artifact-store and regen-scheduler tests.

## Changed files
- `docs/specs/product/contracts/artifact-store.md`
- `docs/specs/product/contracts/jobs.md`
- `core/Cargo.toml`
- `core/crates/craftcad_artifact_store/Cargo.toml`
- `core/crates/craftcad_artifact_store/src/lib.rs`
- `core/crates/craftcad_artifact_store/tests/store.rs`
- `core/crates/craftcad_regen_scheduler/Cargo.toml`
- `core/crates/craftcad_regen_scheduler/src/lib.rs`
- `core/crates/craftcad_regen_scheduler/tests/regen.rs`
- `core/crates/diycad_project/Cargo.toml`
- `core/crates/diycad_project/src/lib.rs`
- `core/crates/diycad_ffi/src/lib.rs`
- `scripts/ci/run_all.sh`
- `docs/verification/G2-13-REGEN-SCHEDULER-001.md`
- `docs/status/trace-index.json`

## History evidence
- `git status --porcelain` (before): dirty docs from prior attempt.
- `git fetch --all --prune`: success.
- `git switch -c feature/g2-13-regen-scheduler-001`: branch active.
- `git rev-parse HEAD`: baseline captured.
- `git log -n 40 --oneline`: reviewed recent changes.
- `git branch -vv`: branch tracking confirmed.

## Discovery evidence
- `rg -n "artifact|cache|derived|viewer_pack|estimate_lite|projection_lite" core/crates -S`
- `rg -n "viewer_pack_v1|ssot_v1" core/crates -S`
  - Confirmed project had `ssot_v1` / `viewer_pack_v1` but no `artifact_store_v1` field at baseline.

## Local verification
- `cargo test -p craftcad_artifact_store --manifest-path core/Cargo.toml`
- `cargo test -p craftcad_regen_scheduler --manifest-path core/Cargo.toml`
- `cargo test -p diycad_project --manifest-path core/Cargo.toml`
- `python -m json.tool docs/status/trace-index.json >/dev/null`
- `./scripts/ci/run_all.sh`

## Self-check
- Allowlist respected (`docs/**`, `core/**`, `tests/**`, `scripts/**`).
- No deletions.
- `trace-index.json` is valid JSON.
