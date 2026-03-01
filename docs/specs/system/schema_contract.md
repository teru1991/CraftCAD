# Schema Contract (v1.0)

Schema files:
- `manifest.schema.json` (`$id`: `https://example.local/schemas/manifest.schema.json`)
- `document.schema.json` (`$id`: `https://example.local/schemas/document.schema.json`)

Compatibility rules:
- v1 loader accepts older documents via normalization (missing add-only fields are injected with defaults).
- Existing field names and semantics are frozen for v1.x.

Schema bump rules:
- bump `schema_version` only for breaking changes that cannot be represented as add-only normalization.
