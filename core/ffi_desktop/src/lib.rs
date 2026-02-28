use craftcad_commands::commands::create_line::{CreateLineCommand, CreateLineInput};
use craftcad_commands::{Command, CommandContext, History};
use craftcad_serialize::{load_diycad, Document, Reason, ReasonCode, Vec2};
use diycad_geom::{intersect, project_point, split_at, EpsilonPolicy, Geom2D, SplitBy};
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::{c_char, CStr, CString};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;

#[derive(Serialize)]
struct Envelope<T: Serialize> {
    ok: bool,
    data: Option<T>,
    reason: Option<serde_json::Value>,
}

fn reason_json(reason: &Reason) -> serde_json::Value {
    serde_json::json!({
        "code": reason.code,
        "severity": "Warn",
        "user_msg_key": reason.code.to_lowercase(),
        "params": reason.params,
        "debug": reason.debug,
        "cause": serde_json::Value::Null
    })
}

fn encode_ok<T: Serialize>(data: T) -> *mut c_char {
    let env = Envelope {
        ok: true,
        data: Some(data),
        reason: None,
    };
    encode_json(&env)
}

fn encode_err(reason: Reason) -> *mut c_char {
    let env: Envelope<serde_json::Value> = Envelope {
        ok: false,
        data: None,
        reason: Some(reason_json(&reason)),
    };
    encode_json(&env)
}

fn encode_json<T: Serialize>(v: &T) -> *mut c_char {
    match serde_json::to_string(v)
        .ok()
        .and_then(|s| CString::new(s).ok())
    {
        Some(s) => s.into_raw(),
        None => std::ptr::null_mut(),
    }
}

