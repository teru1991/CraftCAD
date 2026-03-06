use craftcad_ssot::{
    deterministic_uuid, derive_minimal_ssot_v1, FeatureGraphV1, GrainPolicyV1, MaterialCategoryV1,
    MaterialV1, PartLabelV1, PartV1, SsotDeriveConfig, SsotV1,
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use thiserror::Error;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

pub const DIYCAD_EXTENSION: &str = ".diycad";
pub const SUPPORTED_SCHEMA_VERSION: &str = "0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: String,
    pub app_version: String,
    pub units: String,
    pub created_at: String,
    pub modified_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DataJson {
    #[serde(default)]
    pub entities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiycadProject {
    pub manifest: Manifest,
    pub data: DataJson,
    pub thumbnail_png: Option<Vec<u8>>,
    pub ssot_v1: Option<SsotV1>,
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("project file must end with {0}")]
    InvalidExtension(&'static str),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid zip archive: {0}")]
    InvalidZip(#[from] zip::result::ZipError),
    #[error("manifest.json is missing")]
    MissingManifest,
    #[error("data.json is missing")]
    MissingData,
    #[error("schema version mismatch: expected {expected}, got {actual}")]
    SchemaVersionMismatch { expected: String, actual: String },
    #[error("json parse error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type ProjectResult<T> = Result<T, ProjectError>;

pub fn validate_project_path(path: &str) -> ProjectResult<()> {
    if path.ends_with(DIYCAD_EXTENSION) {
        Ok(())
    } else {
        Err(ProjectError::InvalidExtension(DIYCAD_EXTENSION))
    }
}

pub fn create_empty_project(app_version: &str, units: &str, timestamp: &str) -> DiycadProject {
    DiycadProject {
        manifest: Manifest {
            schema_version: SUPPORTED_SCHEMA_VERSION.to_string(),
            app_version: app_version.to_string(),
            units: units.to_string(),
            created_at: timestamp.to_string(),
            modified_at: timestamp.to_string(),
        },
        data: DataJson::default(),
        thumbnail_png: None,
        ssot_v1: None,
    }
}

pub fn save(path: impl AsRef<Path>, project: &DiycadProject) -> ProjectResult<()> {
    let path_ref = path.as_ref();
    validate_project_path(path_ref.to_string_lossy().as_ref())?;

    let file = File::create(path_ref)?;
    let mut writer = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    writer.start_file("manifest.json", options)?;
    writer.write_all(serde_json::to_string_pretty(&project.manifest)?.as_bytes())?;

    writer.start_file("data.json", options)?;
    writer.write_all(serde_json::to_string_pretty(&project.data)?.as_bytes())?;

    if let Some(ssot) = &project.ssot_v1 {
        writer.start_file("ssot_v1.json", options)?;
        writer.write_all(serde_json::to_string_pretty(ssot)?.as_bytes())?;
    }

    if let Some(thumbnail) = &project.thumbnail_png {
        writer.start_file("assets/thumbnail.png", options)?;
        writer.write_all(thumbnail)?;
    }

    writer.finish()?;
    Ok(())
}

pub fn load(path: impl AsRef<Path>) -> ProjectResult<DiycadProject> {
    let path_ref = path.as_ref();
    validate_project_path(path_ref.to_string_lossy().as_ref())?;

    let file = File::open(path_ref)?;
    let mut zip = ZipArchive::new(file)?;

    let manifest = read_json_file::<Manifest>(&mut zip, "manifest.json")?;
    if manifest.schema_version != SUPPORTED_SCHEMA_VERSION {
        return Err(ProjectError::SchemaVersionMismatch {
            expected: SUPPORTED_SCHEMA_VERSION.to_string(),
            actual: manifest.schema_version,
        });
    }

    let data = read_json_file::<DataJson>(&mut zip, "data.json")?;

    let thumbnail_png = zip
        .by_name("assets/thumbnail.png")
        .ok()
        .map(|mut file| {
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            Ok::<_, std::io::Error>(bytes)
        })
        .transpose()?;

    let project_name = path_ref
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();

    let ssot_v1 = read_json_file_optional::<SsotV1>(&mut zip, "ssot_v1.json")?
        .map(SsotV1::canonicalize)
        .or_else(|| Some(derive_ssot_from_legacy(project_name, &data, SsotDeriveConfig::default())));

    Ok(DiycadProject {
        manifest,
        data,
        thumbnail_png,
        ssot_v1,
    })
}


fn derive_ssot_from_legacy(project_name: &str, data: &DataJson, cfg: SsotDeriveConfig) -> SsotV1 {
    if data.entities.is_empty() {
        return derive_minimal_ssot_v1(project_name, cfg);
    }

    let stable_key = project_name.trim();
    let material_id = deterministic_uuid("material", stable_key);
    let material = MaterialV1 {
        material_id,
        category: MaterialCategoryV1::Unspecified,
        name: "unspecified".to_string(),
        thickness_mm: None,
        grain_policy: GrainPolicyV1::None,
        kerf_mm: cfg.default_kerf_mm,
        margin_mm: cfg.default_margin_mm,
        estimate_loss_factor: None,
    };

    let mut entities = data.entities.clone();
    entities.sort();

    let parts = entities
        .into_iter()
        .map(|entity| {
            let trimmed = entity.trim();
            let part_name = if trimmed.is_empty() {
                "root".to_string()
            } else {
                format!("part:{}", trimmed)
            };
            PartV1 {
                part_id: deterministic_uuid("part", &format!("{}:{}", stable_key, part_name)),
                name: part_name,
                material_id,
                quantity: 1,
                manufacturing_outline_2d: None,
                thickness_mm: None,
                grain_direction: None,
                labels: vec![PartLabelV1 {
                    key: "generated".to_string(),
                    value: "true".to_string(),
                }],
                feature_ids: Vec::new(),
            }
        })
        .collect();

    SsotV1::new(vec![material], parts, FeatureGraphV1::empty()).canonicalize()
}

fn read_json_file<T: for<'de> Deserialize<'de>>(
    zip: &mut ZipArchive<File>,
    path: &str,
) -> ProjectResult<T> {
    let mut file = zip.by_name(path).map_err(|_| match path {
        "manifest.json" => ProjectError::MissingManifest,
        "data.json" => ProjectError::MissingData,
        _ => ProjectError::InvalidZip(zip::result::ZipError::FileNotFound),
    })?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(serde_json::from_str(&content)?)
}

fn read_json_file_optional<T: for<'de> Deserialize<'de>>(
    zip: &mut ZipArchive<File>,
    path: &str,
) -> ProjectResult<Option<T>> {
    match zip.by_name(path) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            Ok(Some(serde_json::from_str(&content)?))
        }
        Err(zip::result::ZipError::FileNotFound) => Ok(None),
        Err(e) => Err(ProjectError::InvalidZip(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use craftcad_ssot::{
        deterministic_uuid, FeatureGraphV1, GrainPolicyV1, MaterialCategoryV1, MaterialV1, PartV1,
        SsotV1,
    };
    use tempfile::tempdir;

    fn sample_project() -> DiycadProject {
        let mut project = create_empty_project("0.1.0", "mm", "2026-02-28T00:00:00Z");
        project.data = DataJson {
            entities: vec!["rect-1".to_string()],
        };
        project.thumbnail_png = Some(vec![137, 80, 78, 71]);
        project
    }

    #[test]
    fn accepts_diycad_extension() {
        assert!(validate_project_path("sample.diycad").is_ok());
    }

    #[test]
    fn create_empty_project_sets_defaults() {
        let project = create_empty_project("0.1.0", "mm", "2026-02-28T00:00:00Z");
        assert_eq!(project.manifest.schema_version, SUPPORTED_SCHEMA_VERSION);
        assert_eq!(project.data.entities.len(), 0);
        assert!(project.thumbnail_png.is_none());
        assert!(project.ssot_v1.is_none());
    }

    #[test]
    fn save_load_roundtrip_preserves_manifest() {
        let dir = tempdir().expect("tempdir must be created");
        let file_path = dir.path().join("roundtrip.diycad");
        let project = sample_project();

        save(&file_path, &project).expect("save should succeed");
        let loaded = load(&file_path).expect("load should succeed");

        assert_eq!(loaded.manifest, project.manifest);
    }

    #[test]
    fn rejects_schema_version_mismatch() {
        let dir = tempdir().expect("tempdir must be created");
        let file_path = dir.path().join("mismatch.diycad");
        let mut project = sample_project();
        project.manifest.schema_version = "999".to_string();

        save(&file_path, &project).expect("save should succeed");
        let err = load(&file_path).expect_err("load should fail");

        assert!(matches!(err, ProjectError::SchemaVersionMismatch { .. }));
    }

    #[test]
    fn roundtrip_persists_ssot_v1() {
        let dir = tempdir().expect("tempdir must be created");
        let file_path = dir.path().join("ssot_roundtrip.diycad");
        let mut project = sample_project();

        let material_id = deterministic_uuid("material", "roundtrip");
        let part_id = deterministic_uuid("part", "roundtrip");
        let ssot = SsotV1::new(
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
            vec![PartV1 {
                part_id,
                name: "root:ssot_roundtrip".to_string(),
                material_id,
                quantity: 1,
                manufacturing_outline_2d: None,
                thickness_mm: None,
                grain_direction: None,
                labels: Vec::new(),
                feature_ids: Vec::new(),
            }],
            FeatureGraphV1::empty(),
        )
        .canonicalize();

        project.ssot_v1 = Some(ssot.clone());

        save(&file_path, &project).expect("save should succeed");
        let loaded = load(&file_path).expect("load should succeed");

        assert_eq!(loaded.ssot_v1, Some(ssot));
    }

    #[test]
    fn legacy_load_derives_ssot_v1() {
        let dir = tempdir().expect("tempdir must be created");
        let file_path = dir.path().join("legacy_project.diycad");
        let project = create_empty_project("0.1.0", "mm", "2026-02-28T00:00:00Z"); // ssot_v1 stays None (legacy)

        save(&file_path, &project).expect("save should succeed");
        let loaded = load(&file_path).expect("load should succeed");

        let ssot = loaded.ssot_v1.expect("ssot should be derived");
        assert_eq!(ssot.ssot_version, SsotV1::VERSION);
        assert_eq!(ssot.materials.len(), 1);
        assert_eq!(ssot.parts.len(), 1);
        assert_eq!(ssot.parts[0].name, "root:legacy_project");
    }
}
