# Feature Scope SSOT (Desktop vs Mobile)

## Desktop (authoring)
### 3D (wood/leather focused 2.5D core)
- FeatureGraph-based modeling:
  - Extrude (add/cut), Hole, Pattern (linear/circular), Chamfer (minimal)
- Material thickness as a first-class attribute (parts are thickness-driven).
- 3D editing **must** trigger 2D + BOM + nesting updates (Pillar 1).

### 2D (drawings)
- Projection from 3D: front/top/side/isometric (sheet builder).
- Dimensions/annotations must not break on typical edits (stable ref IDs).
- Title block + print presets must render consistently (no layout drift).

### Nesting & Cutlist (automatic)
- Immediate estimate: area/length based.
- Confirmed nesting: job-based layout generation → nesting sheet + yield + cutlist.
- Respect grain direction and rotation constraints (where defined).

### Fasteners (screw features) + BOM integration
- ScrewFeature:
  - pattern (line/grid/circle/custom), edge distance, pitch, count
  - pilot hole / countersink metadata (at least carried to drawings/BOM)
- Drawings show screw markers and notes; BOM aggregates counts by spec.

### Safety rules (woodworking-first)
- Edge distance, breakthrough, thickness mismatch, hole spacing, grain violation.
- Block exports on fatal errors (configurable later); always show ReasonCode + fix hints.

### Build steps (minimal but usable)
- Auto-generated steps (cut → drill → chamfer → assemble).
- Exportable as a sheet/page or a checklist view.

### UX primitives (AutoCAD/Fusion-inspired, bounded)
- Command palette + history
- Selection-driven property inspector
- Context toolbar + mark menu (radial)
- HUD numeric input (units + expressions)

## Mobile (read-only viewer)
- Open project file and view **derived artifacts** only:
  - 3D view (orbit/pan/zoom)
  - 2D sheets (paging, zoom)
  - Nesting sheets
  - BOM (materials + fasteners)
  - critical notes (screw counts, manufacturing hints)
- No edit, no recompute, offline-first.

## Out of scope (for now)
- Full mechanical assembly constraints, large assembly interference solver.
- Freeform surfaces, complex solids.
- Mobile editing.

## Compatibility / Guarantees
- Determinism: same input (project+seed+versions) → same artifacts for nesting/projection/BOM/steps.
- Diagnostics: failures always produce ReasonCode; optionally SupportZip.
