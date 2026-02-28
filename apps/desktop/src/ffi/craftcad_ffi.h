#pragma once
#include <cstdint>

extern "C" {
char* craftcad_load_diycad_json(const char* path_utf8);
char* craftcad_geom_project_point(const char* geom_json,const char* point_json,const char* eps_json);
char* craftcad_geom_intersect(const char* a_json,const char* b_json,const char* eps_json);
char* craftcad_geom_split_at_t(const char* geom_json,double t,const char* eps_json);
void craftcad_free_string(char* ptr);

uint64_t craftcad_history_new(void);
void craftcad_history_free(uint64_t h);
char* craftcad_history_apply_create_line(uint64_t h,const char* doc_json,const char* layer_id_uuid,const char* a_json,const char* b_json);
char* craftcad_history_apply_transform_selection(uint64_t h,const char* doc_json,const char* selection_json,const char* transform_json,const char* eps_json);
char* craftcad_history_undo(uint64_t h,const char* doc_json);
char* craftcad_history_redo(uint64_t h,const char* doc_json);
char* craftcad_history_begin_group(uint64_t h,const char* name_utf8);
char* craftcad_history_end_group(uint64_t h);
}
