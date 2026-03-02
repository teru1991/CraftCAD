# Integrity & Salvage contract (SSOT)

Goals
- Corruption detection without crashing.
- Best-effort salvage with read-only mode when integrity is uncertain.

content_manifest (optional)
- manifest.content_manifest.entries: array of:
  - path: string (relative, normalized)
  - size: integer (bytes)
  - sha256: string (lowercase hex, 64 chars)

Verification behavior
- If verify_integrity=true AND content_manifest exists:
  - For each entry present in content_manifest:
    - if entry is missing => warning + read_only=true
    - if size mismatch => warning + read_only=true
    - if sha256 mismatch => warning + read_only=true
- If verify_integrity=true AND content_manifest missing:
  - warning only (compat-first)

Salvage behavior (when corruption or schema failures occur)
- Loader MUST:
  - never panic/abort
  - keep parsing other entries best-effort
  - record failures as FailedEntry (path, reason_code, message, kind)
  - set read_only=true when any critical integrity mismatch or partial load happens
- UI guidance MUST be possible via salvage_actions:
  - export_salvaged_parts
  - export_salvaged_document
  - generate_diagnostics_zip (PII forbidden)
  - re-save_as_new_project (after migration/normalize)

Determinism
- Given same zip bytes and same options, the list of warnings/failed entries/salvage_actions is stable.
