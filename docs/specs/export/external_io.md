# External I/O Export Policy (v1)

## Supported
- SVG export: entities (Line/Polyline) + part outlines/holes.
- Stable element ordering and fixed numeric precision.

## Optional / Deferred
- DXF is deferred in v1 for compatibility-risk reduction.
- Policy: explicitly fail unsupported requested DXF path with reason-coded error.

## Coordinates and Units
- Coordinates are emitted in document coordinates.
- Units are carried as metadata attributes (`data-units`) for consumer-side interpretation.

## Unsupported Handling
- Unsupported entity type -> `EXPORT_UNSUPPORTED_ENTITY`
- Unsupported option/feature -> `EXPORT_UNSUPPORTED_FEATURE`
- Parse/write failures at host boundary -> `EXPORT_IO_PARSE_FAILED` / `EXPORT_IO_WRITE_FAILED`
