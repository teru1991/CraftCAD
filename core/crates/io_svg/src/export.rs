use craftcad_io::model::*;
use craftcad_io::options::ExportOptions;
use craftcad_io::reasons::{AppError, AppResult};
use craftcad_io::report::IoReport;
use craftcad_io_support::{SupportLevel, SupportMatrix};

fn fmt_f(v: f64, places: usize) -> String {
    format!("{:.*}", places, v)
}
fn seg_to_path_cmd(s: &Segment2D, places: usize) -> Option<String> {
    match s {
        Segment2D::Line { a, b } => Some(format!(
            "M {} {} L {} {}",
            fmt_f(a.x, places),
            fmt_f(a.y, places),
            fmt_f(b.x, places),
            fmt_f(b.y, places)
        )),
        Segment2D::CubicBezier { a, c1, c2, b } => Some(format!(
            "M {} {} C {} {}, {} {}, {} {}",
            fmt_f(a.x, places),
            fmt_f(a.y, places),
            fmt_f(c1.x, places),
            fmt_f(c1.y, places),
            fmt_f(c2.x, places),
            fmt_f(c2.y, places),
            fmt_f(b.x, places),
            fmt_f(b.y, places)
        )),
        _ => None,
    }
}

pub fn export_svg(
    model: &InternalModel,
    opts: &ExportOptions,
) -> AppResult<(Vec<u8>, Vec<AppError>, IoReport)> {
    let sm = SupportMatrix::load_from_ssot()?;
    let mut warnings = Vec::new();
    let mut report = IoReport::new("svg");
    let places = 4usize;
    let mut body = String::new();

    for e in &model.entities {
        match e {
            Entity::Path(p) => {
                if sm.level("svg", "entity_path", "export") == SupportLevel::NotSupported {
                    continue;
                }
                let mut parts = Vec::new();
                for s in &p.segments {
                    if let Some(c) = seg_to_path_cmd(s, places) {
                        parts.push(c);
                    } else {
                        for r in sm.reasons("svg", "entity_path_unhandled_segment", "export") {
                            warnings.push(
                                AppError::new(
                                    r,
                                    "unhandled segment best-effort; recommend enable_approx",
                                )
                                .with_context("path_id", p.id.clone()),
                            );
                        }
                    }
                }
                if !parts.is_empty() {
                    body.push_str(&format!(
                        "<path id=\"{}\" d=\"{}\" />\n",
                        p.id,
                        parts.join(" ")
                    ));
                }
            }
            Entity::Text(t) => {
                let lvl = sm.level("svg", "entity_text", "export");
                if lvl == SupportLevel::NotSupported {
                    continue;
                }
                body.push_str(&format!(
                    "<text id=\"{}\" x=\"{}\" y=\"{}\">{}</text>\n",
                    t.id,
                    fmt_f(t.pos.x, places),
                    fmt_f(t.pos.y, places),
                    t.text
                ));
                if lvl == SupportLevel::BestEffort {
                    for r in sm.reasons("svg", "entity_text", "export") {
                        warnings.push(AppError::new(
                            r,
                            "text exported best-effort (no font embedding)",
                        ));
                    }
                }
            }
        }
    }

    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\">\n{}\n</svg>",
        body
    );
    report.entities_in = model.entities.len();
    report.entities_out = model.entities.len();
    report.determinism_tag = opts.determinism_tag();
    Ok((svg.into_bytes(), warnings, report))
}
