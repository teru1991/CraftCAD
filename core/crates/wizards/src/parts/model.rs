use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PartsDraft {
    pub schema_version: i32,
    pub parts: Vec<PartDraft>,
    pub annotations: Vec<AnnotationDraft>,
    pub recommended_nest_job: Option<NestJobDraft>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PartDraft {
    pub part_id: String,
    pub name: String,
    pub outline: Outline2D,
    pub features: Vec<Feature2D>,
    pub qty: i32,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Outline2D {
    Rect { w_mm: f64, h_mm: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Feature2D {
    HoleCircle {
        cx_mm: f64,
        cy_mm: f64,
        diameter_mm: f64,
    },
    StitchHole {
        x_mm: f64,
        y_mm: f64,
        diameter_mm: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AnnotationDraft {
    HoleCallout {
        part_id: String,
        diameter_mm: f64,
        count: i32,
    },
    StitchPitch {
        part_id: String,
        pitch_mm: f64,
    },
    SeamAllowance {
        part_id: String,
        allowance_mm: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NestJobDraft {
    pub material_preset_id: String,
    pub process_preset_id: String,
    pub kerf_mm: f64,
    pub margin_mm: f64,
    pub seed: u64,
    pub allow_rotate: bool,
    pub grain: String,
}
