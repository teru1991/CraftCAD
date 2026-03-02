use super::types::*;
use std::f64::consts::PI;

#[derive(Debug, thiserror::Error)]
pub enum MeasureError {
    #[error("invalid input (NaN/Inf)")]
    InvalidNumber,
    #[error("degenerate geometry (zero length)")]
    Degenerate,
}

fn finite(v: f64) -> Result<f64, MeasureError> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(MeasureError::InvalidNumber)
    }
}

pub fn measure_linear(
    p0: (f64, f64),
    p1: (f64, f64),
    kind: DimensionKind,
) -> Result<MeasuredDimension, MeasureError> {
    let dx = finite(p1.0 - p0.0)?;
    let dy = finite(p1.1 - p0.1)?;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= 1e-9 {
        return Err(MeasureError::Degenerate);
    }
    Ok(MeasuredDimension {
        kind,
        value_mm: len,
        value_deg: None,
        anchor_points_mm: vec![p0, p1],
        radius_mm: None,
    })
}

pub fn measure_radius(
    center: (f64, f64),
    on_circle: (f64, f64),
    kind: DimensionKind,
) -> Result<MeasuredDimension, MeasureError> {
    let dx = finite(on_circle.0 - center.0)?;
    let dy = finite(on_circle.1 - center.1)?;
    let r = (dx * dx + dy * dy).sqrt();
    if r <= 1e-9 {
        return Err(MeasureError::Degenerate);
    }
    Ok(MeasuredDimension {
        kind,
        value_mm: if kind == DimensionKind::Diameter {
            r * 2.0
        } else {
            r
        },
        value_deg: None,
        anchor_points_mm: vec![center, on_circle],
        radius_mm: Some(r),
    })
}

pub fn measure_angle(
    p0: (f64, f64),
    vertex: (f64, f64),
    p2: (f64, f64),
) -> Result<MeasuredDimension, MeasureError> {
    let v1 = (finite(p0.0 - vertex.0)?, finite(p0.1 - vertex.1)?);
    let v2 = (finite(p2.0 - vertex.0)?, finite(p2.1 - vertex.1)?);
    let n1 = (v1.0 * v1.0 + v1.1 * v1.1).sqrt();
    let n2 = (v2.0 * v2.0 + v2.1 * v2.1).sqrt();
    if n1 <= 1e-9 || n2 <= 1e-9 {
        return Err(MeasureError::Degenerate);
    }
    let dot = (v1.0 * v2.0 + v1.1 * v2.1) / (n1 * n2);
    let deg = dot.clamp(-1.0, 1.0).acos() * 180.0 / PI;
    Ok(MeasuredDimension {
        kind: DimensionKind::Angular,
        value_mm: 0.0,
        value_deg: Some(deg),
        anchor_points_mm: vec![p0, vertex, p2],
        radius_mm: None,
    })
}
