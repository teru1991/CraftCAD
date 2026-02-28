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
}

impl ReasonCode {
    fn as_str(self) -> &'static str {
        match self {
            Self::SerializeSchemaValidationFailed => "SERIALIZE_SCHEMA_VALIDATION_FAILED",
            Self::SerializePackageCorrupted => "SERIALIZE_PACKAGE_CORRUPTED",
            Self::SerializeUnsupportedSchemaVersion => "SERIALIZE_UNSUPPORTED_SCHEMA_VERSION",
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
pub struct Document {
    pub schema_version: u32,
    pub id: Uuid,
    pub units: String,
    pub layers: Vec<Layer>,
    pub entities: Vec<Entity>,
    pub parts: Vec<Part>,
    pub jobs: Vec<NestJob>,
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
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestObjective {
    pub w_utilization: f64,
    pub w_sheet_count: f64,
    pub w_cut_count: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestJob {
    pub id: Uuid,
    pub sheet_defs: Vec<SheetDef>,
    pub parts_ref: Vec<Uuid>,
    pub constraints: NestConstraints,
    pub objective: NestObjective,
    pub seed: u64,
    pub result: Option<serde_json::Value>,
    pub trace: Option<serde_json::Value>,
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

pub fn validate_document_json_str(s: &str) -> Result<serde_json::Value> {
    let val: serde_json::Value = serde_json::from_str(s).map_err(|e| {
        Reason::from_code(ReasonCode::SerializeSchemaValidationFailed)
            .with_debug("document_json_parse_error", e.to_string())
    })?;
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
