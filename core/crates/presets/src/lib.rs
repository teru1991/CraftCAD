use semver::Version;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub mod reasons;
pub mod salvage;

pub mod model {
    #[derive(Debug, Clone, Copy)]
    pub enum PresetKind {
        Material,
        Process,
        Output,
        Hardware,
    }
}

pub mod resolve {
    use super::model::PresetKind;
    use semver::VersionReq;

    #[derive(Debug, Clone)]
    pub struct PresetRef {
        pub kind: PresetKind,
        pub id: String,
        pub req: VersionReq,
    }

    impl PresetRef {
        pub fn parse(kind: PresetKind, id: String, req: &str) -> Result<Self, semver::Error> {
            Ok(Self {
                kind,
                id,
                req: VersionReq::parse(req)?,
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PresetsError {
    #[error("io: {0}")]
    Io(String),
    #[error("json: {0}")]
    Json(String),
    #[error("not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Deserialize)]
struct Bundle {
    materials: Vec<Item>,
    processes: Vec<Item>,
    outputs: Vec<Item>,
    hardware: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Item {
    id: String,
    version: String,
}

pub struct PresetsService {
    items: BTreeMap<(String, &'static str), Vec<Version>>,
    _repo_root: PathBuf,
    _user_root: PathBuf,
}

impl PresetsService {
    pub fn new(repo_root: PathBuf, user_root: PathBuf) -> Result<Self, PresetsError> {
        let path = repo_root.join("docs/specs/presets/built_in_presets.json");
        let s = fs::read_to_string(&path)
            .map_err(|e| PresetsError::Io(format!("{}: {}", path.display(), e)))?;
        let bundle: Bundle =
            serde_json::from_str(&s).map_err(|e| PresetsError::Json(e.to_string()))?;

        let mut items: BTreeMap<(String, &'static str), Vec<Version>> = BTreeMap::new();
        ingest(&mut items, "material", &bundle.materials)?;
        ingest(&mut items, "process", &bundle.processes)?;
        ingest(&mut items, "output", &bundle.outputs)?;
        ingest(&mut items, "hardware", &bundle.hardware)?;

        Ok(Self {
            items,
            _repo_root: repo_root,
            _user_root: user_root,
        })
    }

    pub fn resolve_ref_to_version(&self, r: &resolve::PresetRef) -> Result<String, PresetsError> {
        let kind = match r.kind {
            model::PresetKind::Material => "material",
            model::PresetKind::Process => "process",
            model::PresetKind::Output => "output",
            model::PresetKind::Hardware => "hardware",
        };
        let key = (r.id.clone(), kind);
        let list = self
            .items
            .get(&key)
            .ok_or_else(|| PresetsError::NotFound(format!("{}:{}", kind, r.id)))?;

        let mut matched: Vec<&Version> = list.iter().filter(|v| r.req.matches(v)).collect();
        matched.sort();
        matched
            .last()
            .map(|v| v.to_string())
            .ok_or_else(|| PresetsError::NotFound(format!("{}:{}@{}", kind, r.id, r.req)))
    }
}

fn ingest(
    map: &mut BTreeMap<(String, &'static str), Vec<Version>>,
    kind: &'static str,
    items: &[Item],
) -> Result<(), PresetsError> {
    for i in items {
        let v = Version::parse(&i.version)
            .map_err(|e| PresetsError::Json(format!("invalid semver {}: {}", i.version, e)))?;
        map.entry((i.id.clone(), kind)).or_default().push(v);
    }
    Ok(())
}

pub fn repo_root_from_manifest(start: &Path) -> Option<PathBuf> {
    for up in 0..=10usize {
        let mut p = start.to_path_buf();
        for _ in 0..up {
            p = p.parent().unwrap_or(&p).to_path_buf();
        }
        if p.join("docs").join("specs").exists() {
            return Some(p);
        }
    }
    None
}
