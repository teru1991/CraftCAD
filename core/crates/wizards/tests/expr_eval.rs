use craftcad_wizards::engine::eval_expr::eval_number_expr;
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn expr_eval_basic() {
    let mut m = BTreeMap::new();
    m.insert("inner_w_mm".into(), json!(200.0));
    m.insert("thickness_mm".into(), json!(12.0));
    let v = eval_number_expr("inner_w_mm + 2*thickness_mm", &m).unwrap();
    assert!((v - 224.0).abs() < 1e-9);
}
