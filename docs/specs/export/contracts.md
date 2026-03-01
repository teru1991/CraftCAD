# Export Contracts (v1)

## Tiled PDF
- includes 100mm scale gauge (option controlled)
- crop marks optional
- deterministic page order
- metadata fields include app/doc/unit context

## SVG/DXF policy
- SVG: deterministic ordering and fixed precision
- unsupported entities/features map to Reason codes (`EXPORT_UNSUPPORTED_ENTITY`, `EXPORT_UNSUPPORTED_FEATURE`)
- coordinate system and precision are stable by option defaults.
