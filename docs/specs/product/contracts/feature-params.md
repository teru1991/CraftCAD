# Feature Params Schema Contract

## Terminology
- `schema_id`: stable schema identifier string (example: `screw_feature.v1`).
- `v`: params version number inside `FeatureNodeV1.params` JSON.

## Contract
- For known feature types, every `FeatureNodeV1.params` MUST include `{ "v": <u32> }`.
- `schema_id` is uniquely derived from `(feature_type, v)` by the params schema registry.
- New versions are additive-only:
  - new versions can be added,
  - previously released versions must remain readable,
  - destructive semantic rewrites are forbidden.
- Write path emits the latest supported version for each feature type unless explicitly pinned.

## Validation
- On load, registry validation checks:
  - required keys exist (minimum: `v`),
  - `v` resolves to a registered schema for the feature type,
  - unknown extra keys are allowed (forward compatibility).
- Any mismatch must return ReasonCode: `FEATURE_PARAMS_SCHEMA_MISMATCH`.

## Enforcement
- Canonical registry implementation: `craftcad_params_registry`.
- Registry is static and deterministic (compiled-in ordered list).
