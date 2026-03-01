use jsonschema::JSONSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::Path,
};
use uuid::Uuid;
use zip::{write::FileOptions, ZipArchive, ZipWriter};

const MANIFEST_PATH: &str = "manifest.json";
const DOC_PATH: &str = "data/document.json";
const ASSETS_DIR: &str = "assets/";

pub type Result<T> = std::result::Result<T, Reason>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reason {
    pub code: String,
    pub params: serde_json::Map<String, serde_json::Value>,
    pub debug: serde_json::Map<String, serde_json::Value>,
}

impl Reason {
    pub fn from_code(code: ReasonCode) -> Self {
        Self {
            code: code.as_str().to_string(),
            params: serde_json::Map::new(),
            debug: serde_json::Map::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ReasonCode {
    SerializeSchemaValidationFailed,
    SerializePackageCorrupted,
    SerializeUnsupportedSchemaVersion,
    ModelReferenceNotFound,
    GeomInvalidNumeric,
    GeomNoIntersection,
    GeomIntersectionAmbiguous,
    GeomDegenerate,
    GeomSplitPointNotOnGeom,
    GeomArcRangeInvalid,
    GeomCircleRadiusInvalid,
    GeomFallbackLimitReached,
    CoreInvariantViolation,
    GeomOffsetSelfIntersection,
    GeomOffsetNotSupported,
    GeomTrimNoIntersection,
    EditAmbiguousTarget,
    EditTrimAmbiguousCandidate,
    EditNoSelection,
    EditTargetLockedOrHidden,
    EditInvalidNumeric,
    EditTransformWouldDegenerate,
    DrawInvalidNumeric,
    DrawConstraintConflict,
    DrawInsufficientInput,
    EditFilletRadiusTooLarge,
    EditChamferDistanceTooLarge,
    EditMirrorAxisInvalid,
    EditPatternInvalidParams,
    EditAmbiguousCandidate,
    EditCandidateNotFound,
    FaceNoClosedLoop,
    FaceSelfIntersection,
    FaceAmbiguousLoop,
    PartInvalidOutline,
    PartInvalidFields,
    MaterialNotFound,
    BomExportFailed,
    ExportPdfFailed,
    ExportUnsupportedEntity,
    ExportUnsupportedFeature,
    ExportIoParseFailed,
    ExportIoWriteFailed,
    NestPartTooLargeForAnySheet,
    NestGrainConstraintBlocksFit,
    NestNoFeasiblePositionWithMarginAndKerf,
    NestNoGoZoneBlocksFit,
    NestStoppedByTimeLimit,
    NestStoppedByIterationLimit,
    NestInternalInfeasible,
}

impl ReasonCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SerializeSchemaValidationFailed => "SERIALIZE_SCHEMA_VALIDATION_FAILED",
            Self::SerializePackageCorrupted => "SERIALIZE_PACKAGE_CORRUPTED",
            Self::SerializeUnsupportedSchemaVersion => "SERIALIZE_UNSUPPORTED_SCHEMA_VERSION",
            Self::ModelReferenceNotFound => "MODEL_REFERENCE_NOT_FOUND",
            Self::GeomInvalidNumeric => "GEOM_INVALID_NUMERIC",
            Self::GeomNoIntersection => "GEOM_NO_INTERSECTION",
            Self::GeomIntersectionAmbiguous => "GEOM_INTERSECTION_AMBIGUOUS",
            Self::GeomDegenerate => "GEOM_DEGENERATE",
            Self::GeomSplitPointNotOnGeom => "GEOM_SPLIT_POINT_NOT_ON_GEOM",
            Self::GeomArcRangeInvalid => "GEOM_ARC_RANGE_INVALID",
            Self::GeomCircleRadiusInvalid => "GEOM_CIRCLE_RADIUS_INVALID",
            Self::GeomFallbackLimitReached => "GEOM_FALLBACK_LIMIT_REACHED",
            Self::CoreInvariantViolation => "CORE_INVARIANT_VIOLATION",
            Self::GeomOffsetSelfIntersection => "GEOM_OFFSET_SELF_INTERSECTION",
            Self::GeomOffsetNotSupported => "GEOM_OFFSET_NOT_SUPPORTED",
            Self::GeomTrimNoIntersection => "GEOM_TRIM_NO_INTERSECTION",
            Self::EditAmbiguousTarget => "EDIT_AMBIGUOUS_TARGET",
            Self::EditTrimAmbiguousCandidate => "EDIT_TRIM_AMBIGUOUS_CANDIDATE",
            Self::EditNoSelection => "EDIT_NO_SELECTION",
            Self::EditTargetLockedOrHidden => "EDIT_TARGET_LOCKED_OR_HIDDEN",
            Self::EditInvalidNumeric => "EDIT_INVALID_NUMERIC",
            Self::EditTransformWouldDegenerate => "EDIT_TRANSFORM_WOULD_DEGENERATE",
            Self::DrawInvalidNumeric => "DRAW_INVALID_NUMERIC",
            Self::DrawConstraintConflict => "DRAW_CONSTRAINT_CONFLICT",
            Self::DrawInsufficientInput => "DRAW_INSUFFICIENT_INPUT",
            Self::EditFilletRadiusTooLarge => "EDIT_FILLET_RADIUS_TOO_LARGE",
            Self::EditChamferDistanceTooLarge => "EDIT_CHAMFER_DISTANCE_TOO_LARGE",
            Self::EditMirrorAxisInvalid => "EDIT_MIRROR_AXIS_INVALID",
            Self::EditPatternInvalidParams => "EDIT_PATTERN_INVALID_PARAMS",
            Self::EditAmbiguousCandidate => "EDIT_AMBIGUOUS_CANDIDATE",
            Self::EditCandidateNotFound => "EDIT_CANDIDATE_NOT_FOUND",
            Self::FaceNoClosedLoop => "FACE_NO_CLOSED_LOOP",
            Self::FaceSelfIntersection => "FACE_SELF_INTERSECTION",
            Self::FaceAmbiguousLoop => "FACE_AMBIGUOUS_LOOP",
            Self::PartInvalidOutline => "PART_INVALID_OUTLINE",
            Self::PartInvalidFields => "PART_INVALID_FIELDS",
            Self::MaterialNotFound => "MATERIAL_NOT_FOUND",
            Self::BomExportFailed => "BOM_EXPORT_FAILED",
            Self::ExportPdfFailed => "EXPORT_PDF_FAILED",
            Self::ExportUnsupportedEntity => "EXPORT_UNSUPPORTED_ENTITY",
            Self::ExportUnsupportedFeature => "EXPORT_UNSUPPORTED_FEATURE",
            Self::ExportIoParseFailed => "EXPORT_IO_PARSE_FAILED",
            Self::ExportIoWriteFailed => "EXPORT_IO_WRITE_FAILED",
            Self::NestPartTooLargeForAnySheet => "NEST_PART_TOO_LARGE_FOR_ANY_SHEET",
            Self::NestGrainConstraintBlocksFit => "NEST_GRAIN_CONSTRAINT_BLOCKS_FIT",
            Self::NestNoFeasiblePositionWithMarginAndKerf => {
                "NEST_NO_FEASIBLE_POSITION_WITH_MARGIN_AND_KERF"
            }
            Self::NestNoGoZoneBlocksFit => "NEST_NO_GO_ZONE_BLOCKS_FIT",
            Self::NestStoppedByTimeLimit => "NEST_STOPPED_BY_TIME_LIMIT",
            Self::NestStoppedByIterationLimit => "NEST_STOPPED_BY_ITERATION_LIMIT",
            Self::NestInternalInfeasible => "NEST_INTERNAL_INFEASIBLE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestApp {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: u32,
    pub app: ManifestApp,
    pub created_at: String,
    pub document_path: String,
    pub assets_path: String,
    #[serde(default)]
    pub settings_digest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MaterialCategory {
    Wood,
    Leather,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetDefault {
    pub width: f64,
    pub height: f64,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub id: Uuid,
    pub name: String,
    pub category: MaterialCategory,
    pub thickness_mm: Option<f64>,
    #[serde(default)]
    pub sheet_default: Option<SheetDefault>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectSettings {
    #[serde(default)]
    pub bom_delimiter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub schema_version: u32,
    pub id: Uuid,
    pub units: String,
    pub layers: Vec<Layer>,
    pub entities: Vec<Entity>,
    pub parts: Vec<Part>,
    pub jobs: Vec<NestJob>,
    #[serde(default)]
    pub materials: Vec<Material>,
    #[serde(default)]
    pub settings: ProjectSettings,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: Uuid,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub editable: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Geom2D {
    Line {
        a: Vec2,
        b: Vec2,
    },
    Circle {
        c: Vec2,
        r: f64,
    },
    Arc {
        c: Vec2,
        r: f64,
        start_angle: f64,
        end_angle: f64,
        ccw: bool,
    },
    Polyline {
        pts: Vec<Vec2>,
        closed: bool,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub layer_id: Uuid,
    pub geom: Geom2D,
    pub style: serde_json::Value,
    pub tags: Vec<String>,
    pub meta: BTreeMap<String, serde_json::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polygon2D {
    pub outer: Vec<Vec2>,
    pub holes: Vec<Vec<Vec2>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    pub id: Uuid,
    pub name: String,
    pub outline: Polygon2D,
    pub thickness: f64,
    pub quantity: u32,
    pub material_id: Uuid,
    pub grain_dir: Option<f64>,
    pub allow_rotate: bool,
    pub margin: f64,
    pub kerf: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetDef {
    pub id: Uuid,
    pub material_id: Uuid,
    pub width: f64,
    pub height: f64,
    pub quantity: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestConstraints {
    pub global_margin: f64,
    pub global_kerf: f64,
    pub allow_rotate_default: bool,
    #[serde(default)]
    pub no_go_zones: Vec<NoGoZone>,
    #[serde(default)]
    pub grain_policy: GrainPolicy,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub enum GrainPolicy {
    Strict,
    Prefer,
    #[default]
    Ignore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NoGoZone {
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestObjective {
    pub w_utilization: f64,
    pub w_sheet_count: f64,
    pub w_cut_count: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartRef {
    pub part_id: Uuid,
    #[serde(default)]
    pub quantity_override: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Placement {
    pub part_id: Uuid,
    pub sheet_instance_index: u32,
    pub x: f64,
    pub y: f64,
    pub rotation_deg: f64,
    pub bbox: BBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestMetrics {
    pub utilization_per_sheet: Vec<f64>,
    pub sheet_count_used: u32,
    pub cut_count_estimate: u32,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PartPlacementStatusKind {
    Placed,
    Unplaced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartPlacementStatus {
    pub part_id: Uuid,
    pub status: PartPlacementStatusKind,
    #[serde(default)]
    pub reason: Option<Reason>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestResultV1 {
    pub placements: Vec<Placement>,
    pub metrics: NestMetrics,
    pub per_part_status: Vec<PartPlacementStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceBestUpdate {
    pub iter: u32,
    pub score: f64,
    pub sheet_used: u32,
    pub utilization: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestTraceV1 {
    pub seed: u64,
    pub iterations: u32,
    pub time_ms: u64,
    pub stop_reason: String,
    pub best_updates: Vec<TraceBestUpdate>,
    pub failure_stats: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum PartRefCompat {
    Legacy(Uuid),
    Structured(PartRef),
}

fn deserialize_parts_ref<'de, D>(deserializer: D) -> std::result::Result<Vec<PartRef>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = Vec::<PartRefCompat>::deserialize(deserializer)?;
    Ok(raw
        .into_iter()
        .map(|r| match r {
            PartRefCompat::Legacy(id) => PartRef {
                part_id: id,
                quantity_override: None,
            },
            PartRefCompat::Structured(v) => v,
        })
        .collect())
}

fn serialize_parts_ref<S>(v: &[PartRef], serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    v.serialize(serializer)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestJob {
    pub id: Uuid,
    pub sheet_defs: Vec<SheetDef>,
    #[serde(
        deserialize_with = "deserialize_parts_ref",
        serialize_with = "serialize_parts_ref"
    )]
    pub parts_ref: Vec<PartRef>,
    pub constraints: NestConstraints,
    pub objective: NestObjective,
    pub seed: u64,
    pub result: Option<NestResultV1>,
    pub trace: Option<NestTraceV1>,
}

fn compile_schema(raw: &str) -> Result<JSONSchema> {
    let v: serde_json::Value = serde_json::from_str(raw).map_err(|e| {
        Reason::from_code(ReasonCode::SerializeSchemaValidationFailed)
            .with_debug("schema_parse_error", e.to_string())
    })?;
    JSONSchema::compile(&v).map_err(|e| {
        Reason::from_code(ReasonCode::SerializeSchemaValidationFailed)
            .with_debug("schema_compile_error", e.to_string())
    })
}

fn validate_value(schema: &JSONSchema, v: &serde_json::Value) -> Result<()> {
    if let Err(errors) = schema.validate(v) {
        let msgs: Vec<String> = errors.map(|e| e.to_string()).take(20).collect();
        return Err(
            Reason::from_code(ReasonCode::SerializeSchemaValidationFailed)
                .with_debug("errors", serde_json::json!(msgs)),
        );
    }
    Ok(())
}

pub fn validate_manifest_json_str(s: &str) -> Result<serde_json::Value> {
    let val: serde_json::Value = serde_json::from_str(s).map_err(|e| {
        Reason::from_code(ReasonCode::SerializeSchemaValidationFailed)
            .with_debug("manifest_json_parse_error", e.to_string())
    })?;
    let schema = compile_schema(include_str!("../schemas/manifest.schema.json"))?;
    validate_value(&schema, &val)?;
    Ok(val)
}

pub fn normalize_document_json(mut val: serde_json::Value) -> serde_json::Value {
    if let serde_json::Value::Object(ref mut m) = val {
        if !m.contains_key("materials") {
            m.insert("materials".to_string(), serde_json::json!([]));
        }
        if !m.contains_key("settings") {
            m.insert("settings".to_string(), serde_json::json!({}));
        }
    }
    val
}

pub fn validate_document_json_str(s: &str) -> Result<serde_json::Value> {
    let val: serde_json::Value = serde_json::from_str(s).map_err(|e| {
        Reason::from_code(ReasonCode::SerializeSchemaValidationFailed)
            .with_debug("document_json_parse_error", e.to_string())
    })?;
    let val = normalize_document_json(val);
    let schema = compile_schema(include_str!("../schemas/document.schema.json"))?;
    validate_value(&schema, &val)?;
    Ok(val)
}

pub fn create_manifest(app_name: &str, app_version: &str) -> Manifest {
    Manifest {
        schema_version: 1,
        app: ManifestApp {
            name: app_name.to_string(),
            version: app_version.to_string(),
        },
        created_at: time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
        document_path: DOC_PATH.to_string(),
        assets_path: ASSETS_DIR.to_string(),
        settings_digest: None,
    }
}

pub fn save_diycad(path: &Path, manifest: &Manifest, doc: &Document) -> Result<()> {
    let f = std::fs::File::create(path).map_err(|e| {
        Reason::from_code(ReasonCode::SerializePackageCorrupted).with_debug("io", e.to_string())
    })?;
    let mut zip = ZipWriter::new(f);
    let opt = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let m_json = serde_json::to_vec_pretty(manifest).map_err(|e| {
        Reason::from_code(ReasonCode::SerializePackageCorrupted)
            .with_debug("manifest_serialize", e.to_string())
    })?;
    zip.start_file(MANIFEST_PATH, opt).map_err(|e| {
        Reason::from_code(ReasonCode::SerializePackageCorrupted).with_debug("zip", e.to_string())
    })?;
    zip.write_all(&m_json).map_err(|e| {
        Reason::from_code(ReasonCode::SerializePackageCorrupted)
            .with_debug("zip_write", e.to_string())
    })?;

    zip.add_directory("data/", opt).ok();
    let d_json = serde_json::to_vec_pretty(doc).map_err(|e| {
        Reason::from_code(ReasonCode::SerializePackageCorrupted)
            .with_debug("doc_serialize", e.to_string())
    })?;
    zip.start_file(DOC_PATH, opt).map_err(|e| {
        Reason::from_code(ReasonCode::SerializePackageCorrupted).with_debug("zip", e.to_string())
    })?;
    zip.write_all(&d_json).map_err(|e| {
        Reason::from_code(ReasonCode::SerializePackageCorrupted)
            .with_debug("zip_write", e.to_string())
    })?;

    zip.add_directory(ASSETS_DIR, opt).ok();

    zip.finish().map_err(|e| {
        Reason::from_code(ReasonCode::SerializePackageCorrupted)
            .with_debug("zip_finish", e.to_string())
    })?;
    Ok(())
}

pub fn load_diycad(path: &Path) -> Result<(Manifest, Document)> {
    let f = std::fs::File::open(path).map_err(|e| {
        Reason::from_code(ReasonCode::SerializePackageCorrupted).with_debug("io", e.to_string())
    })?;
    let mut zip = ZipArchive::new(f).map_err(|e| {
        Reason::from_code(ReasonCode::SerializePackageCorrupted)
            .with_debug("zip_open", e.to_string())
    })?;

    let mut m_raw = String::new();
    {
        let mut mf = zip.by_name(MANIFEST_PATH).map_err(|e| {
            Reason::from_code(ReasonCode::SerializePackageCorrupted)
                .with_debug("missing_manifest", e.to_string())
        })?;
        mf.read_to_string(&mut m_raw).map_err(|e| {
            Reason::from_code(ReasonCode::SerializePackageCorrupted)
                .with_debug("manifest_read", e.to_string())
        })?;
    }

    let m_val = validate_manifest_json_str(&m_raw)?;
    let manifest: Manifest = serde_json::from_value(m_val).map_err(|e| {
        Reason::from_code(ReasonCode::SerializePackageCorrupted)
            .with_debug("manifest_decode", e.to_string())
    })?;

    if manifest.schema_version != 1 {
        return Err(
            Reason::from_code(ReasonCode::SerializeUnsupportedSchemaVersion)
                .with_param("schema_version", manifest.schema_version),
        );
    }

    let mut d_raw = String::new();
    {
        let mut df = zip.by_name(&manifest.document_path).map_err(|e| {
            Reason::from_code(ReasonCode::SerializePackageCorrupted)
                .with_debug("missing_document", e.to_string())
        })?;
        df.read_to_string(&mut d_raw).map_err(|e| {
            Reason::from_code(ReasonCode::SerializePackageCorrupted)
                .with_debug("doc_read", e.to_string())
        })?;
    }

    let d_val = validate_document_json_str(&d_raw)?;
    let doc: Document = serde_json::from_value(d_val).map_err(|e| {
        Reason::from_code(ReasonCode::SerializePackageCorrupted)
            .with_debug("doc_decode", e.to_string())
    })?;

    Ok((manifest, doc))
}

pub fn digest_settings_json(settings_json: &serde_json::Value) -> String {
    let bytes = serde_json::to_vec(settings_json).unwrap_or_default();
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

trait ReasonExt {
    fn with_param(self, k: &str, v: impl Into<serde_json::Value>) -> Reason;
    fn with_debug(self, k: &str, v: impl Into<serde_json::Value>) -> Reason;
}
impl ReasonExt for Reason {
    fn with_param(mut self, k: &str, v: impl Into<serde_json::Value>) -> Reason {
        self.params.insert(k.to_string(), v.into());
        self
    }
    fn with_debug(mut self, k: &str, v: impl Into<serde_json::Value>) -> Reason {
        self.debug.insert(k.to_string(), v.into());
        self
    }
}
