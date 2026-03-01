# Geometry API (Transform)

Transform JSON payloads used by Desktop->FFI command invocation:

- Translate: `{ "type": "Translate", "dx": number, "dy": number }`
- Rotate: `{ "type": "Rotate", "cx": number, "cy": number, "angle_rad": number }`
- Scale: `{ "type": "Scale", "cx": number, "cy": number, "sx": number, "sy": number }`

Selection payload:

- `{ "ids": ["<uuid>", ...] }`


Desktop FFI edit operations:

- `craftcad_history_apply_offset_entity(h, doc_json, entity_id_uuid, dist, eps_json)`
- `craftcad_history_apply_trim_entity(h, doc_json, target_id_uuid, cutter_id_uuid, pick_point_json, eps_json)`
- `craftcad_history_apply_trim_entity_with_candidate_index(h, doc_json, target_id_uuid, cutter_id_uuid, pick_point_json, eps_json, candidate_index)`

- `craftcad_extract_faces(doc_json, eps_json)` -> `{faces:[{outer:[{x,y}...], holes:[[{x,y}...], ...]}...]}`
- `craftcad_history_apply_create_part(h, doc_json, part_json)`


Part/Face and BOM FFI:

- `craftcad_history_apply_create_part_from_face(h, doc_json, face_json, part_props_json)`
- `craftcad_history_apply_update_part(h, doc_json, part_id_uuid, patch_json)` using JSON Merge Patch (RFC7396)
- `craftcad_history_apply_delete_part(h, doc_json, part_id_uuid)`
- `craftcad_export_bom_csv_bytes(doc_json, bom_options_json)` -> `{bytes_base64, filename, mime}`

## Drawing command JSON params (v1)

- RectParams (fixed mode in v1):
  - `{ "mode": "TwoPoint", "p0": {"x":num,"y":num}, "p1": {"x":num,"y":num}, "corner": "Sharp" }`
- CircleParams (fixed mode in v1):
  - `{ "mode": "CenterRadius", "c": {"x":num,"y":num}, "r": num }`
- ArcParams (fixed mode in v1):
  - `{ "mode": "Center", "c": {"x":num,"y":num}, "r": num, "start_angle": num, "end_angle": num, "ccw": bool }`
- PolylineParams:
  - `{ "pts": [{"x":num,"y":num}, ...], "closed": bool }`
