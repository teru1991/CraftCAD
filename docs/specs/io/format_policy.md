# IO Format Policy (SSOT supplementary)

This document defines *pipeline order* and *determinism constraints* for all import/export formats.

Machine-readable SSOT:
- `docs/specs/io/support_matrix.json`
- `docs/specs/io/mapping_rules.json`

Related acceptance gate:
- `docs/specs/io/compat_policy.md`

## Terminology
- **Importer**: parses bytes into `InternalModel` + `warnings` + `IoReport`
- **Exporter**: converts `InternalModel` into bytes + `warnings` + `IoReport`
- **Pipeline**: shared post-parse steps that must be identical across formats

## Determinism Contract
All IO operations MUST be deterministic under the same:
- seed / rounding step / eps
- segment ordering rules
- stable iteration order (no HashMap order dependence)
- fixed decimal formatting for textual formats (DXF/SVG)

## Shared Import Pipeline (required order)
Importers MUST:
1. `limits` gate (bytes/nodes/depth/groups)
2. parse raw format (DOM/groups/etc)
3. apply `MappingRules` (layer/linetype/unit defaults)
4. **Normalize** geometry (rounding, non-finite sanitize, canonical orientation)
5. **Approx** curves if required by importer or `support_matrix` best-effort actions
6. **Postprocess** path optimization (join/dedup/order) deterministically
7. return `InternalModel`, `warnings`, `IoReport`

## Shared Export Pipeline (required order)
Exporters MUST:
1. receive a normalized model (or normalize internally)
2. apply unit scaling to target_units
3. apply `MappingRules` (layer/linetype canonicalization)
4. if `support_matrix` says best-effort, record warnings using listed `reason_codes`
5. **Approx** curves when target format cannot represent segment types
6. **Postprocess** path optimization if it reduces output without changing geometry within eps
7. format serialization with fixed decimal places (from `mapping_rules.export.decimal_places`)

## Path Optimization Contract
- dedup: remove duplicate segments (exact or within eps)
- join: merge contiguous collinear lines within eps
- order: stable sort by deterministic tie-breakers
- MUST NOT change shape beyond `determinism.close_eps`

## Curve Approx Policy
- `approx_epsilon` controls max deviation
- `approx_min_segments` / `approx_max_segments` clamps segment counts
- Arc/Circle: if target lacks primitives, convert to polyline using deterministic sampling
- CubicBezier: convert to polyline using deterministic sampling
