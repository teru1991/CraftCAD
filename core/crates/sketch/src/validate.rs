use crate::model::Entity;
use crate::model::SketchDoc;
use craftcad_errors::{AppError, AppResult, ReasonCode, Severity};

pub struct ValidateLimits {
    pub max_entities: usize,
    pub max_polyline_points: usize,
    pub max_text_len: usize,
}

impl Default for ValidateLimits {
    fn default() -> Self {
        Self {
            max_entities: 200_000,
            max_polyline_points: 10_000,
            max_text_len: 4096,
        }
    }
}

pub fn validate(doc: &SketchDoc, limits: &ValidateLimits) -> AppResult<()> {
    if doc.entities.len() > limits.max_entities {
        return Err(AppError::new(
            ReasonCode::new("CAD_LIMIT_ENTITIES"),
            Severity::Error,
            "entity count exceeds limit",
        ));
    }
    for e in &doc.entities {
        match e {
            Entity::Circle(c) if c.r <= 0.0 => {
                return Err(AppError::new(
                    ReasonCode::new("CAD_INVALID_RADIUS"),
                    Severity::Error,
                    "radius must be positive",
                ));
            }
            Entity::Arc(a) if a.r <= 0.0 => {
                return Err(AppError::new(
                    ReasonCode::new("CAD_INVALID_RADIUS"),
                    Severity::Error,
                    "radius must be positive",
                ));
            }
            Entity::Polyline(p) if p.pts.len() > limits.max_polyline_points => {
                return Err(AppError::new(
                    ReasonCode::new("CAD_LIMIT_POLYLINE_POINTS"),
                    Severity::Error,
                    "polyline point limit exceeded",
                ));
            }
            Entity::Text(t) if t.text.len() > limits.max_text_len => {
                return Err(AppError::new(
                    ReasonCode::new("CAD_LIMIT_TEXT_LEN"),
                    Severity::Error,
                    "text length limit exceeded",
                ));
            }
            _ => {}
        }
    }
    Ok(())
}
