# FFI Contract (v1.0 frozen)

Envelope:
- success: `{ "ok": true, "data": <json>, "reason": null }`
- error: `{ "ok": false, "data": null, "reason": { code,severity,user_msg_key,params,debug,cause } }`

Memory ownership:
- all returned `char*` must be released via `craftcad_free_string`.

Exported symbols (snapshot):

- `craftcad_free_string`
- `craftcad_load_diycad_json`
- `craftcad_extract_faces`
- `craftcad_history_apply_create_part`
- `craftcad_history_apply_create_part_from_face`
- `craftcad_history_apply_update_part`
- `craftcad_history_apply_delete_part`
- `craftcad_history_apply_run_nesting`
- `craftcad_history_apply_edit_placement`
- `craftcad_export_tiled_pdf`
- `craftcad_export_drawing_pdf`
- `craftcad_export_svg`
- `craftcad_export_bom_csv_bytes`
- `craftcad_geom_project_point`
- `craftcad_geom_intersect`
- `craftcad_geom_split_at_t`
- `craftcad_i18n_resolve_message`
- `craftcad_history_new`
- `craftcad_history_free`
- `craftcad_history_apply_fillet`
- `craftcad_history_apply_chamfer`
- `craftcad_history_apply_mirror`
- `craftcad_history_apply_pattern`
- `craftcad_geom_candidates_for_operation`
- `craftcad_history_apply_create_rect`
- `craftcad_history_apply_create_circle`
- `craftcad_history_apply_create_arc`
- `craftcad_history_apply_create_polyline`
- `craftcad_history_apply_create_line`
- `craftcad_history_apply_transform_selection`
- `craftcad_history_apply_offset_entity`
- `craftcad_history_apply_trim_entity`
- `craftcad_history_apply_trim_entity_with_candidate_index`
- `craftcad_history_undo`
- `craftcad_history_redo`
- `craftcad_history_begin_group`
- `craftcad_history_end_group`
- `craftcad_export_diagnostic_pack`

ffi_symbols_sha256: `4c79f268cb54bd27ef183d3faf1f810f22316bcf39b1233a277f5f297ccc99c8`
