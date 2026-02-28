#ifndef CRAFTCAD_DESKTOP_FFI_H
#define CRAFTCAD_DESKTOP_FFI_H

#ifdef __cplusplus
extern "C" {
#endif

char *craftcad_desktop_load_diycad_json(const char *path);
char *craftcad_desktop_last_error_json(void);
void craftcad_desktop_string_free(char *ptr);

#ifdef __cplusplus
}
#endif

#endif
