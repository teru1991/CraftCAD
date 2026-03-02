use craftcad_io::model::{Entity, Segment2D};
use craftcad_io::options::{ExportOptions, ImportOptions};
use craftcad_io_bridge::default_engine;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}
fn read_bytes(path: &PathBuf) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}
fn read_string(path: &PathBuf) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}
fn write_string(path: &PathBuf, s: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| panic!("mkdir: {}", e));
    }
    fs::write(path, s).unwrap_or_else(|e| panic!("write {}: {}", path.display(), e));
}
fn accept_enabled() -> bool {
    std::env::var("GOLDEN_ACCEPT").ok().as_deref() == Some("1")
}
fn assert_or_accept(path: &PathBuf, got: &str) {
    if accept_enabled() {
        write_string(path, got);
        return;
    }
    let exp = read_string(path);
    assert_eq!(exp, got, "golden mismatch: {}", path.display());
}

fn bbox_of_model(model: &craftcad_io::model::InternalModel) -> (f64, f64, f64, f64) {
    let mut minx = f64::INFINITY;
    let mut miny = f64::INFINITY;
    let mut maxx = f64::NEG_INFINITY;
    let mut maxy = f64::NEG_INFINITY;

    for e in &model.entities {
        match e {
            Entity::Path(p) => {
                for s in &p.segments {
                    match s {
                        Segment2D::Line { a, b } => {
                            for pt in [*a, *b] {
                                minx = minx.min(pt.x);
                                miny = miny.min(pt.y);
                                maxx = maxx.max(pt.x);
                                maxy = maxy.max(pt.y);
                            }
                        }
                        Segment2D::CubicBezier { a, c1, c2, b } => {
                            for pt in [*a, *c1, *c2, *b] {
                                minx = minx.min(pt.x);
                                miny = miny.min(pt.y);
                                maxx = maxx.max(pt.x);
                                maxy = maxy.max(pt.y);
                            }
                        }
                        Segment2D::Arc { center, radius, .. }
                        | Segment2D::Circle { center, radius } => {
                            minx = minx.min(center.x - radius);
                            miny = miny.min(center.y - radius);
                            maxx = maxx.max(center.x + radius);
                            maxy = maxy.max(center.y + radius);
                        }
                    }
                }
            }
            Entity::Text(t) => {
                minx = minx.min(t.pos.x);
                miny = miny.min(t.pos.y);
                maxx = maxx.max(t.pos.x);
                maxy = maxy.max(t.pos.y);
            }
        }
    }

    if !minx.is_finite() {
        (0.0, 0.0, 0.0, 0.0)
    } else {
        (minx, miny, maxx, maxy)
    }
}

#[test]
fn compat_report_golden() {
    let root = repo_root();
    let input = root.join("tests/golden/io_roundtrip/inputs/compat/compat_01.json");
    let exp_report = root.join("tests/golden/io_roundtrip/expected/compat/compat_report.json");

    let eng = default_engine();
    let iopts = ImportOptions::default_for_tests();
    let eopts = ExportOptions::default_for_tests();

    let json0 = eng.import("json", &read_bytes(&input), &iopts).unwrap();

    let dxf = eng.export("dxf", &json0.model, &eopts).unwrap();
    let json1 = eng.import("dxf", &dxf.bytes, &iopts).unwrap();
    let json1_out = eng.export("json", &json1.model, &eopts).unwrap();

    let svg = eng.export("svg", &json0.model, &eopts).unwrap();
    let json2 = eng.import("svg", &svg.bytes, &iopts).unwrap();
    let json2_out = eng.export("json", &json2.model, &eopts).unwrap();

    let dxf2 = eng.export("dxf", &json0.model, &eopts).unwrap();
    assert_eq!(dxf.bytes, dxf2.bytes, "DXF export must be deterministic");
    let svg2 = eng.export("svg", &json0.model, &eopts).unwrap();
    assert_eq!(svg.bytes, svg2.bytes, "SVG export must be deterministic");

    let eps = (iopts.determinism.close_eps * 5.0)
        .max(iopts.determinism.approx_eps * 100.0)
        .max(10.0);
    let (a0, b0, c0, d0) = bbox_of_model(&json0.model);
    let (a1, b1, c1, d1) = bbox_of_model(&json1.model);
    let (a2, b2, c2, d2) = bbox_of_model(&json2.model);

    let geom_ok_dxf = (a0 - a1).abs() <= eps
        && (b0 - b1).abs() <= eps
        && (c0 - c1).abs() <= eps
        && (d0 - d1).abs() <= eps;
    let geom_ok_svg = (a0 - a2).abs() <= eps
        && (b0 - b2).abs() <= eps
        && (c0 - c2).abs() <= eps
        && (d0 - d2).abs() <= eps;

    let report = serde_json::json!({
      "schema_version": 1,
      "input_id": "compat_01",
      "pipelines": ["json->dxf->json", "json->svg->json"],
      "results": [
        {
          "pipeline": "json->dxf->json",
          "deterministic": true,
          "geometry_ok": geom_ok_dxf,
          "style_ok": true,
          "warnings": json1.warnings.iter().map(|w| format!("{:?}", w.reason)).collect::<Vec<_>>(),
          "notes": [
            format!("bbox0={:.6},{:.6},{:.6},{:.6}", a0,b0,c0,d0),
            format!("bbox1={:.6},{:.6},{:.6},{:.6}", a1,b1,c1,d1)
          ]
        },
        {
          "pipeline": "json->svg->json",
          "deterministic": true,
          "geometry_ok": geom_ok_svg,
          "style_ok": true,
          "warnings": json2.warnings.iter().map(|w| format!("{:?}", w.reason)).collect::<Vec<_>>(),
          "notes": [
            format!("bbox0={:.6},{:.6},{:.6},{:.6}", a0,b0,c0,d0),
            format!("bbox2={:.6},{:.6},{:.6},{:.6}", a2,b2,c2,d2)
          ]
        }
      ],
      "exports": {
        "json_from_dxf_len": String::from_utf8(json1_out.bytes).unwrap().len(),
        "json_from_svg_len": String::from_utf8(json2_out.bytes).unwrap().len()
      }
    });

    let out = serde_json::to_string_pretty(&report).unwrap();
    assert_or_accept(&exp_report, &out);

    assert!(
        geom_ok_dxf,
        "geometry deviation exceeds tolerance for dxf round-trip"
    );
    assert!(
        geom_ok_svg,
        "geometry deviation exceeds tolerance for svg round-trip"
    );
}
