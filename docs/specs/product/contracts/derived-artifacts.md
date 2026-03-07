# Derived Artifacts Contract (2D/3D/Nesting/BOM/Steps) + Mobile Packaging

## Derived artifacts (canonical list)
Generated from SSOT (`Part/Material/FeatureGraph`):

1) **3D View Artifact**
- lightweight render representation (mesh/scene) suitable for desktop + mobile viewing.
- must include per-part identifiers for selection and note linking.
- Step1 minimal model: `part_id` + AABB (`min_x/min_y/min_z`, `max_x/max_y/max_z`) + deterministic `display_color` (RGBA).
- This minimal artifact is sufficient for desktop rendering and click-selection in Step1.

2) **2D Drawing Sheets**
- projection views (front/top/side/iso) + dimensions/notes/title block/print presets
- must maintain stable references to SSOT IDs (see master-model.md)
- Step1 projection-lite uses 3D View Artifact Part AABB boxes to generate per-view 2D rectangle outlines.
- Step1 `ViewSpec`: `{ view: front|top|side, scale: 1.0, units: mm }`.
- Determinism contract: same SSOT snapshot must produce identical serialized sheet JSON hash.

3) **Nesting Sheets**
- for each material/stock choice: placement of outlines + labels + yield
- includes cutlist summary

4) **BOM**
- materials summary
- fasteners summary (from ScrewFeature)
- optional: packaging units (later)
- EstimateLite (Step1):
  - inputs: SSOT (`materials` + `parts` + `manufacturing_outline_2d`)
  - outputs per `material_id`: `{material_name, thickness_mm, parts_count, total_area_mm2, total_area_m2}`
  - deterministic ordering: sorted by `material_id`
  - hash: `sha256(canonical json bytes)`

5) **Manufacturing Hints + Build Steps**
- minimal steps list (cut → drill → chamfer → assemble)
- link each step to relevant parts/features (IDs)

6) **ManufacturingHintsLiteV1 (Step1)**
- inputs: SSOT `feature_graph` (`ScrewFeature` only in Step1)
- item shape (per feature):
  - `{feature_id, part_id, spec_name, pilot_hole_mm, countersink, note_text}`
- deterministic ordering: sorted by `(part_id, feature_id)`
- hash: `sha256(canonical json bytes)`

7) **AnnotationPayloadLite (Step1 contract note)**
- manufacturing hints are emitted as machine-readable payloads that can be mapped into 2D note annotations in later steps.
- Step1 stores payload only (no GUI placement/layout generation in this task).

## Update & invalidation rules
- Invalidation mapping SSOT is defined in `dirty-deps.md`.
- Any SSOT change marks impacted artifacts dirty.
- Immediate update: BOM estimate / simple labels.
- Job update: nesting, drawing regeneration, 3D mesh regeneration.

## Mobile packaging (offline read-only)
Mobile **must not recompute**. Therefore, project save must embed a snapshot of derived artifacts.

### ViewerPackV1 (Step1 minimal)
- `viewer_pack_version = 1`
- manifest fields:
  - `ssot_hash_hex`: hash of canonical SSOT JSON bytes (`sha256`)
  - `artifacts`: list of `{name, schema_version, sha256_hex, bytes_len}`
- required Step1 artifact names:
  - `estimate_lite_v1.json`
  - `projection_lite_front_v1.json` (thumbnail basis)
  - `fastener_bom_lite_v1.json`
  - `mfg_hints_lite_v1.json`
- optional Step1 artifacts: none
- manifest ordering must be deterministic (sorted by `name`).

### Packaging requirements
- Project file contains:
  - SSOT (canonical)
  - Derived snapshot block (optional but recommended)
  - Manifest describing which artifacts exist + versions + hashes

### Minimum “Viewer Pack” for mobile
- 3D view artifact (mesh/scene)
- 2D sheets (vector preferred; raster acceptable as fallback)
- nesting sheets (if available)
- BOM (json-like)
- build steps (json-like)
- thumbnails

### Missing artifact behavior
- If the viewer pack is missing or the required artifact is missing, viewer must show "Not generated" and **never** attempt recomputation.

### Hash mismatch behavior
- If an artifact hash mismatches manifest (`sha256_hex`), viewer must show "Corrupt pack" for that artifact only.
- Other artifacts with valid hashes may still be shown.

## Links
- Mobile SSOT: ../mobile/
- Testing SSOT: ../testing/
- Mobile read-only scope: ../feature-scope.md
