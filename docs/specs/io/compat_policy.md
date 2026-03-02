# IO Compatibility Policy (Sprint12 completion gate)

This document defines the acceptance criteria for IO round-trip compatibility and how to report deviations.

## Scope
Formats:
- DXF / SVG / JSON (internal model v1)

Artifacts:
- Deterministic import/export results
- Compatibility report (machine-readable JSON)
- ReasonCode warnings are preserved and actionable

## Determinism Contract
Given identical:
- input bytes
- ImportOptions / ExportOptions (seed/eps/rounding)
The outputs MUST be identical:
- normalized InternalModel JSON (string-equal after stable serialization)
- exported DXF/SVG/JSON bytes (string-equal)

## Round-trip Contract (per format)
For any input model (JSON v1) and target format F:
- export JSON -> F -> import F must produce a model that is:
  - **geometry-equivalent** within `close_eps` (after shared normalization)
  - **topology-stable** for supported primitives (line/arc/circle; cubic may be approximated)
  - **style-compatible** per MappingRules canonicalization (layer/linetype normalization)

## Allowed differences
Allowed (must be reported in compat_report):
- Curve approximation: CubicBezier -> polyline or format-required conversion
  - ReasonCode: IO_CURVE_APPROX_APPLIED
- Text best-effort:
  - font/layout differences are not validated; presence/position/rotation/size are validated
  - ReasonCode: IO_TEXT_FALLBACK_FONT / IO_FALLBACK_024
- External references dropped (SVG):
  - ReasonCode: IO_IMAGE_REFERENCE_DROPPED

Not allowed (test failure):
- Non-deterministic output (same input -> different output)
- Entity loss without corresponding ReasonCode listed by support_matrix.json for that feature
- Geometry deviation exceeding `close_eps` after normalization
- SSOT inconsistency:
  - support_matrix references unknown reason_codes (not in errors/catalog.json)
  - mapping_rules schema invariants broken (schema_version mismatch, invalid regex, etc.)

## Compatibility Report schema (v1)
A run MUST produce:
- `schema_version`: 1
- `input_id`: string
- `pipelines`: list of pipelines tested (json->dxf->json, json->svg->json, etc)
- `results`: per pipeline:
  - `deterministic`: bool
  - `geometry_ok`: bool
  - `style_ok`: bool
  - `warnings`: list of reason codes observed
  - `notes`: optional strings

The report is stored as golden to detect regressions.
