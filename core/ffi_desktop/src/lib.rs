#![allow(
    clippy::default_constructed_unit_structs,
    clippy::missing_safety_doc,
    clippy::result_large_err,
    dead_code
)]

mod editor_bridge;
use craftcad_bom::{generate_bom, write_bom_csv, CsvOptions, RoundingPolicy, UnitPolicy};
use craftcad_commands::commands::advanced_edit::{
    ChamferCommand, ChamferInput, FilletCommand, FilletInput, MirrorCommand, MirrorInput,
    PatternCommand, PatternInput, PatternParams,
};
use craftcad_commands::commands::create_line::{CreateLineCommand, CreateLineInput};
use craftcad_commands::commands::create_part::{
    CreatePartCommand, CreatePartFromFaceCommand, CreatePartFromFaceInput, CreatePartInput,
    DeletePartCommand, PartProps, UpdatePartCommand, UpdatePartInput,
};
use craftcad_commands::commands::create_shapes::{
    ArcParams, CircleParams, CreateArcCommand, CreateArcInput, CreateCircleCommand,
    CreateCircleInput, CreatePolylineCommand, CreatePolylineInput, CreateRectCommand,
    CreateRectInput, PolylineParams, RectParams,
};
use craftcad_commands::commands::nesting::{
    EditPlacementCommand, EditPlacementInput, PlacementPose, RunNestingCommand, RunNestingInput,
};
use craftcad_commands::commands::offset_entity::{OffsetEntityCommand, OffsetEntityInput};
use craftcad_commands::commands::transform_selection::{
    Transform, TransformSelectionCommand, TransformSelectionInput,
};
use craftcad_commands::commands::trim_entity::{TrimEntityCommand, TrimEntityInput};
use craftcad_commands::{Command, CommandContext, History};
use craftcad_diag::{build_diagnostic_pack, DiagnosticOptions};
use craftcad_estimate_lite::{compute_estimate_lite, estimate_hash_hex};
use craftcad_export::{
    export_drawing_pdf, export_svg, export_tiled_pdf, DrawingPdfOptions, SvgExportOptions,
    TiledPdfOptions,
};
use craftcad_faces::{extract_faces, Face};
use craftcad_i18n::resolve_user_message;
use craftcad_projection_lite::{
    project_to_sheet_lite, sheet_hash_hex, Aabb as ProjAabb, PartBox as ProjPartBox, ViewLite,
};
use craftcad_rules_engine::{
    preflight_rules, run_rules_edge_distance, RuleConfig, RuleReport, RuleSeverity,
};
use craftcad_serialize::{load_diycad, Document, Part, Reason, ReasonCode, Vec2};
use diycad_geom::{intersect, project_point, split_at, EpsilonPolicy, Geom2D, SplitBy};
use diycad_nesting::RunLimits;
use diycad_project::load as load_project_file;
use serde::Serialize;
use sha2::{Digest, Sha256};
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

