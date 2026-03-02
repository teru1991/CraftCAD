mod ref_resolver;

use drawing_model::{AnnotationType, DimensionType, DrawingDoc, PlacementSide};
use drawing_style::{
    annotation::{
        AnnotationKind as AKind, AnnotationPayload, ChamferInfo, ChamferType, HoleInfo, LeaderHint,
    },
    apply_bw_mode, apply_line_weight_scale, build_sheet_ir, load_bundle, measure_angle,
    measure_linear, measure_radius, place_annotation, place_dimension, render_svg,
    DimensionKind as DKind, DimensionOverrides as DOverrides, DrawingSsotBundle, Mm,
    PlacementHint as DPlacementHint, Primitive, ProjectMeta, Pt2, Side, SsotError, SsotPaths,
    StrokeStyle, StyledPrimitive, SvgError,
};
use serde_json::Value;
use std::path::Path;
use thiserror::Error;

pub use ref_resolver::*;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("ssot error: {0}")]
    Ssot(#[from] SsotError),
    #[error("svg error: {0}")]
    Svg(#[from] SvgError),
}

#[derive(Debug, Clone)]
pub struct ExportRequest {
    pub style_preset_id: String,
    pub sheet_template_id: String,
    pub print_preset_id: String,
    pub meta: ProjectMeta,
}

pub struct DrawingExporter;

fn pt(x: f64, y: f64) -> Pt2 {
    Pt2 { x: Mm(x), y: Mm(y) }
}

impl DrawingExporter {
    pub fn export_svg(
        repo_root: &Path,
        drawing: Option<&DrawingDoc>,
        req: &ExportRequest,
    ) -> Result<String, ExportError> {
        let paths = SsotPaths::from_repo_root(repo_root);
        let bundle: DrawingSsotBundle = load_bundle(
            &paths,
            &req.style_preset_id,
            &req.sheet_template_id,
            &req.print_preset_id,
        )?;

        let mut ir = build_sheet_ir(&bundle, &req.meta);
        let resolver = SampleBoardResolver;
        let st = &bundle.style;
        let model_stroke = StrokeStyle {
            width_mm: st
                .line_weights
                .normal_mm
                .max(st.line_weights.min_line_weight_mm),
            color_hex: st.colors.default_stroke_hex.clone(),
            dash_pattern_mm: vec![],
        };

        let a = (30.0, 30.0);
        let b = (180.0, 30.0);
        let c = (180.0, 130.0);
        let d = (30.0, 130.0);
        ir.push(StyledPrimitive {
            z: 10,
            stable_id: "MODEL_BOARD".to_string(),
            stroke: Some(model_stroke.clone()),
            fill: None,
            text: None,
            prim: Primitive::Polyline {
                pts: vec![pt(a.0, a.1), pt(b.0, b.1), pt(c.0, c.1), pt(d.0, d.1)],
                closed: true,
            },
        });
        ir.push(StyledPrimitive {
            z: 11,
            stable_id: "MODEL_HOLE1".to_string(),
            stroke: Some(model_stroke.clone()),
            fill: None,
            text: None,
            prim: Primitive::Circle {
                c: pt(90.0, 80.0),
                r: Mm(5.0),
            },
        });
        ir.push(StyledPrimitive {
            z: 12,
            stable_id: "MODEL_CHAMFER_A".to_string(),
            stroke: Some(model_stroke.clone()),
            fill: None,
            text: None,
            prim: Primitive::Line {
                a: pt(30.0, 35.0),
                b: pt(35.0, 30.0),
            },
        });

        if let Some(drw) = drawing {
            for de in &drw.dimensions {
                let hint = DPlacementHint {
                    side: match de.placement_hint.side {
                        PlacementSide::Left => Side::Left,
                        PlacementSide::Right => Side::Right,
                        PlacementSide::Top => Side::Top,
                        PlacementSide::Bottom => Side::Bottom,
                        PlacementSide::Auto => Side::Auto,
                    },
                    offset_level: de.placement_hint.offset_level,
                    manual_text_pos_mm: de
                        .placement_hint
                        .manual_text_pos_mm
                        .as_ref()
                        .map(|p| (p.x, p.y)),
                };
                let ov = DOverrides {
                    text_override: de.overrides.text_override.clone(),
                    precision_override: de.overrides.precision_override,
                };
                let mut refs = de.ref_geometry.clone();
                refs.sort_by(|l, r| l.stable_id.cmp(&r.stable_id));
                let resolved: Vec<_> = refs.iter().filter_map(|r| resolver.resolve(r)).collect();
                let measured = match de.kind.ty {
                    DimensionType::LinearSerial | DimensionType::LinearBaseline => {
                        if let Some((sa, sb)) = resolved.iter().find_map(|g| {
                            if let ResolvedGeom::Segment { a, b } = g {
                                Some((*a, *b))
                            } else {
                                None
                            }
                        }) {
                            measure_linear(
                                sa,
                                sb,
                                if de.kind.ty == DimensionType::LinearBaseline {
                                    DKind::LinearBaseline
                                } else {
                                    DKind::LinearSerial
                                },
                            )
                            .ok()
                        } else {
                            let pts: Vec<(f64, f64)> = resolved
                                .iter()
                                .filter_map(|g| match g {
                                    ResolvedGeom::Point { p } => Some(*p),
                                    _ => None,
                                })
                                .collect();
                            if pts.len() >= 2 {
                                measure_linear(
                                    pts[0],
                                    pts[1],
                                    if de.kind.ty == DimensionType::LinearBaseline {
                                        DKind::LinearBaseline
                                    } else {
                                        DKind::LinearSerial
                                    },
                                )
                                .ok()
                            } else {
                                None
                            }
                        }
                    }
                    DimensionType::Angular => {
                        let pts: Vec<(f64, f64)> = resolved
                            .iter()
                            .filter_map(|g| match g {
                                ResolvedGeom::Point { p } => Some(*p),
                                _ => None,
                            })
                            .collect();
                        if pts.len() >= 3 {
                            measure_angle(pts[0], pts[1], pts[2]).ok()
                        } else {
                            None
                        }
                    }
                    DimensionType::Radius | DimensionType::Diameter => {
                        if let Some((c, r_mm)) = resolved.iter().find_map(|g| match g {
                            ResolvedGeom::Circle { c, r_mm } => Some((*c, *r_mm)),
                            _ => None,
                        }) {
                            let on = (c.0 + r_mm, c.1);
                            measure_radius(
                                c,
                                on,
                                if de.kind.ty == DimensionType::Diameter {
                                    DKind::Diameter
                                } else {
                                    DKind::Radius
                                },
                            )
                            .ok()
                        } else {
                            None
                        }
                    }
                };
                if let Some(m) = measured {
                    for mut item in place_dimension(&bundle, &m, &hint, &ov).items {
                        item.stable_id = format!("DIM_{}_{}", de.id, item.stable_id);
                        ir.push(item);
                    }
                }
            }

            for ae in &drw.annotations {
                let anchor = ae
                    .ref_geometry
                    .iter()
                    .filter_map(|r| resolver.resolve(r))
                    .find_map(|g| match g {
                        ResolvedGeom::Point { p } => Some(p),
                        _ => None,
                    });
                let Some(anchor) = anchor else {
                    continue;
                };

                let value = serde_json::to_value(&ae.payload).unwrap_or(Value::Null);
                let text_pos = ae
                    .placement_hint
                    .manual_text_pos_mm
                    .as_ref()
                    .map(|p| (p.x, p.y));
                let payload = match ae.kind.ty {
                    AnnotationType::Text => AnnotationPayload::Text {
                        text: value
                            .get("text")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                    },
                    AnnotationType::Leader => {
                        let text = value
                            .get("text")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string();
                        let ang = value
                            .get("leader_default_angle_deg")
                            .and_then(Value::as_f64)
                            .unwrap_or(bundle.style.dimension.leader.default_angle_deg);
                        AnnotationPayload::LeaderText {
                            text,
                            leader: LeaderHint {
                                default_angle_deg: ang,
                                bend_mm: None,
                                text_pos_mm: text_pos,
                            },
                        }
                    }
                    AnnotationType::HoleCallout => AnnotationPayload::Hole {
                        info: HoleInfo {
                            diameter_mm: value
                                .get("hole_diameter_mm")
                                .and_then(Value::as_f64)
                                .unwrap_or(0.0),
                            depth_mm: value.get("hole_depth_mm").and_then(Value::as_f64),
                            count: value
                                .get("hole_count")
                                .and_then(Value::as_u64)
                                .map(|v| v as u32),
                        },
                        leader: LeaderHint {
                            default_angle_deg: bundle.style.dimension.leader.default_angle_deg,
                            bend_mm: None,
                            text_pos_mm: text_pos,
                        },
                    },
                    AnnotationType::ChamferCallout => AnnotationPayload::Chamfer {
                        info: ChamferInfo {
                            ty: if value.get("chamfer_type").and_then(Value::as_str) == Some("R") {
                                ChamferType::R
                            } else {
                                ChamferType::C
                            },
                            value_mm: value
                                .get("chamfer_value_mm")
                                .and_then(Value::as_f64)
                                .unwrap_or(0.0),
                        },
                        leader: LeaderHint {
                            default_angle_deg: bundle.style.dimension.leader.default_angle_deg,
                            bend_mm: None,
                            text_pos_mm: text_pos,
                        },
                    },
                };

                let kind = match ae.kind.ty {
                    AnnotationType::Text => AKind::Text,
                    AnnotationType::Leader => AKind::Leader,
                    AnnotationType::HoleCallout => AKind::HoleCallout,
                    AnnotationType::ChamferCallout => AKind::ChamferCallout,
                };
                let (items, _) =
                    place_annotation(&bundle, &format!("ANN_{}", ae.id), kind, anchor, &payload);
                for item in items {
                    ir.push(item);
                }
            }
        }

        ir.sort_stable();
        if bundle.print.color_mode == "bw" {
            apply_bw_mode(&mut ir);
        }
        apply_line_weight_scale(
            &mut ir,
            bundle.print.line_weight_scale,
            bundle.style.line_weights.min_line_weight_mm,
        );

        let svg = render_svg(&ir, bundle.print.export.svg_precision)?;
        Ok(svg)
    }
}
