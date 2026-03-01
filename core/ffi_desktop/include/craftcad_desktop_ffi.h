#ifndef CRAFTCAD_DESKTOP_FFI_H
#define CRAFTCAD_DESKTOP_FFI_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

char *craftcad_load_diycad_json(const char *path_utf8);
char *craftcad_extract_faces(const char *doc_json, const char *eps_json);
char *craftcad_geom_project_point(const char *geom_json, const char *point_json, const char *eps_json);
char *craftcad_geom_intersect(const char *a_json, const char *b_json, const char *eps_json);
char *craftcad_geom_split_at_t(const char *geom_json, double t, const char *eps_json);
void craftcad_free_string(char *ptr);

uint64_t craftcad_history_new(void);
void craftcad_history_free(uint64_t h);
char *craftcad_history_apply_create_line(uint64_t h, const char *doc_json, const char *layer_id_uuid, const char *a_json, const char *b_json);
char *craftcad_history_apply_transform_selection(uint64_t h, const char *doc_json, const char *selection_json, const char *transform_json, const char *eps_json);
char *craftcad_history_apply_offset_entity(uint64_t h, const char *doc_json, const char *entity_id_uuid, double dist, const char *eps_json);
char *craftcad_history_apply_trim_entity(uint64_t h, const char *doc_json, const char *target_id_uuid, const char *cutter_id_uuid, const char *pick_point_json, const char *eps_json);
char *craftcad_history_apply_trim_entity_with_candidate_index(uint64_t h, const char *doc_json, const char *target_id_uuid, const char *cutter_id_uuid, const char *pick_point_json, const char *eps_json, int candidate_index);
char *craftcad_history_apply_create_part(uint64_t h, const char *doc_json, const char *part_json);
char *craftcad_history_undo(uint64_t h, const char *doc_json);
char *craftcad_history_redo(uint64_t h, const char *doc_json);
char *craftcad_history_begin_group(uint64_t h, const char *name_utf8);
char *craftcad_history_end_group(uint64_t h);

#ifdef __cplusplus
}
#endif

#endif
