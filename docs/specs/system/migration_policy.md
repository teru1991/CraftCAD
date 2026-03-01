# Migration Policy

## Versioning intent
- `.diycad` remains source of truth.
- `document.schema_version` is currently `1`.

## v1 policy
- Add-only, backward-compatible fields only.
- New fields must have safe defaults via loader normalization.
- No breaking removals/renames in v1.

## Future v1 -> v2
- Introduce explicit migrators in loader path.
- Old docs are normalized then transformed deterministically.
- Unsupported major versions return `SERIALIZE_UNSUPPORTED_SCHEMA_VERSION`.
