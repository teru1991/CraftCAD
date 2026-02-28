# Geometry API (Transform)

Transform JSON payloads used by Desktop->FFI command invocation:

- Translate: `{ "type": "Translate", "dx": number, "dy": number }`
- Rotate: `{ "type": "Rotate", "cx": number, "cy": number, "angle_rad": number }`
- Scale: `{ "type": "Scale", "cx": number, "cy": number, "sx": number, "sy": number }`

Selection payload:

- `{ "ids": ["<uuid>", ...] }`
