use crate::model::*;
use regex::Regex;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DrawingValidateError {
    #[error("unsupported schema_version: {0}")]
    UnsupportedSchemaVersion(u32),

    #[error("invalid id format: {0}")]
    InvalidId(String),

    #[error("duplicate entity id: {0}")]
    DuplicateEntityId(String),

    #[error("ref_geometry must not be empty for entity: {0}")]
    EmptyRefGeometry(String),

    #[error("invalid *_vN preset id: {0}")]
    InvalidPresetId(String),

    #[error("invalid numeric value at {path}: {value}")]
    InvalidNumber { path: &'static str, value: f64 },

    #[error("invalid chamfer_type (must be C or R) for annotation: {0}")]
    InvalidChamferType(String),

    #[error("precision_override out of range (0..=6) for dimension: {0}")]
    InvalidPrecisionOverride(String),
}

fn is_valid_core_id(s: &str) -> bool {
    Regex::new(r"^[A-Za-z0-9_-]{6,64}$").unwrap().is_match(s)
}

fn is_valid_preset_id(s: &str) -> bool {
    Regex::new(r"^[a-z][a-z0-9_]*_v[0-9]+$")
        .unwrap()
        .is_match(s)
}

pub fn validate_drawing_doc(doc: &DrawingDoc) -> Result<(), DrawingValidateError> {
    if doc.schema_version != DrawingDoc::LATEST_SCHEMA_VERSION {
        return Err(DrawingValidateError::UnsupportedSchemaVersion(
            doc.schema_version,
        ));
    }

    if !is_valid_core_id(&doc.id) {
        return Err(DrawingValidateError::InvalidId(doc.id.clone()));
    }
    if !is_valid_preset_id(&doc.style_preset_id) {
        return Err(DrawingValidateError::InvalidPresetId(
            doc.style_preset_id.clone(),
        ));
    }
    if !is_valid_preset_id(&doc.sheet_template_id) {
        return Err(DrawingValidateError::InvalidPresetId(
            doc.sheet_template_id.clone(),
        ));
    }
    if !is_valid_preset_id(&doc.print_preset_id) {
        return Err(DrawingValidateError::InvalidPresetId(
            doc.print_preset_id.clone(),
        ));
    }

    let scale = doc.view.model_to_sheet.scale;
    if !scale.is_finite() || scale <= 0.0 || scale > 1000.0 {
        return Err(DrawingValidateError::InvalidNumber {
            path: "view.model_to_sheet.scale",
            value: scale,
        });
    }

    for (path, value) in [
        (
            "view.model_to_sheet.translate_mm.x",
            doc.view.model_to_sheet.translate_mm.x,
        ),
        (
            "view.model_to_sheet.translate_mm.y",
            doc.view.model_to_sheet.translate_mm.y,
        ),
    ] {
        if !value.is_finite() || value.abs() > 1_000_000.0 {
            return Err(DrawingValidateError::InvalidNumber { path, value });
        }
    }

    let mut ids = HashSet::<String>::new();
    for d in &doc.dimensions {
        if !is_valid_core_id(&d.id) {
            return Err(DrawingValidateError::InvalidId(d.id.clone()));
        }
        if !ids.insert(d.id.clone()) {
            return Err(DrawingValidateError::DuplicateEntityId(d.id.clone()));
        }
        if d.ref_geometry.is_empty() {
            return Err(DrawingValidateError::EmptyRefGeometry(d.id.clone()));
        }
        if let Some(p) = d.overrides.precision_override {
            if p > 6 {
                return Err(DrawingValidateError::InvalidPrecisionOverride(d.id.clone()));
            }
        }
        for r in &d.ref_geometry {
            if !is_valid_core_id(&r.stable_id) {
                return Err(DrawingValidateError::InvalidId(r.stable_id.clone()));
            }
        }
    }

    for a in &doc.annotations {
        if !is_valid_core_id(&a.id) {
            return Err(DrawingValidateError::InvalidId(a.id.clone()));
        }
        if !ids.insert(a.id.clone()) {
            return Err(DrawingValidateError::DuplicateEntityId(a.id.clone()));
        }
        if a.ref_geometry.is_empty() {
            return Err(DrawingValidateError::EmptyRefGeometry(a.id.clone()));
        }
        for r in &a.ref_geometry {
            if !is_valid_core_id(&r.stable_id) {
                return Err(DrawingValidateError::InvalidId(r.stable_id.clone()));
            }
        }

        if let AnnotationPayload::Chamfer {
            chamfer_type,
            chamfer_value_mm,
        } = &a.payload
        {
            if let Some(t) = chamfer_type {
                if t != "C" && t != "R" {
                    return Err(DrawingValidateError::InvalidChamferType(a.id.clone()));
                }
            }
            if let Some(v) = chamfer_value_mm {
                if !v.is_finite() || *v < 0.0 || *v > 1_000_000.0 {
                    return Err(DrawingValidateError::InvalidNumber {
                        path: "annotations[].payload.chamfer_value_mm",
                        value: *v,
                    });
                }
            }
        }
    }

    Ok(())
}
