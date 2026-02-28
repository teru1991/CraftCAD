use diycad_common::paths::logs_dir;
use diycad_common::{collect_basic_diagnostics, init_logging, log_info};
use diycad_project::load;
use std::ffi::{c_char, CStr, CString};
use std::os::raw::c_int;
use std::path::Path;
use std::sync::{Once, OnceLock};

const RC_NULL_PATH: c_int = -1;
const RC_INVALID_UTF8: c_int = -2;
const RC_VALIDATE_FAILED: c_int = -3;

static LOG_INIT: Once = Once::new();
static VERSION_CSTR: OnceLock<CString> = OnceLock::new();

#[no_mangle]
pub extern "C" fn diycad_version() -> *const c_char {
    let cstr = VERSION_CSTR.get_or_init(|| {
        let diagnostics = collect_basic_diagnostics();
        CString::new(diagnostics.app_version).expect("version string must not contain NUL")
    });
    cstr.as_ptr()
}

#[no_mangle]
/// # Safety
///
/// `path` must be a valid, NUL-terminated C string pointer.
/// It may be null, in which case a negative error code is returned.
pub unsafe extern "C" fn diycad_validate_project(path: *const c_char) -> c_int {
    init_logging_once();

    if path.is_null() {
        let _ = log_info("ffi validate failed: null path pointer");
        return RC_NULL_PATH;
    }

    let c_path = CStr::from_ptr(path);
    let Ok(path_str) = c_path.to_str() else {
        let _ = log_info("ffi validate failed: non-utf8 path");
        return RC_INVALID_UTF8;
    };

    match validate_project(path_str) {
        Ok(()) => {
            let _ = log_info("ffi validate succeeded");
            0
        }
        Err(error_code) => {
            let _ = log_info("ffi validate failed: project load/validation error");
            error_code
        }
    }
}

fn init_logging_once() {
    LOG_INIT.call_once(|| {
        if let Some(log_dir) = logs_dir() {
            let _ = init_logging(log_dir);
        }
    });
}

fn validate_project(path: &str) -> Result<(), c_int> {
    let project = load(Path::new(path)).map_err(|_| RC_VALIDATE_FAILED)?;
    if project.manifest.schema_version.trim().is_empty()
        || project.manifest.app_version.trim().is_empty()
        || project.manifest.units.trim().is_empty()
        || project.manifest.created_at.trim().is_empty()
        || project.manifest.modified_at.trim().is_empty()
    {
        return Err(RC_VALIDATE_FAILED);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use diycad_project::{save, DataJson, DiycadProject, Manifest, SUPPORTED_SCHEMA_VERSION};
    use std::ffi::CString;
    use tempfile::tempdir;

    fn sample_project() -> DiycadProject {
        DiycadProject {
            manifest: Manifest {
                schema_version: SUPPORTED_SCHEMA_VERSION.to_string(),
                app_version: "0.1.0".to_string(),
                units: "mm".to_string(),
                created_at: "2026-02-28T00:00:00Z".to_string(),
                modified_at: "2026-02-28T00:00:00Z".to_string(),
            },
            data: DataJson::default(),
            thumbnail_png: None,
        }
    }

    #[test]
    fn validate_project_succeeds_for_valid_file() {
        let dir = tempdir().expect("tempdir");
        let file_path = dir.path().join("ok.diycad");
        save(&file_path, &sample_project()).expect("save");

        let c_path = CString::new(file_path.to_string_lossy().as_bytes()).expect("cstring");
        let rc = unsafe { diycad_validate_project(c_path.as_ptr()) };
        assert_eq!(rc, 0);
    }

    #[test]
    fn null_pointer_returns_negative_code() {
        let rc = unsafe { diycad_validate_project(std::ptr::null()) };
        assert_eq!(rc, RC_NULL_PATH);
    }
}
