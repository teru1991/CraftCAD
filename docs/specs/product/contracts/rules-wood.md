# Woodworking Rules Contract (Safety Checks)

## Severity
- **FATAL**: blocks export/print/nesting confirm unless user explicitly overrides (future config).
- **WARN**: shown prominently; export allowed.

## Rule set (minimum)
### 1) Edge distance (FATAL)
- Trigger: screw point is closer than `min_edge_distance_mm` to any outer boundary of the part.
- Step1 inputs:
  - Part outline: `ManufacturingOutline2dV1` axis-aligned bounding box (`min_x`, `min_y`, `max_x`, `max_y`).
  - Screw points: `(x, y)` from `ScrewFeature` in the same part-local coordinate basis as the bbox.
- Step1 threshold policy:
  - `min_edge_distance_mm = 10.0` (fixed constant for Step1).
- Enforcement: FATAL findings block export, nesting-confirm, and print preflight gates.
- Fix hints: increase `edge_offset_mm`, increase part size, reduce screw count, increase pitch.

### 2) Screw breakthrough (FATAL)
- Trigger: screw length (effective) > material thickness - safety margin.
- Fix hints: shorter screw, thicker material, countersink adjustment.

### 3) Thickness mismatch (FATAL)
- Trigger: feature requires a min thickness (e.g. countersink depth) exceeding part thickness.
- Fix hints: change thickness, change feature parameters.

### 4) Hole spacing (WARN/FATAL by threshold)
- Trigger: hole-to-hole spacing too small for material (risk of split).
- Fix hints: increase pitch, reduce count, change hole diameter.

### 5) Grain policy violation (WARN)
- Trigger: part rotated against grain_policy fixed direction, or nesting rotation breaks grain constraint.
- Fix hints: lock rotation, choose alternate layout, change grain policy.

## Step1 missing-input policy
- If `manufacturing_outline_2d` is missing, emit `RULE_INPUT_MISSING` as **WARN** and skip edge distance evaluation for that point.
- Step1 intentionally avoids globally blocking all jobs on missing outline while surface contracts are still being phased in.

## Links
- Nesting SSOT: ../nesting/
- Part/BOM SSOT: ../part_bom/
