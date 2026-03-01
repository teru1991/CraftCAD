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
use craftcad_export::{
    export_drawing_pdf, export_svg, export_tiled_pdf, DrawingPdfOptions, SvgExportOptions,
    TiledPdfOptions,
};
use craftcad_faces::{extract_faces, Face};
use craftcad_i18n::resolve_user_message;
use craftcad_serialize::{load_diycad, Document, Part, Reason, ReasonCode, Vec2};
use diycad_geom::{intersect, project_point, split_at, EpsilonPolicy, Geom2D, SplitBy};
use diycad_nesting::RunLimits;
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
