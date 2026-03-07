# Dirty Dependency Table (SSOT)

## Purpose
`dirty-deps` defines a stable map from SSOT change kind to derived artifacts that must be invalidated.
This table is the SSOT used by dirty-engine / job scheduling / diff presentation.

## Compatibility policy (additive-only)
- Additive-only changes are allowed:
  - add new `ChangeKind`
  - add new `ArtifactKind`
  - add new invalidation targets for a change kind
- Existing meanings must not be changed incompatibly.

## Canonical identifiers
### ChangeKind (string enum)
- `ssot.material.changed`
- `ssot.part.geometry.changed`
- `ssot.part.quantity.changed`
- `ssot.feature.screw.changed`
- `ssot.feature.hole.changed`
- `ssot.feature.pattern.changed`
- `ssot.feature.extrude.changed`
- `ssot.feature.chamfer.changed`

### ArtifactKind (string enum)
- `estimate_lite_v1`
- `projection_lite_v1`
- `fastener_bom_lite_v1`
- `mfg_hints_lite_v1`
- `viewpack_v1`

## Table rules (Step1)
- `ssot.material.changed` -> `estimate_lite_v1`, `viewpack_v1`
- `ssot.part.geometry.changed` -> `estimate_lite_v1`, `projection_lite_v1`, `viewpack_v1`
- `ssot.part.quantity.changed` -> `estimate_lite_v1`, `fastener_bom_lite_v1`, `viewpack_v1`
- `ssot.feature.screw.changed` -> `fastener_bom_lite_v1`, `mfg_hints_lite_v1`, `viewpack_v1`
- `ssot.feature.hole.changed` -> `projection_lite_v1`, `viewpack_v1`
- `ssot.feature.pattern.changed` -> `projection_lite_v1`, `fastener_bom_lite_v1`, `viewpack_v1`
- `ssot.feature.extrude.changed` -> `projection_lite_v1`, `estimate_lite_v1`, `viewpack_v1`
- `ssot.feature.chamfer.changed` -> `projection_lite_v1`, `viewpack_v1`

## Note
Future artifacts (full drawing sheets, nesting, etc.) will be appended in later steps.
