# S15-STEP1 Verification

## History evidence
### Preflight commands and key outputs
- `git status --porcelain`
  - (no output; working tree clean before changes)
- `git fetch --all --prune`
  - completed without error
- `git checkout -b feature/s15-step1-001`
  - `Switched to a new branch 'feature/s15-step1-001'`
- `git log --oneline -n 20`
  - `8bf61bd Merge pull request #74 from teru1991/codex/finalize-sprint14-integration-and-documentation`
  - `13f93bb Sprint14 Step8: finalize survival runbook and PR gate evidence`
  - `87027ac Merge pull request #73 from teru1991/codex/add-pr-gated-testing-framework`
- `git log --graph --decorate --oneline --all -n 40`
  - HEAD branch at start: `feature/s15-step1-001`
  - graph captured merge ancestry through Sprint13/Sprint14 milestones
- `git branch -vv`
  - `* feature/s15-step1-001 8bf61bd Merge pull request #74 ...`
  - `work                  8bf61bd Merge pull request #74 ...`
- `git rev-parse HEAD`
  - `8bf61bd9ea96f4591f5769e9d2c926d2179f19dc`

## Scope / SSOT alignment
- Step1 establishes `tests/datasets/manifest.json` as SSOT and adds machine-readable schema docs.
- Rust-side validation/lint is implemented without runtime jsonschema dependencies (serde/serde_json only).
- Validation guarantees:
  - structure + required fields
  - path safety + allowlisted prefixes
  - determinism numeric ranges
  - compare enum contract
  - reproducibility metadata presence (`dataset_id`, `seed`, `determinism`, `limits_ref` via manifest contract)

## Changed files (allowlist)
- `docs/specs/testing/datasets_manifest.schema.json`
- `tests/datasets/manifest.json`
- `tests/datasets/README.md`
- `tests/golden/inputs/io/svg/min_rect.svg`
- `tests/golden/inputs/io/json/min_rect.json`
- `tests/golden/inputs/projects/min_project.diycad`
- `tests/golden/expected/io_roundtrip_smoke/normalized_model.json`
- `tests/golden/expected/io_roundtrip_smoke/warnings.json`
- `tests/golden/expected/diycad_open_save_smoke/open_result.json`
- `tests/golden/expected/diycad_open_save_smoke/warnings.json`
- `core/src/testing/mod.rs`
- `core/src/testing/datasets_manifest.rs`
- `core/tests/ssot_datasets_manifest_lint.rs`
- `core/crates/ssot_lint/Cargo.toml`
- `core/crates/ssot_lint/tests/ssot_datasets_manifest_lint.rs`
- `docs/status/trace-index.json`

## How verified
- `cargo test -p ssot_lint` ✅
- `./scripts/ci/run_all.sh` (first run) ❌ failed at `rust_fmt`; root cause confirmed in `.ci_logs/summary.json`
- `cargo fmt --all` ✅
- `./scripts/ci/run_all.sh` (rerun) ✅ all gates passed
- `.ci_logs/summary.json` shows `"total_failures": 0` ✅

## Notes on determinism / compatibility alignment
- Determinism guard rails are now explicit in manifest validation: `epsilon` and `round_step` must be finite and in `(0, 1e-2]`; `ordering_tag` non-empty.
- Paths are strictly relative and traversal-safe (`..`, abs path, backslash segments rejected), limiting fixture access to `tests/golden/inputs/` and `tests/golden/expected/`.
- Placeholder fixtures were added only to satisfy Step1 lint/SSOT path existence and are intentionally minimal for later golden population.
