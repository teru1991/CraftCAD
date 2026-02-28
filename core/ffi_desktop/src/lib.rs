use craftcad_serialize::{load_diycad, Reason, ReasonCode};
use serde::Serialize;
use std::ffi::{c_char, CStr, CString};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static LAST_ERROR_JSON: OnceLock<Mutex<String>> = OnceLock::new();

#[derive(Serialize)]
struct FfiError {
    code: String,
    message: String,
}

fn set_last_error_json(value: String) {
    let cell = LAST_ERROR_JSON.get_or_init(|| Mutex::new(String::new()));
    if let Ok(mut slot) = cell.lock() {
        *slot = value;
    }
}

fn reason_to_json(reason: &Reason) -> String {
    serde_json::to_string(reason).unwrap_or_else(|_| {
        String::from(
            "{\"code\":\"SERIALIZE_PACKAGE_CORRUPTED\",\"message\":\"failed to serialize reason\"}",
        )
    })
}

fn simple_error_json(code: ReasonCode, message: &str) -> String {
    let err = FfiError {
        code: Reason::from_code(code).code,
        message: message.to_string(),
    };
    serde_json::to_string(&err).unwrap_or_else(|_| {
        String::from("{\"code\":\"SERIALIZE_PACKAGE_CORRUPTED\",\"message\":\"error\"}")
    })
}

fn string_to_c_ptr(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(v) => v.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_desktop_load_diycad_json(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        set_last_error_json(simple_error_json(
            ReasonCode::SerializePackageCorrupted,
            "path pointer is null",
        ));
        return std::ptr::null_mut();
    }

    let c_path = CStr::from_ptr(path);
    let path_str = match c_path.to_str() {
        Ok(v) => v,
        Err(_) => {
            set_last_error_json(simple_error_json(
                ReasonCode::SerializePackageCorrupted,
                "path is not valid UTF-8",
            ));
            return std::ptr::null_mut();
        }
    };

    let document = match load_diycad(Path::new(path_str)) {
        Ok((_manifest, doc)) => doc,
        Err(reason) => {
            set_last_error_json(reason_to_json(&reason));
            return std::ptr::null_mut();
        }
    };

    match serde_json::to_string(&document) {
        Ok(json) => {
            let ptr = string_to_c_ptr(json);
            if ptr.is_null() {
                set_last_error_json(simple_error_json(
                    ReasonCode::SerializePackageCorrupted,
                    "failed to allocate output string",
                ));
            }
            ptr
        }
        Err(_) => {
            set_last_error_json(simple_error_json(
                ReasonCode::SerializePackageCorrupted,
                "failed to encode document json",
            ));
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn craftcad_desktop_last_error_json() -> *mut c_char {
    let cell = LAST_ERROR_JSON.get_or_init(|| Mutex::new(String::new()));
    let payload = if let Ok(slot) = cell.lock() {
        if slot.is_empty() {
            String::from("{}")
        } else {
            slot.clone()
        }
    } else {
        String::from("{}")
    };

    string_to_c_ptr(payload)
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_desktop_string_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    let _ = CString::from_raw(ptr);
}
