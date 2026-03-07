# Artifact Store Contract (artifact_store_v1)

## Purpose
`artifact_store_v1` is an in-project cache for derived artifacts so downstream consumers (desktop/viewer/mobile) can read stable outputs without mandatory recompute.

## Step1 schema
- `artifact_store_v1` is an optional project field.
- `schema_version` is fixed to `1`.
- Entries are keyed by `ArtifactKind` and each entry has:
  - `kind`
  - `schema_version`
  - `sha256_hex`
  - `generated_at` (optional in future)
  - `bytes` (canonical JSON bytes; encoded by project serialization when needed)

## Rules
- Determinism: `sha256_hex` is computed from canonical bytes.
- Canonical ordering: entries are sorted by `ArtifactKind`.
- Duplicate entries are normalized by deterministic rule (last-wins before sorting).
- Compatibility is additive-only (new entry kinds may be added; existing semantics must not change).

## Relationship to viewpack
`viewpack_v1` can be built directly from SSOT or from artifact-store-backed data. Step1 keeps them aligned while preserving backward compatibility.
