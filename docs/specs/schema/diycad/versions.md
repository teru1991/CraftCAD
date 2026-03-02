# .diycad schema versions (SSOT)

Rules
- schema_version is an integer in manifest.json.
- Read support: latest, latest-1, latest-2 (N-2).
- Write support: latest only.
- Migration: stepwise only (vN -> vN+1), never skip.

Latest
- latest_schema_version: 1
- Note: 初版。将来増える。互換ポリシーは migration_policy.md に従う。

Version history
- v1 (latest)
  - Initial schema set: manifest/document/part/nest_job
  - content_manifest optional in manifest
  - determinism_tag optional in manifest
