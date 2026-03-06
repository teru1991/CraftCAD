#ifndef CRAFTCAD_DESKTOP_FFI_H
#define CRAFTCAD_DESKTOP_FFI_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

char *craftcad_load_diycad_json(const char *path_utf8);
char *craftcad_export_bom_csv_bytes(const char *doc_json, const char *bom_options_json);
char *craftcad_export_tiled_pdf(const char *doc_json, const char *options_json);
char *craftcad_export_drawing_pdf(const char *doc_json, const char *options_json);
char *craftcad_export_svg(const char *doc_json, const char *options_json);
char *craftcad_export_diagnostic_pack(const char *doc_json, const char *options_json);
char *craftcad_extract_faces(const char *doc_json, const char *eps_json);
char *craftcad_geom_project_point(const char *geom_json, const char *point_json, const char *eps_json);
char *craftcad_geom_intersect(const char *a_json, const char *b_json, const char *eps_json);
char *craftcad_geom_split_at_t(const char *geom_json, double t, const char *eps_json);
void craftcad_free_string(char *ptr);
char *craftcad_i18n_resolve_message(const char *user_msg_key_utf8, const char *params_json, const char *locale_utf8);

typedef struct craftcad_aabb_t {
  double min_x;
  double min_y;
  double min_z;
  double max_x;
  double max_y;
  double max_z;
} craftcad_aabb_t;

typedef struct craftcad_part_box_t {
  unsigned char part_id_utf8[37];
  craftcad_aabb_t aabb;
  uint32_t color_rgba;
} craftcad_part_box_t;

int craftcad_view3d_get_part_boxes(const char *project_path_utf8, craftcad_part_box_t **out_ptr, size_t *out_len);
void craftcad_view3d_free_part_boxes(craftcad_part_box_t *ptr, size_t len);
char *craftcad_last_error_message(void);

typedef struct craftcad_projection_lite_hashes_t {
  unsigned char front_hash_hex[65];
  unsigned char top_hash_hex[65];
  unsigned char side_hash_hex[65];
  size_t part_count;
} craftcad_projection_lite_hashes_t;

int craftcad_projection_lite_hashes(const char *project_path_utf8, craftcad_projection_lite_hashes_t *out_hashes);

typedef struct craftcad_estimate_lite_hash_t {
  unsigned char hash_hex[65];
  size_t item_count;
  unsigned char first_material_id_utf8[37];
} craftcad_estimate_lite_hash_t;

int craftcad_estimate_lite_hash(const char *project_path_utf8, craftcad_estimate_lite_hash_t *out_est);

uint64_t craftcad_history_new(void);
void craftcad_history_free(uint64_t h);
char *craftcad_history_apply_create_line(uint64_t h, const char *doc_json, const char *layer_id_uuid, const char *a_json, const char *b_json);
char *craftcad_history_apply_create_rect(uint64_t h, const char *doc_json, const char *layer_id_uuid, const char *rect_params_json, const char *eps_json);
char *craftcad_history_apply_create_circle(uint64_t h, const char *doc_json, const char *layer_id_uuid, const char *circle_params_json, const char *eps_json);
char *craftcad_history_apply_create_arc(uint64_t h, const char *doc_json, const char *layer_id_uuid, const char *arc_params_json, const char *eps_json);
char *craftcad_history_apply_create_polyline(uint64_t h, const char *doc_json, const char *layer_id_uuid, const char *polyline_params_json, const char *eps_json);
char *craftcad_history_apply_fillet(uint64_t h, const char *doc_json, const char *fillet_json, const char *eps_json);
char *craftcad_history_apply_chamfer(uint64_t h, const char *doc_json, const char *chamfer_json, const char *eps_json);
char *craftcad_history_apply_mirror(uint64_t h, const char *doc_json, const char *mirror_json, const char *eps_json);
char *craftcad_history_apply_pattern(uint64_t h, const char *doc_json, const char *pattern_json, const char *eps_json);
char *craftcad_geom_candidates_for_operation(const char *op_json);
char *craftcad_history_apply_transform_selection(uint64_t h, const char *doc_json, const char *selection_json, const char *transform_json, const char *eps_json);
char *craftcad_history_apply_offset_entity(uint64_t h, const char *doc_json, const char *entity_id_uuid, double dist, const char *eps_json);
char *craftcad_history_apply_trim_entity(uint64_t h, const char *doc_json, const char *target_id_uuid, const char *cutter_id_uuid, const char *pick_point_json, const char *eps_json);
char *craftcad_history_apply_trim_entity_with_candidate_index(uint64_t h, const char *doc_json, const char *target_id_uuid, const char *cutter_id_uuid, const char *pick_point_json, const char *eps_json, int candidate_index);
char *craftcad_history_apply_create_part(uint64_t h, const char *doc_json, const char *part_json);
char *craftcad_history_apply_create_part_from_face(uint64_t h, const char *doc_json, const char *face_json, const char *part_props_json);
char *craftcad_history_apply_update_part(uint64_t h, const char *doc_json, const char *part_id_uuid, const char *patch_json);
char *craftcad_history_apply_delete_part(uint64_t h, const char *doc_json, const char *part_id_uuid);
char *craftcad_history_undo(uint64_t h, const char *doc_json);
char *craftcad_history_redo(uint64_t h, const char *doc_json);
char *craftcad_history_begin_group(uint64_t h, const char *name_utf8);
char *craftcad_history_end_group(uint64_t h);
char *craftcad_history_apply_run_nesting(uint64_t h, const char *doc_json, const char *job_id_uuid, const char *eps_json, const char *limits_json);
char *craftcad_history_apply_edit_placement(uint64_t h, const char *doc_json, const char *job_id_uuid, const char *part_id_uuid, int sheet_index, const char *new_pose_json);

#ifdef __cplusplus
}
#endif

#endif
