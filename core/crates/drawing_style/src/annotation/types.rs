use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnnotationKind {
    Text,
    Leader,
    HoleCallout,
    ChamferCallout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaderHint {
    pub default_angle_deg: f64,
    pub bend_mm: Option<(f64, f64)>,
    pub text_pos_mm: Option<(f64, f64)>,
}

impl Default for LeaderHint {
    fn default() -> Self {
        Self {
            default_angle_deg: 30.0,
            bend_mm: None,
            text_pos_mm: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HoleInfo {
    pub diameter_mm: f64,
    pub depth_mm: Option<f64>,
    pub count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChamferType {
    C,
    R,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChamferInfo {
    pub ty: ChamferType,
    pub value_mm: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnnotationPayload {
    Text {
        text: String,
    },
    LeaderText {
        text: String,
        leader: LeaderHint,
    },
    Hole {
        info: HoleInfo,
        leader: LeaderHint,
    },
    Chamfer {
        info: ChamferInfo,
        leader: LeaderHint,
    },
}
