pub mod deps;
pub mod index;
pub mod reasons;
pub mod store;
pub mod tags;

use crate::index::{AssetKind, AssetMeta, LibraryIndex};
use crate::reasons::{LibraryReason, LibraryReasonCode};
use crate::tags::{load_tags_policy_from_repo_root, normalize_and_validate_tags, TagsPolicy};
use craftcad_presets::PresetsService;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

pub struct LibraryService {
    pub presets: PresetsService,
    pub tags_policy: TagsPolicy,
    repo_root: PathBuf,
}

#[derive(Debug, Deserialize)]
struct BuiltinBundle {
    materials: Vec<BuiltinItem>,
    processes: Vec<BuiltinItem>,
    outputs: Vec<BuiltinItem>,
    hardware: Vec<BuiltinItem>,
}

#[derive(Debug, Deserialize)]
struct BuiltinItem {
    id: String,
    version: String,
    display_name_key: Option<String>,
    tags: Vec<String>,
}

impl LibraryService {
    pub fn new(repo_root: PathBuf, user_root: PathBuf) -> Result<Self, LibraryReason> {
        let presets = PresetsService::new(repo_root.clone(), user_root).map_err(|e| {
            LibraryReason::new(
                LibraryReasonCode::LibIoError,
                format!("presets init failed: {e:?}"),
            )
        })?;
        let tags_policy = load_tags_policy_from_repo_root(Some(repo_root.clone()))?;
        Ok(Self {
            presets,
            tags_policy,
            repo_root,
        })
    }

    pub fn rebuild_builtin_index(
        &self,
        built_at_unix_ms: i64,
    ) -> Result<LibraryIndex, LibraryReason> {
        let bundle_path = self
            .repo_root
            .join("docs")
            .join("specs")
            .join("presets")
            .join("built_in_presets.json");
        let raw = fs::read_to_string(&bundle_path).map_err(|e| {
            LibraryReason::new(LibraryReasonCode::LibIoError, format!("read failed: {e}"))
                .with_path(bundle_path.display().to_string())
        })?;
        let b: BuiltinBundle = serde_json::from_str(&raw).map_err(|e| {
            LibraryReason::new(
                LibraryReasonCode::LibTemplateInvalid,
                format!("parse failed: {e}"),
            )
            .with_path(bundle_path.display().to_string())
        })?;

        let mut assets = vec![];

        for m in b.materials {
            let (tags, _) = normalize_and_validate_tags(&m.tags, &self.tags_policy)?;
            assets.push(AssetMeta {
                kind: AssetKind::PresetMaterial,
                id: m.id,
                version: m.version,
                display_name_key: m.display_name_key,
                tags,
                source: "builtin".to_string(),
            });
        }
        for p in b.processes {
            let (tags, _) = normalize_and_validate_tags(&p.tags, &self.tags_policy)?;
            assets.push(AssetMeta {
                kind: AssetKind::PresetProcess,
                id: p.id,
                version: p.version,
                display_name_key: p.display_name_key,
                tags,
                source: "builtin".to_string(),
            });
        }
        for o in b.outputs {
            let (tags, _) = normalize_and_validate_tags(&o.tags, &self.tags_policy)?;
            assets.push(AssetMeta {
                kind: AssetKind::PresetOutput,
                id: o.id,
                version: o.version,
                display_name_key: o.display_name_key,
                tags,
                source: "builtin".to_string(),
            });
        }
        for h in b.hardware {
            let (tags, _) = normalize_and_validate_tags(&h.tags, &self.tags_policy)?;
            assets.push(AssetMeta {
                kind: AssetKind::PresetHardware,
                id: h.id,
                version: h.version,
                display_name_key: h.display_name_key,
                tags,
                source: "builtin".to_string(),
            });
        }

        crate::index::rebuild_index(built_at_unix_ms, assets)
    }
}
