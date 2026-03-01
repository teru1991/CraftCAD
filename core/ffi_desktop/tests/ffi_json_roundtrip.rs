use craftcad_serialize::{Document, Layer, Vec2};
use std::ffi::{CStr, CString};
use uuid::Uuid;

unsafe fn take(ptr: *mut std::ffi::c_char) -> String {
    let s = CStr::from_ptr(ptr).to_string_lossy().to_string();
    craftcad_ffi_desktop::craftcad_free_string(ptr);
    s
}

fn sample_doc() -> Document {
    Document {
        schema_version: 1,
        id: Uuid::new_v4(),
        units: "mm".to_string(),
        layers: vec![Layer {
            id: Uuid::new_v4(),
            name: "Default".into(),
            visible: true,
            locked: false,
            editable: true,
        }],
        entities: vec![],
        parts: vec![],
        jobs: vec![],
        materials: vec![],
        settings: craftcad_serialize::ProjectSettings::default(),
    }
}

#[test]
fn history_apply_undo_redo_roundtrip() {
    let h = craftcad_ffi_desktop::craftcad_history_new();
    let mut doc = sample_doc();
    let base = serde_json::to_value(&doc).expect("serialize");

    let doc_s = CString::new(serde_json::to_string(&doc).expect("doc json")).expect("cstring");
    let layer = CString::new(doc.layers[0].id.to_string()).expect("cstring");
    let a = CString::new("{\"x\":0,\"y\":0}").expect("cstring");
    let b = CString::new("{\"x\":10,\"y\":0}").expect("cstring");

    let res = unsafe {
        take(craftcad_ffi_desktop::craftcad_history_apply_create_line(
            h,
            doc_s.as_ptr(),
            layer.as_ptr(),
            a.as_ptr(),
            b.as_ptr(),
        ))
    };
    let v: serde_json::Value = serde_json::from_str(&res).expect("envelope");
    assert_eq!(v["ok"], true);
    doc = serde_json::from_value(v["data"]["document"].clone()).expect("doc value");
    assert_eq!(doc.entities.len(), 1);

    let doc_s = CString::new(serde_json::to_string(&doc).expect("doc json")).expect("cstring");
    let res = unsafe {
        take(craftcad_ffi_desktop::craftcad_history_undo(
            h,
            doc_s.as_ptr(),
        ))
    };
    let v: serde_json::Value = serde_json::from_str(&res).expect("envelope");
    doc = serde_json::from_value(v["data"]["document"].clone()).expect("doc value");
    assert_eq!(serde_json::to_value(&doc).expect("serialize"), base);

    let doc_s = CString::new(serde_json::to_string(&doc).expect("doc json")).expect("cstring");
    let res = unsafe {
        take(craftcad_ffi_desktop::craftcad_history_redo(
            h,
            doc_s.as_ptr(),
        ))
    };
    let v: serde_json::Value = serde_json::from_str(&res).expect("envelope");
    doc = serde_json::from_value(v["data"]["document"].clone()).expect("doc value");
    assert_eq!(doc.entities.len(), 1);

    craftcad_ffi_desktop::craftcad_history_free(h);
}
