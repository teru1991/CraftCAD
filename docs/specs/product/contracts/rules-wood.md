# Woodworking Rules Contract (Safety Checks)

## Severity
- **FATAL**: blocks export/print/nesting confirm unless user explicitly overrides (future config).
- **WARN**: shown prominently; export allowed.

## Rule set (minimum)
### 1) Edge distance (FATAL)
- Trigger: hole/screw/slot is closer than `min_edge_distance_mm` to any outer boundary of the part.
- Inputs: part outline, feature position, material/thickness.
- Fix hints: increase edge distance, reduce count, increase part size, change pattern.

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

## Links
- Nesting SSOT: ../nesting/
- Part/BOM SSOT: ../part_bom/
