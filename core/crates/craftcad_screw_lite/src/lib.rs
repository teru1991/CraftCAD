use craftcad_ssot::{FeatureTypeV1, SsotV1};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScrewPoint {
    pub part_id: Uuid,
    pub feature_id: Uuid,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum ScrewEvalError {
    #[error("screw feature params missing points: feature_id={feature_id}")]
    MissingPoints { feature_id: Uuid },
    #[error("screw feature point parse failed: feature_id={feature_id}")]
    InvalidPoint { feature_id: Uuid },
}

fn parse_points(
    feature_id: Uuid,
    params: &serde_json::Value,
) -> Result<Vec<(f64, f64)>, ScrewEvalError> {
    let points = params
        .get("points")
        .and_then(|v| v.as_array())
        .ok_or(ScrewEvalError::MissingPoints { feature_id })?;

    let mut out = Vec::with_capacity(points.len());
    for p in points {
        let x = p.get("x").and_then(|v| v.as_f64());
        let y = p.get("y").and_then(|v| v.as_f64());
        match (x, y) {
            (Some(x), Some(y)) => out.push((x, y)),
            _ => return Err(ScrewEvalError::InvalidPoint { feature_id }),
        }
    }
    Ok(out)
}

pub fn eval_screw_points(ssot: &SsotV1) -> Result<Vec<ScrewPoint>, ScrewEvalError> {
    let mut points = Vec::new();
    for feature in &ssot.feature_graph.features {
        if feature.feature_type != FeatureTypeV1::ScrewFeature {
            continue;
        }
        let parsed = parse_points(feature.feature_id, &feature.params)?;
        for target in &feature.targets {
            for (x, y) in parsed.iter().copied() {
                points.push(ScrewPoint {
                    part_id: target.part_id,
                    feature_id: feature.feature_id,
                    x,
                    y,
                });
            }
        }
    }

    points.sort_by(|a, b| {
        (a.part_id, a.feature_id, a.x.to_bits(), a.y.to_bits()).cmp(&(
            b.part_id,
            b.feature_id,
            b.x.to_bits(),
            b.y.to_bits(),
        ))
    });
    Ok(points)
}
