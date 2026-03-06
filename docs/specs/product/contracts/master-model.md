# Master Model Contract (SSOT)

## Overview
The SSOT is:
- `Material` (attributes and constraints)
- `Part` (manufacturing unit: quantity + material link + identifiers)
- `FeatureGraph` (history-based operations producing geometry and references)

Derived artifacts are generated from SSOT; see `derived-artifacts.md`.

## Stable identity rules (hard contract)
- Every Part and Feature has a stable ID (`Uuid` or equivalent) persisted in the project file.
- References from 2D dimensions/notes or BOM lines must point to stable IDs, not ephemeral topology indices.

## Minimal data definitions (language-agnostic)
### Material
Fields (minimum):
- `material_id`
- `category`: wood | plywood | mdf | leather | hardware (hardware is for BOM grouping)
- `name`
- `thickness_mm` (required for wood/leather parts)
- `grain_policy`: none | along_x | along_y | fixed (if fixed, provide reference axis)
- `kerf_mm` (default for nesting)
- `margin_mm` (default for nesting)
- `estimate_loss_factor` (optional; leather recommended > 1.0)

### Part
Fields (minimum):
- `part_id`
- `name`
- `material_id`
- `quantity`
- `manufacturing_bbox_2d` OR `manufacturing_outline_2d` (for nesting/cutlist)
- `thickness_mm` (derived from material, but stored for snapshot stability)
- `grain_direction` (optional override)
- `labels` (part mark / drawing label)
- `features_ref`: references to FeatureGraph nodes that apply to the part (holes/screws etc)

### FeatureGraph
- A deterministic ordered list/tree of features (history).
- Each feature:
  - `feature_id`
  - `type` (ExtrudeAdd/ExtrudeCut/Hole/Pattern/Chamfer/ScrewFeature/…)
  - `params` (typed + versioned)
  - `targets` (stable refs to parts or sketches)
- Feature evaluation must be deterministic:
  - stable sorting for generated instances
  - stable floating rounding/eps policy (see ../determinism/)
  - seed captured in project for randomized heuristics (nesting)

## “2D drives 3D” scope limitation (initial)
- 2D dimensions can modify parameters only if explicitly mapped:
  - `DimensionRef → ParameterRef` mapping must exist, otherwise 2D edit is display-only.
- This avoids ambiguous reverse-engineering of geometry from drawings.

## Links
- Determinism SSOT: ../determinism/
- Project file SSOT: ../project_file/
- Part/BOM SSOT: ../part_bom/


## Persistence & Backward Compatibility (Step1)
- Project file may include optional `ssot_v1` snapshot block. Missing `ssot_v1` is valid for backward compatibility.
- Loader fallback when `ssot_v1` is missing derives a minimal snapshot:
  - one Material named `unspecified` (`thickness_mm` may be null)
  - Parts derived from existing project parts/entities when available; otherwise one root Part
  - `FeatureGraph` is empty
- Stable identity contract:
  - `part_id` / `material_id` / `feature_id` are UUIDs persisted in the project file
  - fallback derivation generates UUIDs deterministically from stable inputs and preserves deterministic ordering
- Versioning contract:
  - `ssot_version = 1`
  - future versions are additive-only; removals are forbidden (deprecated fields must remain)

For overall project file layout, see `../project_file/`.
