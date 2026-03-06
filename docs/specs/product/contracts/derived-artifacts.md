# Derived Artifacts Contract (2D/3D/Nesting/BOM/Steps) + Mobile Packaging

## Derived artifacts (canonical list)
Generated from SSOT (`Part/Material/FeatureGraph`):

1) **3D View Artifact**
- lightweight render representation (mesh/scene) suitable for desktop + mobile viewing.
- must include per-part identifiers for selection and note linking.

2) **2D Drawing Sheets**
- projection views (front/top/side/iso) + dimensions/notes/title block/print presets
- must maintain stable references to SSOT IDs (see master-model.md)

3) **Nesting Sheets**
- for each material/stock choice: placement of outlines + labels + yield
- includes cutlist summary

4) **BOM**
- materials summary
- fasteners summary (from ScrewFeature)
- optional: packaging units (later)

5) **Manufacturing Hints + Build Steps**
- minimal steps list (cut → drill → chamfer → assemble)
- link each step to relevant parts/features (IDs)

## Update & invalidation rules
- Any SSOT change marks impacted artifacts dirty.
- Immediate update: BOM estimate / simple labels.
- Job update: nesting, drawing regeneration, 3D mesh regeneration.

## Mobile packaging (offline read-only)
Mobile **must not recompute**. Therefore, project save must embed a snapshot of derived artifacts.

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
- Viewer pack uses SSOT snapshot; if snapshot is missing, viewer must show "Not generated".
- If an artifact is missing, mobile must show “Not generated” and never attempt generation.

## Links
- Mobile SSOT: ../mobile/
- Testing SSOT: ../testing/
