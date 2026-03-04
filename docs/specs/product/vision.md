# CraftCAD Product Vision (Final Goal SSOT)

## Mission
CraftCAD is a woodworking + leathercraft focused CAD that does **not** stop at geometry creation.
It generates and keeps consistent: **3D model, 2D drawings, nesting/cutlist, material/fastener BOM, manufacturing hints, and build steps**,
and keeps them in sync under deterministic rules.

## Target users
- DIY beginners to intermediate makers (shelves, boxes, drawers, small furniture, leather goods)
- Small workshops / creators who need repeatable outputs (cutlists, jigs, stable drawings)

## Device split (hard scope)
- **Desktop**: authoring environment (high-performance) – create/edit, run heavy jobs, generate outputs.
- **Mobile**: **read-only viewer** – open project and inspect derived artifacts offline (no editing, no re-computation).

## The 6 differentiation pillars (functional)
### Pillar 1 — SSOT + Change-resilient sync
- The SSOT is `Part + Material + FeatureGraph`.
- 2D/3D/nesting/BOM/instructions are **Derived Artifacts** generated from SSOT and updated on change.
- References (dimensions/annotations/holes/fasteners) are maintained using stable IDs; failures surface as ReasonCodes.

### Pillar 2 — “Buy & Build” outputs
- Drawings are production-grade (projection views, dimensions, title blocks, print presets).
- Nesting produces cut layouts + cutlist + yield.
- BOM includes materials + fasteners; drawings can show part labels, grain direction, and fastener notes.

### Pillar 3 — Woodworking safety rules (fail-fast)
- Detect common fatal mistakes (edge distance, screw breakthrough, thickness mismatch, grain constraint violations).
- Provide actionable fix hints (ReasonCode → UI hints).

### Pillar 4 — Build guarantee (manufacturing hints + build steps)
- Generate minimal build steps and manufacturing hints from features/parts (cut → drill → chamfer → assemble).
- Steps are exportable (optional) and link to parts/drawings.

### Pillar 5 — On-site usability (mobile read-only)
- Mobile can open the project offline and view:
  3D view, 2D sheets, nesting sheets, BOM, critical notes (fasteners, hole patterns).

### Pillar 6 — Product-grade reliability
- Deterministic outputs (same input → same output) for: nesting, drawing projection, BOM, steps.
- Heavy work runs as Jobs (progress/cancel/result link).
- SupportZip/diagnostics reproduce failures safely (redaction/consent/limits).

## Non-goals (near/mid term)
- General-purpose CAD parity with AutoCAD/Fusion (freeform surfacing, full mechanical assembly).
- Mobile editing or computation.
- Large mechanical assemblies; woodworking-first checks are prioritized.

## References (existing SSOT)
- Determinism: ../determinism/
- Jobs/UX: ../ux/ , ../system/ (see job UX + error UX)
- Drawing: ../drawing/ and ../drawing_style/
- Nesting: ../nesting/
- Part/BOM: ../part_bom/
- Diagnostics + SupportZip: ../diagnostics/
- Security/consent/redaction/limits: ../security/