fn parse_cstr(ptr: *const c_char, key: &str) -> std::result::Result<String, Reason> {
    if ptr.is_null() {
        let mut r = Reason::from_code(ReasonCode::SerializePackageCorrupted);
        r.debug
            .insert(key.to_string(), serde_json::json!("null_ptr"));
        return Err(r);
    }
    let s = unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map(|s| s.to_string())
        .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
    Ok(s)
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_load_diycad_json(path_utf8: *const c_char) -> *mut c_char {
    let path = match parse_cstr(path_utf8, "path") {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    match load_diycad(Path::new(&path)) {
        Ok((manifest, document)) => {
            encode_ok(serde_json::json!({"manifest":manifest, "document":document}))
        }
        Err(r) => encode_err(r),
    }
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_geom_project_point(
    geom_json: *const c_char,
    point_json: *const c_char,
    eps_json: *const c_char,
) -> *mut c_char {
    match (|| -> craftcad_serialize::Result<serde_json::Value> {
        let g: Geom2D = serde_json::from_str(&parse_cstr(geom_json, "geom")?)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
        let p: diycad_geom::Vec2 = serde_json::from_str(&parse_cstr(point_json, "point")?)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
        let eps: EpsilonPolicy = serde_json::from_str(&parse_cstr(eps_json, "eps")?)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
        let h = project_point(&g, p, &eps)?;
        Ok(serde_json::to_value(h)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?)
    })() {
        Ok(v) => encode_ok(v),
        Err(r) => encode_err(r),
    }
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_geom_intersect(
    a_json: *const c_char,
    b_json: *const c_char,
    eps_json: *const c_char,
) -> *mut c_char {
    match (|| -> craftcad_serialize::Result<serde_json::Value> {
        let a: Geom2D = serde_json::from_str(&parse_cstr(a_json, "a")?)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
        let b: Geom2D = serde_json::from_str(&parse_cstr(b_json, "b")?)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
        let eps: EpsilonPolicy = serde_json::from_str(&parse_cstr(eps_json, "eps")?)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
        let out = intersect(&a, &b, &eps)?;
        Ok(serde_json::to_value(out)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?)
    })() {
        Ok(v) => encode_ok(v),
        Err(r) => encode_err(r),
    }
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_geom_split_at_t(
    geom_json: *const c_char,
    t: f64,
    eps_json: *const c_char,
) -> *mut c_char {
    match (|| -> craftcad_serialize::Result<serde_json::Value> {
        let g: Geom2D = serde_json::from_str(&parse_cstr(geom_json, "geom")?)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
        let eps: EpsilonPolicy = serde_json::from_str(&parse_cstr(eps_json, "eps")?)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
        let out = split_at(&g, SplitBy::T(t), &eps)?;
        Ok(serde_json::to_value(out)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?)
    })() {
        Ok(v) => encode_ok(v),
        Err(r) => encode_err(r),
    }
}

static HISTORIES: OnceLock<Mutex<HashMap<u64, History>>> = OnceLock::new();
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn histories() -> &'static Mutex<HashMap<u64, History>> {
    HISTORIES.get_or_init(|| Mutex::new(HashMap::new()))
}

#[no_mangle]
pub extern "C" fn craftcad_history_new() -> u64 {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    if let Ok(mut m) = histories().lock() {
        m.insert(id, History::new());
    }
    id
}

#[no_mangle]
pub extern "C" fn craftcad_history_free(handle: u64) {
    if let Ok(mut m) = histories().lock() {
        m.remove(&handle);
    }
}

fn with_history_doc<F>(handle: u64, doc_json: *const c_char, f: F) -> *mut c_char
where
    F: FnOnce(&mut History, &mut Document) -> craftcad_serialize::Result<()>,
{
    let mut doc: Document = match unsafe { parse_cstr(doc_json, "doc_json") }.and_then(|s| {
        serde_json::from_str(&s)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))
    }) {
        Ok(d) => d,
        Err(r) => return encode_err(r),
    };
    let mut map = match histories().lock() {
        Ok(m) => m,
        Err(_) => return encode_err(Reason::from_code(ReasonCode::CoreInvariantViolation)),
    };
    let h = match map.get_mut(&handle) {
        Some(h) => h,
        None => return encode_err(Reason::from_code(ReasonCode::CoreInvariantViolation)),
    };
    match f(h, &mut doc) {
        Ok(()) => encode_ok(serde_json::json!({"document": doc})),
        Err(r) => encode_err(r),
    }
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_create_line(
    handle: u64,
    doc_json: *const c_char,
    layer_id_uuid: *const c_char,
    a_json: *const c_char,
    b_json: *const c_char,
) -> *mut c_char {
    let layer_id = match parse_cstr(layer_id_uuid, "layer_id").and_then(|s| {
        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let a: Vec2 = match parse_cstr(a_json, "a").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::GeomInvalidNumeric))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let b: Vec2 = match parse_cstr(b_json, "b").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::GeomInvalidNumeric))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    with_history_doc(handle, doc_json, |h, doc| {
        let mut cmd = CreateLineCommand::new(layer_id);
        cmd.begin(&CommandContext::default())?;
        cmd.update(CreateLineInput { a, b })?;
        let delta = cmd.commit()?;
        delta.apply(doc)?;
        h.push(delta);
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn craftcad_history_undo(handle: u64, doc_json: *const c_char) -> *mut c_char {
    with_history_doc(handle, doc_json, |h, doc| h.undo(doc))
}

#[no_mangle]
pub extern "C" fn craftcad_history_redo(handle: u64, doc_json: *const c_char) -> *mut c_char {
    with_history_doc(handle, doc_json, |h, doc| h.redo(doc))
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_history_begin_group(
    handle: u64,
    name_utf8: *const c_char,
) -> *mut c_char {
    let name = match parse_cstr(name_utf8, "name") {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let mut map = match histories().lock() {
        Ok(m) => m,
        Err(_) => return encode_err(Reason::from_code(ReasonCode::CoreInvariantViolation)),
    };
    if let Some(h) = map.get_mut(&handle) {
        h.begin_group(name);
        return encode_ok(serde_json::json!({}));
    }
    encode_err(Reason::from_code(ReasonCode::CoreInvariantViolation))
}

#[no_mangle]
pub extern "C" fn craftcad_history_end_group(handle: u64) -> *mut c_char {
    let mut map = match histories().lock() {
        Ok(m) => m,
        Err(_) => return encode_err(Reason::from_code(ReasonCode::CoreInvariantViolation)),
    };
    if let Some(h) = map.get_mut(&handle) {
        h.end_group();
        return encode_ok(serde_json::json!({}));
    }
    encode_err(Reason::from_code(ReasonCode::CoreInvariantViolation))
}