pub const EXPORTED_SYMBOLS: &[&str] = &[
    "craftcad_free_string",
    "craftcad_load_diycad_json",
    "craftcad_extract_faces",
    "craftcad_history_apply_create_part",
    "craftcad_history_apply_create_part_from_face",
    "craftcad_history_apply_update_part",
    "craftcad_history_apply_delete_part",
    "craftcad_history_apply_run_nesting",
    "craftcad_history_apply_edit_placement",
    "craftcad_export_tiled_pdf",
    "craftcad_export_drawing_pdf",
    "craftcad_export_svg",
    "craftcad_export_bom_csv_bytes",
    "craftcad_geom_project_point",
    "craftcad_geom_intersect",
    "craftcad_geom_split_at_t",
    "craftcad_i18n_resolve_message",
    "craftcad_history_new",
    "craftcad_history_free",
    "craftcad_history_apply_fillet",
    "craftcad_history_apply_chamfer",
    "craftcad_history_apply_mirror",
    "craftcad_history_apply_pattern",
    "craftcad_geom_candidates_for_operation",
    "craftcad_history_apply_create_rect",
    "craftcad_history_apply_create_circle",
    "craftcad_history_apply_create_arc",
    "craftcad_history_apply_create_polyline",
    "craftcad_history_apply_create_line",
    "craftcad_history_apply_transform_selection",
    "craftcad_history_apply_offset_entity",
    "craftcad_history_apply_trim_entity",
    "craftcad_history_apply_trim_entity_with_candidate_index",
    "craftcad_history_undo",
    "craftcad_history_redo",
    "craftcad_history_begin_group",
    "craftcad_history_end_group",
    "craftcad_export_diagnostic_pack",
    "craftcad_view3d_get_part_boxes",
    "craftcad_view3d_free_part_boxes",
    "craftcad_last_error_message",
    "craftcad_projection_lite_hashes",
    "craftcad_estimate_lite_hash",
    "craftcad_rules_edge_report",
    "craftcad_rules_edge_free_json",
    "craftcad_export_preflight_check",
];

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
pub unsafe extern "C" fn craftcad_extract_faces(
    doc_json: *const c_char,
    eps_json: *const c_char,
) -> *mut c_char {
    let doc: Document = match parse_cstr(doc_json, "doc_json").and_then(|s| {
        serde_json::from_str(&s)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let eps: EpsilonPolicy = match parse_cstr(eps_json, "eps").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::GeomInvalidNumeric))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let geoms: Vec<_> = doc.entities.iter().map(|e| e.geom.clone()).collect();
    match extract_faces(&geoms, &eps) {
        Ok(fs) => {
            let faces = fs
                .faces
                .into_iter()
                .map(|f| {
                    serde_json::json!({
                        "outer": f.outer.pts,
                        "holes": f.holes.into_iter().map(|h| h.pts).collect::<Vec<_>>()
                    })
                })
                .collect::<Vec<_>>();
            encode_ok(serde_json::json!({"faces": faces}))
        }
        Err(r) => encode_err(r),
    }
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_create_part(
    handle: u64,
    doc_json: *const c_char,
    part_json: *const c_char,
) -> *mut c_char {
    let part: Part = match parse_cstr(part_json, "part_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::PartInvalidOutline))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    with_history_doc(handle, doc_json, |h, doc| {
        let mut cmd = CreatePartCommand::new();
        cmd.begin(&CommandContext::default())?;
        cmd.update(CreatePartInput { part })?;
        let delta = cmd.commit()?;
        delta.apply(doc)?;
        h.push(delta);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_create_part_from_face(
    handle: u64,
    doc_json: *const c_char,
    face_json: *const c_char,
    part_props_json: *const c_char,
) -> *mut c_char {
    let face: Face = match parse_cstr(face_json, "face_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::PartInvalidOutline))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let props: PartProps = match parse_cstr(part_props_json, "part_props_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::PartInvalidFields))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };

    with_history_doc(handle, doc_json, |h, doc| {
        let mut cmd = CreatePartFromFaceCommand::new();
        cmd.begin(&CommandContext::default())?;
        cmd.update(CreatePartFromFaceInput {
            face,
            part_props: props,
        })?;
        let delta = cmd.commit()?;
        delta.apply(doc)?;
        h.push(delta);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_update_part(
    handle: u64,
    doc_json: *const c_char,
    part_id_uuid: *const c_char,
    patch_json: *const c_char,
) -> *mut c_char {
    let part_id = match parse_cstr(part_id_uuid, "part_id").and_then(|s| {
        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let patch: serde_json::Value = match parse_cstr(patch_json, "patch_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::PartInvalidFields))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };

    with_history_doc(handle, doc_json, |h, doc| {
        let before = doc
            .parts
            .iter()
            .find(|p| p.id == part_id)
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
            .clone();
        let mut after_v = serde_json::to_value(&before)
            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        merge_patch(&mut after_v, &patch);
        let after: Part = serde_json::from_value(after_v)
            .map_err(|_| Reason::from_code(ReasonCode::PartInvalidFields))?;
        let mut cmd = UpdatePartCommand::new();
        cmd.begin(&CommandContext::default())?;
        cmd.update(UpdatePartInput { before, after })?;
        let delta = cmd.commit()?;
        delta.apply(doc)?;
        h.push(delta);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_delete_part(
    handle: u64,
    doc_json: *const c_char,
    part_id_uuid: *const c_char,
) -> *mut c_char {
    let part_id = match parse_cstr(part_id_uuid, "part_id").and_then(|s| {
        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };

    with_history_doc(handle, doc_json, |h, doc| {
        let before = doc
            .parts
            .iter()
            .find(|p| p.id == part_id)
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
            .clone();
        let mut cmd = DeletePartCommand::new();
        cmd.begin(&CommandContext::default())?;
        cmd.update(before)?;
        let delta = cmd.commit()?;
        delta.apply(doc)?;
        h.push(delta);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_run_nesting(
    handle: u64,
    doc_json: *const c_char,
    job_id_uuid: *const c_char,
    eps_json: *const c_char,
    limits_json: *const c_char,
) -> *mut c_char {
    let job_id = match parse_cstr(job_id_uuid, "job_id").and_then(|s| {
        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let eps: EpsilonPolicy = match parse_cstr(eps_json, "eps_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::GeomInvalidNumeric))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let limits: RunLimits = match parse_cstr(limits_json, "limits_json").and_then(|s| {
        serde_json::from_str(&s)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    with_history_doc(handle, doc_json, |h, doc| {
        let snapshot = doc.clone();
        let mut cmd = RunNestingCommand::new();
        cmd.begin(&CommandContext::default())?;
        cmd.update(RunNestingInput {
            job_id,
            eps,
            limits,
            doc_snapshot: snapshot,
        })?;
        let delta = cmd.commit()?;
        delta.apply(doc)?;
        h.push(delta);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_edit_placement(
    handle: u64,
    doc_json: *const c_char,
    job_id_uuid: *const c_char,
    part_id_uuid: *const c_char,
    sheet_index: i32,
    new_pose_json: *const c_char,
) -> *mut c_char {
    let job_id = match parse_cstr(job_id_uuid, "job_id").and_then(|s| {
        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let part_id = match parse_cstr(part_id_uuid, "part_id").and_then(|s| {
        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let new_pose: PlacementPose = match parse_cstr(new_pose_json, "new_pose_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::EditInvalidNumeric))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    with_history_doc(handle, doc_json, |h, doc| {
        let old = doc
            .jobs
            .iter()
            .find(|j| j.id == job_id)
            .and_then(|j| j.result.as_ref())
            .and_then(|r| {
                r.placements
                    .iter()
                    .find(|p| p.part_id == part_id && p.sheet_instance_index == sheet_index as u32)
            })
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
        let old_pose = PlacementPose {
            x: old.x,
            y: old.y,
            rotation_deg: old.rotation_deg,
        };

        let mut cmd = EditPlacementCommand::new();
        cmd.begin(&CommandContext::default())?;
        cmd.update(EditPlacementInput {
            job_id,
            part_id,
            sheet_index,
            old_pose,
            new_pose,
        })?;
        let delta = cmd.commit()?;
        delta.apply(doc)?;
        h.push(delta);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_export_tiled_pdf(
    doc_json: *const c_char,
    options_json: *const c_char,
) -> *mut c_char {
    let doc: Document = match parse_cstr(doc_json, "doc_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::ExportIoParseFailed))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let opts: TiledPdfOptions = match parse_cstr(options_json, "options_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::ExportIoParseFailed))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    match export_tiled_pdf(&doc, &opts) {
        Ok(bytes) => {
            use base64::Engine;
            encode_ok(
                serde_json::json!({"bytes_base64": base64::engine::general_purpose::STANDARD.encode(bytes), "filename":"tiled.pdf", "mime":"application/pdf"}),
            )
        }
        Err(r) => encode_err(r),
    }
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_export_drawing_pdf(
    doc_json: *const c_char,
    options_json: *const c_char,
) -> *mut c_char {
    let doc: Document = match parse_cstr(doc_json, "doc_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::ExportIoParseFailed))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let opts: DrawingPdfOptions = match parse_cstr(options_json, "options_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::ExportIoParseFailed))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    match export_drawing_pdf(&doc, &opts) {
        Ok(bytes) => {
            use base64::Engine;
            encode_ok(
                serde_json::json!({"bytes_base64": base64::engine::general_purpose::STANDARD.encode(bytes), "filename":"drawing.pdf", "mime":"application/pdf"}),
            )
        }
        Err(r) => encode_err(r),
    }
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_export_svg(
    doc_json: *const c_char,
    options_json: *const c_char,
) -> *mut c_char {
    let doc: Document = match parse_cstr(doc_json, "doc_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::ExportIoParseFailed))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let opts: SvgExportOptions = match parse_cstr(options_json, "options_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::ExportIoParseFailed))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    match export_svg(&doc, &opts) {
        Ok(text) => {
            use base64::Engine;
            encode_ok(
                serde_json::json!({"bytes_base64": base64::engine::general_purpose::STANDARD.encode(text.as_bytes()), "filename":"drawing.svg", "mime":"image/svg+xml"}),
            )
        }
        Err(r) => encode_err(r),
    }
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_export_bom_csv_bytes(
    doc_json: *const c_char,
    bom_options_json: *const c_char,
) -> *mut c_char {
    let doc: Document = match parse_cstr(doc_json, "doc_json").and_then(|s| {
        serde_json::from_str(&s)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let opts: CsvOptions = match parse_cstr(bom_options_json, "bom_options_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::BomExportFailed))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    match generate_bom(&doc, UnitPolicy, RoundingPolicy).and_then(|t| write_bom_csv(&t, opts)) {
        Ok(bytes) => {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
            encode_ok(serde_json::json!({
                "bytes_base64": b64,
                "filename": "bom.csv",
                "mime": "text/csv"
            }))
        }
        Err(r) => encode_err(r),
    }
}

fn merge_patch(target: &mut serde_json::Value, patch: &serde_json::Value) {
    match (target, patch) {
        (serde_json::Value::Object(t), serde_json::Value::Object(p)) => {
            for (k, v) in p {
                if v.is_null() {
                    t.remove(k);
                } else {
                    merge_patch(t.entry(k.clone()).or_insert(serde_json::Value::Null), v);
                }
            }
        }
        (t, p) => *t = p.clone(),
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
        serde_json::to_value(h)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))
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
        serde_json::to_value(out)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))
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
        serde_json::to_value(out)
            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))
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
pub unsafe extern "C" fn craftcad_i18n_resolve_message(
    user_msg_key_utf8: *const c_char,
    params_json: *const c_char,
    locale_utf8: *const c_char,
) -> *mut c_char {
    let key = match parse_cstr(user_msg_key_utf8, "user_msg_key") {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let params_src = match parse_cstr(params_json, "params_json") {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let locale = parse_cstr(locale_utf8, "locale").unwrap_or_else(|_| "ja-JP".to_string());
    let params: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&params_src).unwrap_or_default();
    let message = resolve_user_message(&key, &params, &locale);
    encode_ok(serde_json::json!({"message": message}))
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
    let mut doc: Document = match parse_cstr(doc_json, "doc_json").and_then(|s| {
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
pub unsafe extern "C" fn craftcad_history_apply_fillet(
    handle: u64,
    doc_json: *const c_char,
    fillet_json: *const c_char,
    _eps_json: *const c_char,
) -> *mut c_char {
    #[derive(serde::Deserialize)]
    struct In {
        e1: String,
        e2: String,
        radius: f64,
    }
    let i: In = match parse_cstr(fillet_json, "fillet_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::DrawInvalidNumeric))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let e1 = match Uuid::parse_str(&i.e1) {
        Ok(v) => v,
        Err(_) => return encode_err(Reason::from_code(ReasonCode::ModelReferenceNotFound)),
    };
    let e2 = match Uuid::parse_str(&i.e2) {
        Ok(v) => v,
        Err(_) => return encode_err(Reason::from_code(ReasonCode::ModelReferenceNotFound)),
    };
    with_history_doc(handle, doc_json, |h, doc| {
        let mut c = FilletCommand::new();
        c.begin(&CommandContext::default())?;
        c.update(FilletInput {
            e1,
            e2,
            radius: i.radius,
        })?;
        let d = c.commit()?;
        d.apply(doc)?;
        h.push(d);
        Ok(())
    })
}
#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_chamfer(
    handle: u64,
    doc_json: *const c_char,
    chamfer_json: *const c_char,
    _eps_json: *const c_char,
) -> *mut c_char {
    #[derive(serde::Deserialize)]
    struct In {
        e1: String,
        e2: String,
        distance: f64,
    }
    let i: In = match parse_cstr(chamfer_json, "chamfer_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::DrawInvalidNumeric))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let e1 = match Uuid::parse_str(&i.e1) {
        Ok(v) => v,
        Err(_) => return encode_err(Reason::from_code(ReasonCode::ModelReferenceNotFound)),
    };
    let e2 = match Uuid::parse_str(&i.e2) {
        Ok(v) => v,
        Err(_) => return encode_err(Reason::from_code(ReasonCode::ModelReferenceNotFound)),
    };
    with_history_doc(handle, doc_json, |h, doc| {
        let mut c = ChamferCommand::new();
        c.begin(&CommandContext::default())?;
        c.update(ChamferInput {
            e1,
            e2,
            distance: i.distance,
        })?;
        let d = c.commit()?;
        d.apply(doc)?;
        h.push(d);
        Ok(())
    })
}
#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_mirror(
    handle: u64,
    doc_json: *const c_char,
    mirror_json: *const c_char,
    _eps_json: *const c_char,
) -> *mut c_char {
    #[derive(serde::Deserialize)]
    struct In {
        selection_ids: Vec<String>,
        axis_a: Vec2,
        axis_b: Vec2,
    }
    let i: In = match parse_cstr(mirror_json, "mirror_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::EditMirrorAxisInvalid))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let mut ids = Vec::new();
    for sid in i.selection_ids {
        let id = match Uuid::parse_str(&sid) {
            Ok(v) => v,
            Err(_) => return encode_err(Reason::from_code(ReasonCode::ModelReferenceNotFound)),
        };
        ids.push(id);
    }
    with_history_doc(handle, doc_json, |h, doc| {
        let mut c = MirrorCommand::new();
        c.begin(&CommandContext::default())?;
        c.update(MirrorInput {
            selection_ids: ids,
            axis_a: i.axis_a,
            axis_b: i.axis_b,
        })?;
        let d = c.commit()?;
        d.apply(doc)?;
        h.push(d);
        Ok(())
    })
}
#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_pattern(
    handle: u64,
    doc_json: *const c_char,
    pattern_json: *const c_char,
    _eps_json: *const c_char,
) -> *mut c_char {
    #[derive(serde::Deserialize)]
    struct In {
        selection_ids: Vec<String>,
        params: PatternParams,
    }
    let i: In = match parse_cstr(pattern_json, "pattern_json").and_then(|s| {
        serde_json::from_str(&s)
            .map_err(|_| Reason::from_code(ReasonCode::EditPatternInvalidParams))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let mut ids = Vec::new();
    for sid in i.selection_ids {
        let id = match Uuid::parse_str(&sid) {
            Ok(v) => v,
            Err(_) => return encode_err(Reason::from_code(ReasonCode::ModelReferenceNotFound)),
        };
        ids.push(id);
    }
    with_history_doc(handle, doc_json, |h, doc| {
        let mut c = PatternCommand::new();
        c.begin(&CommandContext::default())?;
        c.update(PatternInput {
            selection_ids: ids,
            params: i.params,
        })?;
        let d = c.commit()?;
        d.apply(doc)?;
        h.push(d);
        Ok(())
    })
}
#[no_mangle]
pub unsafe extern "C" fn craftcad_geom_candidates_for_operation(
    op_json: *const c_char,
) -> *mut c_char {
    let v: serde_json::Value = match parse_cstr(op_json, "op_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::EditAmbiguousCandidate))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let cands = v
        .get("candidates")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    encode_ok(serde_json::json!({"candidates": cands}))
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_create_rect(
    handle: u64,
    doc_json: *const c_char,
    layer_id_uuid: *const c_char,
    rect_params_json: *const c_char,
    _eps_json: *const c_char,
) -> *mut c_char {
    let layer_id = match parse_cstr(layer_id_uuid, "layer_id").and_then(|s| {
        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let params: RectParams = match parse_cstr(rect_params_json, "rect_params_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::DrawInvalidNumeric))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    with_history_doc(handle, doc_json, |h, doc| {
        let mut cmd = CreateRectCommand::new(layer_id);
        cmd.begin(&CommandContext::default())?;
        cmd.update(CreateRectInput { params })?;
        let d = cmd.commit()?;
        d.apply(doc)?;
        h.push(d);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_create_circle(
    handle: u64,
    doc_json: *const c_char,
    layer_id_uuid: *const c_char,
    circle_params_json: *const c_char,
    _eps_json: *const c_char,
) -> *mut c_char {
    let layer_id = match parse_cstr(layer_id_uuid, "layer_id").and_then(|s| {
        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let params: CircleParams =
        match parse_cstr(circle_params_json, "circle_params_json").and_then(|s| {
            serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::DrawInvalidNumeric))
        }) {
            Ok(v) => v,
            Err(r) => return encode_err(r),
        };
    with_history_doc(handle, doc_json, |h, doc| {
        let mut cmd = CreateCircleCommand::new(layer_id);
        cmd.begin(&CommandContext::default())?;
        cmd.update(CreateCircleInput { params })?;
        let d = cmd.commit()?;
        d.apply(doc)?;
        h.push(d);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_create_arc(
    handle: u64,
    doc_json: *const c_char,
    layer_id_uuid: *const c_char,
    arc_params_json: *const c_char,
    _eps_json: *const c_char,
) -> *mut c_char {
    let layer_id = match parse_cstr(layer_id_uuid, "layer_id").and_then(|s| {
        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let params: ArcParams = match parse_cstr(arc_params_json, "arc_params_json").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::DrawInvalidNumeric))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    with_history_doc(handle, doc_json, |h, doc| {
        let mut cmd = CreateArcCommand::new(layer_id);
        cmd.begin(&CommandContext::default())?;
        cmd.update(CreateArcInput { params })?;
        let d = cmd.commit()?;
        d.apply(doc)?;
        h.push(d);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_create_polyline(
    handle: u64,
    doc_json: *const c_char,
    layer_id_uuid: *const c_char,
    polyline_params_json: *const c_char,
    _eps_json: *const c_char,
) -> *mut c_char {
    let layer_id = match parse_cstr(layer_id_uuid, "layer_id").and_then(|s| {
        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let params: PolylineParams = match parse_cstr(polyline_params_json, "polyline_params_json")
        .and_then(|s| {
            serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::DrawInvalidNumeric))
        }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    with_history_doc(handle, doc_json, |h, doc| {
        let mut cmd = CreatePolylineCommand::new(layer_id);
        cmd.begin(&CommandContext::default())?;
        cmd.update(CreatePolylineInput { params })?;
        let d = cmd.commit()?;
        d.apply(doc)?;
        h.push(d);
        Ok(())
    })
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
pub unsafe extern "C" fn craftcad_history_apply_transform_selection(
    handle: u64,
    doc_json: *const c_char,
    selection_json: *const c_char,
    transform_json: *const c_char,
    _eps_json: *const c_char,
) -> *mut c_char {
    #[derive(serde::Deserialize)]
    struct SelectionIds {
        ids: Vec<String>,
    }

    let selection: SelectionIds = match parse_cstr(selection_json, "selection").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::EditInvalidNumeric))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };

    let selection_ids = {
        let mut ids = Vec::with_capacity(selection.ids.len());
        for s in selection.ids {
            let id = match Uuid::parse_str(&s) {
                Ok(v) => v,
                Err(_) => return encode_err(Reason::from_code(ReasonCode::ModelReferenceNotFound)),
            };
            ids.push(id);
        }
        ids
    };

    let transform: Transform = match parse_cstr(transform_json, "transform").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::EditInvalidNumeric))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };

    with_history_doc(handle, doc_json, |h, doc| {
        let mut cmd = TransformSelectionCommand::new();
        cmd.begin(&CommandContext::default())?;
        cmd.update(TransformSelectionInput {
            selection_ids,
            transform,
        })?;
        let delta = cmd.commit()?;
        delta.apply(doc)?;
        h.push(delta);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_offset_entity(
    handle: u64,
    doc_json: *const c_char,
    entity_id_uuid: *const c_char,
    dist: f64,
    eps_json: *const c_char,
) -> *mut c_char {
    let entity_id = match parse_cstr(entity_id_uuid, "entity_id").and_then(|s| {
        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let eps: EpsilonPolicy = match parse_cstr(eps_json, "eps").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::GeomInvalidNumeric))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };

    with_history_doc(handle, doc_json, |h, doc| {
        let mut cmd = OffsetEntityCommand::new();
        cmd.begin(&CommandContext::default())?;
        cmd.update(OffsetEntityInput {
            entity_id,
            dist,
            eps,
        })?;
        let delta = cmd.commit()?;
        delta.apply(doc)?;
        h.push(delta);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_trim_entity(
    handle: u64,
    doc_json: *const c_char,
    target_id_uuid: *const c_char,
    cutter_id_uuid: *const c_char,
    pick_point_json: *const c_char,
    eps_json: *const c_char,
) -> *mut c_char {
    craftcad_history_apply_trim_entity_with_candidate_index(
        handle,
        doc_json,
        target_id_uuid,
        cutter_id_uuid,
        pick_point_json,
        eps_json,
        -1,
    )
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_history_apply_trim_entity_with_candidate_index(
    handle: u64,
    doc_json: *const c_char,
    target_id_uuid: *const c_char,
    cutter_id_uuid: *const c_char,
    pick_point_json: *const c_char,
    eps_json: *const c_char,
    candidate_index: i32,
) -> *mut c_char {
    let target_id = match parse_cstr(target_id_uuid, "target_id").and_then(|s| {
        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let cutter_id = match parse_cstr(cutter_id_uuid, "cutter_id").and_then(|s| {
        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let pick_point: Vec2 = match parse_cstr(pick_point_json, "pick_point").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::GeomInvalidNumeric))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let eps: EpsilonPolicy = match parse_cstr(eps_json, "eps").and_then(|s| {
        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::GeomInvalidNumeric))
    }) {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };

    with_history_doc(handle, doc_json, |h, doc| {
        let mut cmd = TrimEntityCommand::new();
        cmd.begin(&CommandContext::default())?;
        cmd.update(TrimEntityInput {
            entity_id: target_id,
            cutter_id,
            pick_point,
            eps,
            candidate_index: if candidate_index < 0 {
                None
            } else {
                Some(candidate_index as usize)
            },
        })?;
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

#[no_mangle]
pub unsafe extern "C" fn craftcad_export_diagnostic_pack(
    doc_json: *const c_char,
    options_json: *const c_char,
) -> *mut c_char {
    use base64::Engine;
    let doc_src = match parse_cstr(doc_json, "doc_json") {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let opts_src = match parse_cstr(options_json, "options_json") {
        Ok(v) => v,
        Err(r) => return encode_err(r),
    };
    let opts_v: serde_json::Value = match serde_json::from_str(&opts_src) {
        Ok(v) => v,
        Err(_) => return encode_err(Reason::from_code(ReasonCode::ExportIoParseFailed)),
    };

    let include_doc = opts_v
        .get("include_doc")
        .and_then(|v| v.as_bool())
        .or_else(|| opts_v.get("include_doc_snapshot").and_then(|v| v.as_bool()))
        .unwrap_or(false);
    let include_system = opts_v
        .get("include_system")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let max_logs = opts_v
        .get("max_logs")
        .or_else(|| opts_v.get("latest_n"))
        .and_then(|v| v.as_u64())
        .unwrap_or(50) as usize;
    let reason_logs: Vec<String> = opts_v
        .get("reason_logs")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let opts = DiagnosticOptions {
        include_doc,
        include_system,
        max_logs,
        locale: opts_v
            .get("locale")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        eps: opts_v.get("eps").cloned(),
        seed: opts_v.get("seed").and_then(|v| v.as_u64()),
        settings_digest: opts_v
            .get("settings_digest")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        nesting_limits: opts_v.get("nesting_limits").cloned(),
        reason_logs,
    };

    let bytes = match build_diagnostic_pack(&doc_src, &opts) {
        Ok(v) => v,
        Err(_) => return encode_err(Reason::from_code(ReasonCode::ExportIoWriteFailed)),
    };

    encode_ok(serde_json::json!({
        "bytes_base64": base64::engine::general_purpose::STANDARD.encode(bytes),
        "filename": "diagnostic_pack.zip",
        "mime": "application/zip"
    }))
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CraftcadAabb {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CraftcadPartBox {
    pub part_id_utf8: [u8; 37],
    pub aabb: CraftcadAabb,
    pub color_rgba: u32,
}

static LAST_ERROR: OnceLock<Mutex<String>> = OnceLock::new();

fn set_last_error(msg: impl Into<String>) {
    let lock = LAST_ERROR.get_or_init(|| Mutex::new(String::new()));
    if let Ok(mut g) = lock.lock() {
        *g = msg.into();
    }
}

fn get_last_error() -> String {
    let lock = LAST_ERROR.get_or_init(|| Mutex::new(String::new()));
    lock.lock().map(|g| g.clone()).unwrap_or_default()
}

fn color_for_part_id(part_id: Uuid) -> u32 {
    let mut hasher = Sha256::new();
    hasher.update(part_id.as_bytes());
    let hash = hasher.finalize();
    let r = hash[0] as u32;
    let g = hash[1] as u32;
    let b = hash[2] as u32;
    (r << 24) | (g << 16) | (b << 8) | 0xff
}

fn default_box(thickness: Option<f64>) -> CraftcadAabb {
    let z = thickness.unwrap_or(18.0).max(0.0);
    CraftcadAabb {
        min_x: 0.0,
        min_y: 0.0,
        min_z: 0.0,
        max_x: 100.0,
        max_y: 100.0,
        max_z: z,
    }
}

fn part_id_buf(id: Uuid) -> [u8; 37] {
    let mut buf = [0u8; 37];
    let s = id.to_string();
    let bytes = s.as_bytes();
    buf[..bytes.len()].copy_from_slice(bytes);
    buf[bytes.len()] = 0;
    buf
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CraftcadProjectionLiteHashes {
    pub front_hash_hex: [u8; 65],
    pub top_hash_hex: [u8; 65],
    pub side_hash_hex: [u8; 65],
    pub part_count: usize,
}

fn hash_buf(hash: &str) -> [u8; 65] {
    let mut out = [0u8; 65];
    let bytes = hash.as_bytes();
    let len = bytes.len().min(64);
    out[..len].copy_from_slice(&bytes[..len]);
    out[len] = 0;
    out
}

fn load_ssot_for_project(path: &Path) -> std::result::Result<craftcad_ssot::SsotV1, String> {
    load_project_file(path)
        .map(|project| {
            project.ssot_v1.unwrap_or_else(|| {
                craftcad_ssot::derive_minimal_ssot_v1(
                    "",
                    craftcad_ssot::SsotDeriveConfig::default(),
                )
            })
        })
        .map_err(|e| e.to_string())
}

fn run_edge_report_for_project(path: &Path) -> std::result::Result<RuleReport, String> {
    let ssot = load_ssot_for_project(path)?;
    run_rules_edge_distance(&ssot, RuleConfig::default()).map_err(|e| e.to_string())
}

fn c_string_and_len(s: String) -> std::result::Result<(*mut c_char, usize), String> {
    let len = s.len();
    CString::new(s)
        .map(|c| (c.into_raw(), len))
        .map_err(|e| e.to_string())
}

fn to_projection_part_boxes(ssot: &craftcad_ssot::SsotV1) -> Vec<ProjPartBox> {
    let mut parts = ssot.parts.clone();
    parts.sort_by_key(|p| p.part_id);
    parts
        .into_iter()
        .map(|part| {
            let aabb = match part.manufacturing_outline_2d {
                Some(outline) => ProjAabb {
                    min_x: outline.min_x.min(outline.max_x),
                    min_y: outline.min_y.min(outline.max_y),
                    min_z: 0.0,
                    max_x: outline.max_x.max(outline.min_x),
                    max_y: outline.max_y.max(outline.min_y),
                    max_z: part.thickness_mm.unwrap_or(0.0).max(0.0),
                },
                None => {
                    let d = default_box(part.thickness_mm);
                    ProjAabb {
                        min_x: d.min_x,
                        min_y: d.min_y,
                        min_z: d.min_z,
                        max_x: d.max_x,
                        max_y: d.max_y,
                        max_z: d.max_z,
                    }
                }
            };
            ProjPartBox {
                part_id: part.part_id,
                aabb,
            }
        })
        .collect()
}

fn ssot_to_part_boxes(ssot: &craftcad_ssot::SsotV1) -> Vec<CraftcadPartBox> {
    let mut parts = ssot.parts.clone();
    parts.sort_by_key(|p| p.part_id);
    parts
        .into_iter()
        .map(|part| {
            let aabb = match part.manufacturing_outline_2d {
                Some(outline) => CraftcadAabb {
                    min_x: outline.min_x.min(outline.max_x),
                    min_y: outline.min_y.min(outline.max_y),
                    min_z: 0.0,
                    max_x: outline.max_x.max(outline.min_x),
                    max_y: outline.max_y.max(outline.min_y),
                    max_z: part.thickness_mm.unwrap_or(0.0).max(0.0),
                },
                None => default_box(part.thickness_mm),
            };
            CraftcadPartBox {
                part_id_utf8: part_id_buf(part.part_id),
                aabb,
                color_rgba: color_for_part_id(part.part_id),
            }
        })
        .collect()
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_view3d_get_part_boxes(
    project_path_utf8: *const c_char,
    out_ptr: *mut *mut CraftcadPartBox,
    out_len: *mut usize,
) -> i32 {
    if project_path_utf8.is_null() || out_ptr.is_null() || out_len.is_null() {
        set_last_error("null pointer input");
        return 1;
    }

    let path = match parse_cstr(project_path_utf8, "project_path") {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e.code);
            return 2;
        }
    };

    match load_ssot_for_project(Path::new(&path)) {
        Ok(ssot) => {
            let mut boxes = ssot_to_part_boxes(&ssot);
            let len = boxes.len();
            let ptr = if len == 0 {
                std::ptr::null_mut()
            } else {
                let ptr = boxes.as_mut_ptr();
                std::mem::forget(boxes);
                ptr
            };
            *out_ptr = ptr;
            *out_len = len;
            set_last_error("");
            0
        }
        Err(e) => {
            set_last_error(e.to_string());
            3
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_view3d_free_part_boxes(ptr: *mut CraftcadPartBox, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    let _ = Vec::from_raw_parts(ptr, len, len);
}

#[no_mangle]
pub extern "C" fn craftcad_last_error_message() -> *mut c_char {
    match CString::new(get_last_error()) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_projection_lite_hashes(
    project_path_utf8: *const c_char,
    out_hashes: *mut CraftcadProjectionLiteHashes,
) -> i32 {
    if project_path_utf8.is_null() || out_hashes.is_null() {
        set_last_error("null pointer input");
        return 1;
    }

    let path = match parse_cstr(project_path_utf8, "project_path") {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e.code);
            return 2;
        }
    };

    let ssot = match load_ssot_for_project(Path::new(&path)) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return 3;
        }
    };

    let parts = to_projection_part_boxes(&ssot);
    let front = sheet_hash_hex(&project_to_sheet_lite(ViewLite::Front, parts.clone()));
    let top = sheet_hash_hex(&project_to_sheet_lite(ViewLite::Top, parts.clone()));
    let side = sheet_hash_hex(&project_to_sheet_lite(ViewLite::Side, parts.clone()));

    *out_hashes = CraftcadProjectionLiteHashes {
        front_hash_hex: hash_buf(&front),
        top_hash_hex: hash_buf(&top),
        side_hash_hex: hash_buf(&side),
        part_count: parts.len(),
    };
    set_last_error("");
    0
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CraftcadEstimateLiteHash {
    pub hash_hex: [u8; 65],
    pub item_count: usize,
    pub first_material_id_utf8: [u8; 37],
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_estimate_lite_hash(
    project_path_utf8: *const c_char,
    out_est: *mut CraftcadEstimateLiteHash,
) -> i32 {
    if project_path_utf8.is_null() || out_est.is_null() {
        set_last_error("null pointer input");
        return 1;
    }

    let path = match parse_cstr(project_path_utf8, "project_path") {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e.code);
            return 2;
        }
    };

    let ssot = match load_ssot_for_project(Path::new(&path)) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return 3;
        }
    };

    let est = compute_estimate_lite(&ssot);
    let hash = estimate_hash_hex(&est);
    let first_material_id_utf8 = est
        .items
        .first()
        .map(|i| part_id_buf(i.material_id))
        .unwrap_or([0u8; 37]);

    *out_est = CraftcadEstimateLiteHash {
        hash_hex: hash_buf(&hash),
        item_count: est.items.len(),
        first_material_id_utf8,
    };
    set_last_error("");
    0
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_rules_edge_report(
    project_path_utf8: *const c_char,
    out_json_ptr: *mut *mut c_char,
    out_len: *mut usize,
) -> i32 {
    if project_path_utf8.is_null() || out_json_ptr.is_null() || out_len.is_null() {
        set_last_error("null pointer input");
        return 1;
    }
    let path = match parse_cstr(project_path_utf8, "project_path") {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e.code);
            return 2;
        }
    };

    let report = match run_edge_report_for_project(Path::new(&path)) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return 3;
        }
    };
    let json = match serde_json::to_string(&report) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e.to_string());
            return 4;
        }
    };
    match c_string_and_len(json) {
        Ok((ptr, len)) => {
            *out_json_ptr = ptr;
            *out_len = len;
            set_last_error("");
            0
        }
        Err(e) => {
            set_last_error(e);
            5
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_rules_edge_free_json(ptr: *mut c_char) {
    craftcad_free_string(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn craftcad_export_preflight_check(project_path_utf8: *const c_char) -> i32 {
    if project_path_utf8.is_null() {
        set_last_error("null pointer input");
        return 1;
    }
    let path = match parse_cstr(project_path_utf8, "project_path") {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e.code);
            return 2;
        }
    };
    let ssot = match load_ssot_for_project(Path::new(&path)) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(e);
            return 3;
        }
    };
    match preflight_rules(&ssot, RuleConfig::default()) {
        Ok(report) => {
            let warns = report
                .findings
                .iter()
                .filter(|f| f.severity == RuleSeverity::Warn)
                .count();
            set_last_error(format!("PRECHECK_OK warns={warns}"));
            0
        }
        Err(err) => {
            set_last_error(serde_json::to_string(&err.report).unwrap_or_else(|_| err.reason_code));
            10
        }
    }
}

#[cfg(test)]
mod view3d_tests {
    use super::*;
    use craftcad_ssot::{
        FeatureGraphV1, GrainPolicyV1, MaterialCategoryV1, MaterialV1, PartLabelV1, PartV1, SsotV1,
    };

    fn sample_ssot() -> SsotV1 {
        let material_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        SsotV1::new(
            vec![MaterialV1 {
                material_id,
                category: MaterialCategoryV1::Unspecified,
                name: "unspecified".to_string(),
                thickness_mm: None,
                grain_policy: GrainPolicyV1::None,
                kerf_mm: 2.0,
                margin_mm: 5.0,
                estimate_loss_factor: None,
            }],
            vec![
                PartV1 {
                    part_id: Uuid::parse_str("00000000-0000-0000-0000-0000000000b2").unwrap(),
                    name: "b".to_string(),
                    material_id,
                    quantity: 1,
                    manufacturing_outline_2d: None,
                    thickness_mm: Some(12.0),
                    grain_direction: None,
                    labels: vec![PartLabelV1 {
                        key: "generated".into(),
                        value: "true".into(),
                    }],
                    feature_ids: vec![],
                },
                PartV1 {
                    part_id: Uuid::parse_str("00000000-0000-0000-0000-0000000000a1").unwrap(),
                    name: "a".to_string(),
                    material_id,
                    quantity: 1,
                    manufacturing_outline_2d: Some(craftcad_ssot::ManufacturingOutline2dV1 {
                        min_x: 1.0,
                        min_y: 2.0,
                        max_x: 10.0,
                        max_y: 20.0,
                    }),
                    thickness_mm: Some(5.0),
                    grain_direction: None,
                    labels: vec![],
                    feature_ids: vec![],
                },
            ],
            FeatureGraphV1::empty(),
        )
    }

    #[test]
    fn deterministic_ordering_by_part_id() {
        let boxes = ssot_to_part_boxes(&sample_ssot());
        let first = std::ffi::CStr::from_bytes_until_nul(&boxes[0].part_id_utf8)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(first, "00000000-0000-0000-0000-0000000000a1");
    }

    #[test]
    fn same_part_id_same_color() {
        let id = Uuid::parse_str("00000000-0000-0000-0000-0000000000a1").unwrap();
        assert_eq!(color_for_part_id(id), color_for_part_id(id));
    }

    #[test]
    fn missing_outline_default_box_stable() {
        let boxes = ssot_to_part_boxes(&sample_ssot());
        let b = boxes.iter().find(|b| b.aabb.max_x == 100.0).unwrap();
        assert_eq!(b.aabb.min_x, 0.0);
        assert_eq!(b.aabb.max_z, 12.0);
    }

    #[test]
    fn empty_list_ok() {
        let empty = SsotV1::new(vec![], vec![], craftcad_ssot::FeatureGraphV1::empty());
        let boxes = ssot_to_part_boxes(&empty);
        assert!(boxes.is_empty());
    }

    #[test]
    fn projection_hashes_are_deterministic() {
        let ssot = sample_ssot();
        let parts_a = to_projection_part_boxes(&ssot);
        let mut parts_b = parts_a.clone();
        parts_b.reverse();

        let h1 = sheet_hash_hex(&project_to_sheet_lite(ViewLite::Front, parts_a));
        let h2 = sheet_hash_hex(&project_to_sheet_lite(ViewLite::Front, parts_b));
        assert_eq!(h1, h2);
    }

    #[test]
    fn estimate_hash_is_deterministic() {
        let ssot = sample_ssot();
        let h1 = estimate_hash_hex(&compute_estimate_lite(&ssot));
        let h2 = estimate_hash_hex(&compute_estimate_lite(&ssot));
        assert_eq!(h1, h2);
    }
}
