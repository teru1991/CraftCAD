use craftcad_io::model::*;
use craftcad_io::options::ExportOptions;
use craftcad_io::reasons::{AppError, AppResult};
use craftcad_io::report::IoReport;
use serde_json::{json, Map, Value};

fn units_to_str(u: Units) -> &'static str {
    match u {
        Units::Inch => "inch",
        Units::Mm => "mm",
    }
}

fn fixed_dec(v: f64, places: usize) -> Value {
    let p = 10f64.powi(places as i32);
    let r = (v * p).round() / p;
    Value::from(r)
}

fn point(p: Point2D, places: usize) -> Value {
    json!({ "x": fixed_dec(p.x, places), "y": fixed_dec(p.y, places) })
}

fn color_policy(c: &ColorPolicy) -> Value {
    match c {
        ColorPolicy::ByLayer => Value::String("by_layer".to_string()),
        ColorPolicy::FixedRgb { r, g, b } => json!({ "fixed_rgb": { "r": *r, "g": *g, "b": *b } }),
    }
}

fn segment(s: &Segment2D, places: usize) -> Value {
    match s {
        Segment2D::Line { a, b } => {
            json!({ "kind": "line", "a": point(*a, places), "b": point(*b, places) })
        }
        Segment2D::Arc {
            center,
            radius,
            start_rad,
            end_rad,
            ccw,
        } => json!({
            "kind": "arc",
            "center": point(*center, places),
            "radius": fixed_dec(*radius, places),
            "start_rad": fixed_dec(*start_rad, places),
            "end_rad": fixed_dec(*end_rad, places),
            "ccw": *ccw
        }),
        Segment2D::Circle { center, radius } => json!({
            "kind": "circle",
            "center": point(*center, places),
            "radius": fixed_dec(*radius, places)
        }),
        Segment2D::CubicBezier { a, c1, c2, b } => json!({
            "kind": "cubic_bezier",
            "a": point(*a, places),
            "c1": point(*c1, places),
            "c2": point(*c2, places),
            "b": point(*b, places)
        }),
    }
}

fn entity(e: &Entity, places: usize) -> Value {
    match e {
        Entity::Path(p) => {
            let segs: Vec<Value> = p.segments.iter().map(|s| segment(s, places)).collect();
            json!({
                "type": "path",
                "id": p.id,
                "stroke": {
                    "layer": p.stroke.layer,
                    "linetype": p.stroke.linetype,
                    "weight": fixed_dec(p.stroke.weight as f64, places),
                    "color_policy": color_policy(&p.stroke.color_policy)
                },
                "closed": p.closed,
                "segments": segs
            })
        }
        Entity::Text(t) => json!({
            "type": "text",
            "id": t.id,
            "layer": t.layer,
            "pos": point(t.pos, places),
            "text": t.text,
            "size": fixed_dec(t.size as f64, places),
            "font_hint": t.font_hint,
            "rotation_rad": fixed_dec(t.rotation_rad, places)
        }),
    }
}

fn stable_pretty_json(v: &Value) -> Vec<u8> {
    serde_json::to_vec_pretty(v).expect("json serialize should not fail")
}

pub fn export_json(
    model: &InternalModel,
    opts: &ExportOptions,
) -> AppResult<(Vec<u8>, Vec<AppError>, IoReport)> {
    let warnings: Vec<AppError> = Vec::new();
    let mut report = IoReport::new("json");
    let places = 4usize;

    let entities: Vec<Value> = model.entities.iter().map(|e| entity(e, places)).collect();

    let mut meta = Map::<String, Value>::new();
    meta.insert("source_format".into(), Value::String("json".into()));
    meta.insert(
        "unit_guess".into(),
        match &model.metadata.unit_guess {
            Some(s) => Value::String(s.clone()),
            None => Value::Null,
        },
    );
    meta.insert(
        "determinism_tag".into(),
        Value::String(opts.determinism_tag()),
    );

    let mut root = Map::<String, Value>::new();
    root.insert("schema_version".into(), Value::from(1));
    root.insert(
        "units".into(),
        Value::String(units_to_str(model.units).into()),
    );
    root.insert("metadata".into(), Value::Object(meta));
    root.insert("entities".into(), Value::Array(entities));

    report.entities_in = model.entities.len();
    report.texts_in = model.texts.len();
    report.entities_out = model.entities.len();
    report.texts_out = model.texts.len();
    report.determinism_tag = opts.determinism_tag();

    Ok((stable_pretty_json(&Value::Object(root)), warnings, report))
}
