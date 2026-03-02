mod ref_resolver;

use drawing_model::{AnnotationType, DimensionType, DrawingDoc, PlacementSide};
use drawing_style::{
    annotation::{
        AnnotationKind as AKind, AnnotationPayload, ChamferInfo, ChamferType, HoleInfo, LeaderHint,
    },
    apply_bw_mode, apply_line_weight_scale, build_sheet_ir, compute_keepouts,
    compute_print_placement, load_bundle, measure_angle, measure_linear, measure_radius,
    place_annotation, place_dimension, render_svg, DimensionKind as DKind,
    DimensionOverrides as DOverrides, DrawingSsotBundle, Mm, PlacementHint as DPlacementHint,
    Primitive, ProjectMeta, Pt2, Rect, Side, SsotError, SsotPaths, StrokeStyle, StyledPrimitive,
    SvgError,
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

fn transform_xy(scale: f64, tr: (f64, f64), p: (f64, f64)) -> (f64, f64) {
    (p.0 * scale + tr.0, p.1 * scale + tr.1)
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
        let keepouts = compute_keepouts(&bundle);
        let resolver = SampleBoardResolver;
        let model_bbox = Rect {
            x: Mm(30.0),
            y: Mm(30.0),
            w: Mm(150.0),
            h: Mm(100.0),
        };
        let place = compute_print_placement(&bundle, model_bbox);

        let st = &bundle.style;
        let model_stroke = StrokeStyle {
            width_mm: st
                .line_weights
                .normal_mm
                .max(st.line_weights.min_line_weight_mm),
            color_hex: st.colors.default_stroke_hex.clone(),
            dash_pattern_mm: vec![],
        };

        let a = transform_xy(place.scale, place.translate_mm, (30.0, 30.0));
        let b = transform_xy(place.scale, place.translate_mm, (180.0, 30.0));
        let c = transform_xy(place.scale, place.translate_mm, (180.0, 130.0));
        let d = transform_xy(place.scale, place.translate_mm, (30.0, 130.0));
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
        let hc = transform_xy(place.scale, place.translate_mm, (90.0, 80.0));
        ir.push(StyledPrimitive {
            z: 11,
            stable_id: "MODEL_HOLE1".to_string(),
            stroke: Some(model_stroke.clone()),
            fill: None,
            text: None,
            prim: Primitive::Circle {
                c: pt(hc.0, hc.1),
                r: Mm(5.0 * place.scale),
            },
        });
        let c0 = transform_xy(place.scale, place.translate_mm, (30.0, 35.0));
        let c1 = transform_xy(place.scale, place.translate_mm, (35.0, 30.0));
        ir.push(StyledPrimitive {
            z: 12,
            stable_id: "MODEL_CHAMFER_A".to_string(),
            stroke: Some(model_stroke.clone()),
            fill: None,
            text: None,
            prim: Primitive::Line {
                a: pt(c0.0, c0.1),
                b: pt(c1.0, c1.1),
            },
        });

        if let Some(drw) = drawing {
            let mut used_bboxes: Vec<Rect> = vec![];

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
                        .map(|p| transform_xy(place.scale, place.translate_mm, (p.x, p.y))),
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
                            let sa = transform_xy(place.scale, place.translate_mm, sa);
                            let sb = transform_xy(place.scale, place.translate_mm, sb);
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
                            None
                        }
                    }
                    DimensionType::Angular => {
                        let pts: Vec<(f64, f64)> = resolved
                            .iter()
                            .filter_map(|g| match g {
                                ResolvedGeom::Point { p } => {
                                    Some(transform_xy(place.scale, place.translate_mm, *p))
                                }
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
                            let c = transform_xy(place.scale, place.translate_mm, c);
                            let on = (c.0 + r_mm * place.scale, c.1);
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
                    let placed =
                        place_dimension(&bundle, &de.id, &m, &hint, &ov, &keepouts, &used_bboxes);
                    if let Some(bb) = placed.text_bbox_mm {
                        used_bboxes.push(bb);
                    }
                    for mut item in placed.items {
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
                        ResolvedGeom::Point { p } => {
                            Some(transform_xy(place.scale, place.translate_mm, p))
                        }
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
                    .map(|p| transform_xy(place.scale, place.translate_mm, (p.x, p.y)));
                let payload = match ae.kind.ty {
                    AnnotationType::Text => AnnotationPayload::Text {
                        text: value
                            .get("text")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                    },
                    AnnotationType::Leader => AnnotationPayload::LeaderText {
                        text: value
                            .get("text")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        leader: LeaderHint {
                            default_angle_deg: value
                                .get("leader_default_angle_deg")
                                .and_then(Value::as_f64)
                                .unwrap_or(bundle.style.dimension.leader.default_angle_deg),
                            bend_mm: None,
                            text_pos_mm: text_pos,
                        },
                    },
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
                let (items, bbox) = place_annotation(
                    &bundle,
                    &format!("ANN_{}", ae.id),
                    kind,
                    anchor,
                    &payload,
                    &keepouts,
                    &used_bboxes,
                );
                if let Some(bb) = bbox {
                    used_bboxes.push(bb);
                }
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

        Ok(render_svg(&ir, bundle.print.export.svg_precision)?)
    }
}
