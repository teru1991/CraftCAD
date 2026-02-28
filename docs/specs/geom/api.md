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
