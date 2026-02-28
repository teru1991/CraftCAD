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

    Ok(DiycadProject {
        manifest,
        data,
        thumbnail_png,
    })
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
