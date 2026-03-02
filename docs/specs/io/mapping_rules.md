# Mapping Rules

This document describes the mapping rule source of truth for IO format conversion.

See machine-readable specification:
- `docs/specs/io/mapping_rules.json`

## Semantics (SSOT補足)
### Layer mapping
- Input layer name is normalized using:
  - trim / whitespace collapse / replace spaces (see `layer.normalize`)
  - forbidden chars replacement (see `layer.forbidden_chars_regex`)
  - max length clamp (see `layer.max_len`)
- After normalization, if it matches `layer.aliases` key (case-insensitive by normalized uppercase), it is mapped to the alias value.
- If the normalized name becomes empty, fallback to `layer.default`.
- Otherwise (unknown but valid), preserve the normalized layer name (do NOT collapse all unknown names to default).

### Linetype mapping
- Input linetype name is normalized similarly (`linetype.*`).
- If it matches `linetype.aliases`, map to the alias value.
- If unknown or empty, fallback to `linetype.default` (DXF/SVG interoperability prefers canonical linetype).

### Units rules
- `units.default` must be one of `units.supported`.
- Units inference order is controlled by `units.import_guess_order` (importers may use it; policy only).
