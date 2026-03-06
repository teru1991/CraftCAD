use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// SSOT snapshot embedded in the project file.
/// v1 is additive-only; removals are forbidden (use deprecation).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SsotV1 {
    pub ssot_version: u32, // must be 1
    pub materials: Vec<MaterialV1>,
    pub parts: Vec<PartV1>,
    pub feature_graph: FeatureGraphV1,
}

impl SsotV1 {
    pub const VERSION: u32 = 1;

    pub fn new(
        materials: Vec<MaterialV1>,
        parts: Vec<PartV1>,
        feature_graph: FeatureGraphV1,
    ) -> Self {
        Self {
            ssot_version: Self::VERSION,
            materials,
            parts,
            feature_graph,
        }
    }

    /// Returns a deterministic canonicalized copy (stable ordering).
    pub fn canonicalize(mut self) -> Self {
        self.materials.sort_by_key(|m| m.material_id);
        self.parts.sort_by_key(|p| p.part_id);
        self.feature_graph = self.feature_graph.canonicalize();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaterialV1 {
    pub material_id: Uuid,
    pub category: MaterialCategoryV1,
    pub name: String,
    pub thickness_mm: Option<f64>,
    pub grain_policy: GrainPolicyV1,
    pub kerf_mm: f64,
    pub margin_mm: f64,
    pub estimate_loss_factor: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MaterialCategoryV1 {
    Wood,
    Plywood,
    Mdf,
    Leather,
    Hardware,
    Unspecified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GrainPolicyV1 {
    None,
    AlongX,
    AlongY,
    Fixed, // fixed axis in world/sketch coordinates (v1 does not model axis; policy only)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PartV1 {
    pub part_id: Uuid,
    pub name: String,
    pub material_id: Uuid,
    pub quantity: u32,
    /// For Step1, manufacturing outline can be absent; later phases populate it.
    pub manufacturing_outline_2d: Option<ManufacturingOutline2dV1>,
    pub thickness_mm: Option<f64>, // snapshot for stability even if material changes
    pub grain_direction: Option<GrainDirectionV1>,
    pub labels: Vec<PartLabelV1>,
    pub feature_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManufacturingOutline2dV1 {
    /// v1: axis-aligned bbox (min/max) as the minimal stable representation.
    /// Later versions can add polygon/path without breaking this contract.
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GrainDirectionV1 {
    AlongX,
    AlongY,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PartLabelV1 {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeatureGraphV1 {
    /// v1: a deterministic ordered list (history). Tree/branching can be layered later.
    pub features: Vec<FeatureNodeV1>,
}

impl FeatureGraphV1 {
    pub fn empty() -> Self {
        Self {
            features: Vec::new(),
        }
    }

    pub fn canonicalize(self) -> Self {
        // Determinism rule: keep insertion order; but ensure feature_ids are unique.
        // Do not reorder to avoid changing semantics.
        // Future: validate uniqueness elsewhere.
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeatureNodeV1 {
    pub feature_id: Uuid,
    pub feature_type: FeatureTypeV1,
    pub params: serde_json::Value, // typed in later phases; v1 stores generic JSON
    pub targets: Vec<FeatureTargetV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeatureTypeV1 {
    ExtrudeAdd,
    ExtrudeCut,
    Hole,
    Pattern,
    Chamfer,
    ScrewFeature,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeatureTargetV1 {
    /// v1: can target a part (by ID). Later: sketches/edges/faces with stable refs.
    pub part_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct SsotDeriveConfig {
    pub default_kerf_mm: f64,
    pub default_margin_mm: f64,
}

impl Default for SsotDeriveConfig {
    fn default() -> Self {
        Self {
            default_kerf_mm: 2.0,
            default_margin_mm: 5.0,
        }
    }
}

pub fn deterministic_uuid(tag: &str, key: &str) -> Uuid {
    let mut hasher = Sha256::new();
    hasher.update(tag.as_bytes());
    hasher.update([0x1f]);
    hasher.update(key.as_bytes());
    let hash = hasher.finalize();

    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

/// Derive a minimal SSOT from legacy project content.
/// This must be deterministic: stable ordering, stable naming.
pub fn derive_minimal_ssot_v1(project_name: &str, cfg: SsotDeriveConfig) -> SsotV1 {
    let stable_key = project_name.trim();
    let material_id = deterministic_uuid("material", stable_key);
    let part_id = deterministic_uuid("part", stable_key);

    let mat = MaterialV1 {
        material_id,
        category: MaterialCategoryV1::Unspecified,
        name: "unspecified".to_string(),
        thickness_mm: None,
        grain_policy: GrainPolicyV1::None,
        kerf_mm: cfg.default_kerf_mm,
        margin_mm: cfg.default_margin_mm,
        estimate_loss_factor: None,
    };

    let part = PartV1 {
        part_id,
        name: if stable_key.is_empty() {
            "root".to_string()
        } else {
            format!("root:{}", stable_key)
        },
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
    };

    SsotV1::new(vec![mat], vec![part], FeatureGraphV1::empty()).canonicalize()
}
