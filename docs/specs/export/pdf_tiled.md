# Tiled PDF Export (v1)

## Scope
1:1 tiled print export for fabrication layout checks.

## Page Options
- `page_size`: `A4` or `Letter`
- `orientation`: `Portrait` or `Landscape`
- `margin_mm`: printable margin in millimeters
- Optional crop marks and 100mm scale gauge

## Tiling Rules
- Geometry is exported at **1:1** in document units (`mm`/`inch`) with deterministic unit conversion.
- Page tile ordering is row-major (`R1C1`, `R1C2`, ...).
- Content ordering is stable by entity/part deterministic order.

## Crop Marks
- If enabled, corner crop marks are rendered on every tile.

## Scale Gauge
- If enabled, draw one `100mm` gauge line and label `Gauge 100mm` on every page.
- In `inch` documents, internal doc-space gauge length is `100 / 25.4`.

## Embedded Metadata
When `include_metadata = true`, include:
- app producer (`CraftCAD`)
- title
- doc id
- units
- nesting job metadata (v1: first job seed if present)
