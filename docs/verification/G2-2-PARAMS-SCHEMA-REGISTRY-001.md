# G2-2-PARAMS-SCHEMA-REGISTRY-001 Verification

## Summary
- Added a params schema registry contract for FeatureGraph params and fixed additive-only compatibility rules.
- Implemented `craftcad_params_registry` as a deterministic compiled-in registry mapping `(feature_type, v)` to `schema_id`.
- Added registry validation API returning `FEATURE_PARAMS_SCHEMA_MISMATCH` for missing/unsupported versions.
- Added regression coverage to keep ScrewFeature fixtures on `v:1` in tests.
- Wired CI to run `craftcad_params_registry` tests.

## Changed files
- `docs/specs/product/contracts/feature-params.md`
- `docs/specs/product/contracts/master-model.md`
- `core/Cargo.toml`
- `core/crates/craftcad_params_registry/Cargo.toml`
- `core/crates/craftcad_params_registry/src/lib.rs`
- `core/crates/craftcad_params_registry/tests/registry.rs`
- `core/crates/craftcad_rules_engine/tests/edge_distance.rs`
- `core/crates/craftcad_screw_lite/tests/params_version.rs`
- `scripts/ci/run_all.sh`
- `docs/verification/G2-2-PARAMS-SCHEMA-REGISTRY-001.md`
- `docs/status/trace-index.json`

## History evidence
- `git status --porcelain`
- `git fetch --all --prune`
- `git switch -c feature/g2-2-params-schema-registry-001`
- `git rev-parse HEAD`
- `git log -n 40 --oneline`
- `git branch -vv`

## Current params usage findings
- `rg -n "FeatureNodeV1|FeatureTypeV1|params: serde_json::Value|serde_json::Value" core/crates -S`
  - confirms `FeatureNodeV1.params` is still `serde_json::Value` in SSOT and widely consumed.
- `rg -n "\"v\"\s*:\s*\d+|schema_version|params_version" core/crates docs/specs -S`
  - confirms existing `v` patterns in screw/mfg tests and other schema_version patterns.
- `rg -n "FeatureTypeV1::ScrewFeature[\s\S]{0,200}params|feature_type:\s*FeatureTypeV1::ScrewFeature" core/crates -S`
  - identified ScrewFeature fixtures and updated missing `v` fixture in rules-engine tests.

## Local verification
- `cargo test -p craftcad_params_registry --manifest-path core/Cargo.toml`
- `cargo test -p craftcad_screw_lite --manifest-path core/Cargo.toml`
- `./scripts/ci/run_all.sh`
- `python -m json.tool docs/status/trace-index.json >/dev/null`

## Self-check
- Allowlist respected (`docs/**`, `core/**`, `tests/**`, `scripts/**`).
- No deletions.
- `trace-index.json` valid.
