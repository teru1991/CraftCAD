use diycad_geom::{intersect, project_point, split_at, EpsilonPolicy, Geom2D, SplitBy, Vec2};
use serde_json::Value;

fn round(v: f64, p: i32) -> f64 {
    let m = 10f64.powi(p);
    (v * m).round() / m
}

fn read_json(path: &str) -> Value {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../tests/golden/geom")
        .join(path);
    serde_json::from_str(&std::fs::read_to_string(root).expect("read fixture")).expect("json")
}

#[test]
fn golden_geom_cases() {
    let input = read_json("input.json");
    let expected = read_json("expected.json");
    let eps = EpsilonPolicy::default();

    for case in input["cases"].as_array().expect("cases") {
        let name = case["name"].as_str().expect("name");
        let op = case["op"].as_str().expect("op");
        match op {
            "intersect" => {
                let a: Geom2D = serde_json::from_value(case["a"].clone()).expect("geom a");
                let b: Geom2D = serde_json::from_value(case["b"].clone()).expect("geom b");
                let hit = intersect(&a, &b, &eps).unwrap_or_else(|e| {
                    panic!("case={name} failed reason={} eps={:?}", e.code, eps)
                });
                let mut pts: Vec<(f64, f64)> = hit
                    .points
                    .iter()
                    .map(|p| (round(p.x, 9), round(p.y, 9)))
                    .collect();
                pts.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.total_cmp(&b.1)));
                let exp = &expected[name];
                let exp_pts: Vec<(f64, f64)> = exp["points"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| (v[0].as_f64().unwrap(), v[1].as_f64().unwrap()))
                    .collect();
                assert_eq!(pts, exp_pts, "case={name} debug={}", hit.debug);
                assert_eq!(
                    hit.ambiguous,
                    exp["ambiguous"].as_bool().unwrap(),
                    "case={name}"
                );
            }
            "project" => {
                let geom: Geom2D = serde_json::from_value(case["geom"].clone()).expect("geom");
                let p = Vec2 {
                    x: case["point"]["x"].as_f64().unwrap(),
                    y: case["point"]["y"].as_f64().unwrap(),
                };
                let hit = project_point(&geom, p, &eps).unwrap_or_else(|e| {
                    panic!("case={name} failed reason={} eps={:?}", e.code, eps)
                });
                let exp = &expected[name];
                assert_eq!(
                    round(hit.point.x, 9),
                    exp["point"][0].as_f64().unwrap(),
                    "case={name}"
                );
                assert_eq!(
                    round(hit.point.y, 9),
                    exp["point"][1].as_f64().unwrap(),
                    "case={name}"
                );
                assert_eq!(
                    round(hit.t_global, 12),
                    exp["t_global"].as_f64().unwrap(),
                    "case={name}"
                );
            }
            "split_t" => {
                let geom: Geom2D = serde_json::from_value(case["geom"].clone()).expect("geom");
                let t = case["t"].as_f64().unwrap();
                let out = split_at(&geom, SplitBy::T(t), &eps).unwrap_or_else(|e| {
                    panic!("case={name} failed reason={} eps={:?}", e.code, eps)
                });
                let exp = &expected[name];
                assert_eq!(
                    round(out.split_point.x, 9),
                    exp["split_point"][0].as_f64().unwrap(),
                    "case={name}"
                );
                assert_eq!(
                    round(out.split_point.y, 9),
                    exp["split_point"][1].as_f64().unwrap(),
                    "case={name}"
                );
            }
            _ => panic!("unknown op={op}"),
        }
    }
}
