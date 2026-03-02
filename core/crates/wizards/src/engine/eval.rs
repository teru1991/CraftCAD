use crate::engine::ast::Template;
use crate::engine::eval_expr::eval_number_expr;
use crate::reasons::{WizardReason, WizardReasonCode};
use serde_json::Value;
use std::collections::BTreeMap;

fn num(v: f64) -> Result<Value, WizardReason> {
    serde_json::Number::from_f64(v)
        .map(Value::Number)
        .ok_or_else(|| {
            WizardReason::new(
                WizardReasonCode::WizardDslInvalid,
                format!("invalid numeric value: {v}"),
            )
        })
}

fn get_num_input(inputs: &BTreeMap<String, Value>, key: &str) -> Result<f64, WizardReason> {
    inputs.get(key).and_then(|v| v.as_f64()).ok_or_else(|| {
        WizardReason::new(
            WizardReasonCode::WizardDslInvalid,
            format!("missing/invalid numeric input: {key}"),
        )
        .with_key(key.to_string())
    })
}

fn get_bool_input(inputs: &BTreeMap<String, Value>, key: &str) -> Result<bool, WizardReason> {
    inputs.get(key).and_then(|v| v.as_bool()).ok_or_else(|| {
        WizardReason::new(
            WizardReasonCode::WizardDslInvalid,
            format!("missing/invalid bool input: {key}"),
        )
        .with_key(key.to_string())
    })
}

fn add_arg(out: &mut BTreeMap<String, Value>, k: &str, v: Value) {
    out.insert(k.to_string(), v);
}

pub fn eval_generation_steps(
    tpl: &Template,
    filled_inputs: &BTreeMap<String, Value>,
) -> Result<Vec<crate::types::EvaluatedOp>, WizardReason> {
    let mut out = vec![];

    for step in &tpl.generation_steps {
        match step.op.as_str() {
            "add_part_rect" => {
                let name = step
                    .args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        WizardReason::new(
                            WizardReasonCode::WizardDslInvalid,
                            "add_part_rect requires args.name",
                        )
                    })?;
                let mut args = BTreeMap::new();
                add_arg(&mut args, "name", Value::String(name.to_string()));

                if let (Some(w_expr), Some(h_expr)) = (
                    step.args.get("w_expr").and_then(|v| v.as_str()),
                    step.args.get("h_expr").and_then(|v| v.as_str()),
                ) {
                    let w = eval_number_expr(w_expr, filled_inputs)?;
                    let h = eval_number_expr(h_expr, filled_inputs)?;
                    if w <= 0.0 || h <= 0.0 {
                        return Err(WizardReason::new(
                            WizardReasonCode::WizardInputInvalid,
                            format!("evaluated dimensions must be >0: w={w}, h={h}"),
                        ));
                    }
                    add_arg(&mut args, "w_mm", num(w)?);
                    add_arg(&mut args, "h_mm", num(h)?);
                } else if step.args.contains_key("w_expr") || step.args.contains_key("h_expr") {
                    return Err(WizardReason::new(
                        WizardReasonCode::WizardDslInvalid,
                        "add_part_rect requires both w_expr and h_expr",
                    ));
                } else if let (Some(wk), Some(hk)) = (
                    step.args.get("w_key").and_then(|v| v.as_str()),
                    step.args.get("h_key").and_then(|v| v.as_str()),
                ) {
                    let w = get_num_input(filled_inputs, wk)?;
                    let h = get_num_input(filled_inputs, hk)?;
                    add_arg(&mut args, "w_mm", num(w)?);
                    add_arg(&mut args, "h_mm", num(h)?);
                } else if let (Some(wc), Some(hc)) = (
                    step.args.get("w_const").and_then(|v| v.as_f64()),
                    step.args.get("h_const").and_then(|v| v.as_f64()),
                ) {
                    add_arg(&mut args, "w_mm", num(wc)?);
                    add_arg(&mut args, "h_mm", num(hc)?);
                } else {
                    return Err(WizardReason::new(
                        WizardReasonCode::WizardDslInvalid,
                        "add_part_rect requires (w_expr,h_expr), (w_key,h_key) or (w_const,h_const)",
                    ));
                }

                out.push(crate::types::EvaluatedOp {
                    op: "add_part_rect".to_string(),
                    args,
                });
            }
            "add_hole_circle_array" => {
                let enabled_key = step
                    .args
                    .get("enabled_key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        WizardReason::new(
                            WizardReasonCode::WizardDslInvalid,
                            "add_hole_circle_array requires enabled_key",
                        )
                    })?;
                let enabled = get_bool_input(filled_inputs, enabled_key)?;
                if !enabled {
                    continue;
                }
                let part_name = step
                    .args
                    .get("part_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        WizardReason::new(
                            WizardReasonCode::WizardDslInvalid,
                            "add_hole_circle_array requires part_name",
                        )
                    })?;
                let diam_key = step
                    .args
                    .get("diam_key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        WizardReason::new(
                            WizardReasonCode::WizardDslInvalid,
                            "add_hole_circle_array requires diam_key",
                        )
                    })?;
                let offset_key = step
                    .args
                    .get("offset_key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        WizardReason::new(
                            WizardReasonCode::WizardDslInvalid,
                            "add_hole_circle_array requires offset_key",
                        )
                    })?;
                let pattern = step
                    .args
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        WizardReason::new(
                            WizardReasonCode::WizardDslInvalid,
                            "add_hole_circle_array requires pattern",
                        )
                    })?;

                let diam = get_num_input(filled_inputs, diam_key)?;
                let off = get_num_input(filled_inputs, offset_key)?;

                let mut args = BTreeMap::new();
                add_arg(&mut args, "part_name", Value::String(part_name.to_string()));
                add_arg(&mut args, "diameter_mm", num(diam)?);
                add_arg(&mut args, "offset_mm", num(off)?);
                add_arg(&mut args, "pattern", Value::String(pattern.to_string()));
                out.push(crate::types::EvaluatedOp {
                    op: "add_hole_circle_array".to_string(),
                    args,
                });
            }
            "add_annotation" => {
                let kind = step
                    .args
                    .get("kind")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        WizardReason::new(
                            WizardReasonCode::WizardDslInvalid,
                            "add_annotation requires kind",
                        )
                    })?;
                if let Some(enabled_key) = step.args.get("enabled_key").and_then(|v| v.as_str()) {
                    let enabled = get_bool_input(filled_inputs, enabled_key)?;
                    if !enabled {
                        continue;
                    }
                }
                let mut args = BTreeMap::new();
                add_arg(&mut args, "kind", Value::String(kind.to_string()));

                if let Some(text_key) = step.args.get("text_key").and_then(|v| v.as_str()) {
                    let x = filled_inputs.get(text_key).cloned().ok_or_else(|| {
                        WizardReason::new(WizardReasonCode::WizardDslInvalid, "text_key missing")
                    })?;
                    add_arg(&mut args, "text_value", x);
                } else if let Some(text) = step.args.get("text").and_then(|v| v.as_str()) {
                    if let Some(found) = filled_inputs.get(text) {
                        add_arg(&mut args, "text_value", found.clone());
                    } else {
                        add_arg(&mut args, "text_value", Value::String(text.to_string()));
                    }
                } else {
                    return Err(WizardReason::new(
                        WizardReasonCode::WizardDslInvalid,
                        "add_annotation requires text or text_key",
                    ));
                }
                out.push(crate::types::EvaluatedOp {
                    op: "add_annotation".to_string(),
                    args,
                });
            }
            "recommend_nest" => {
                let mut args = BTreeMap::new();
                if let Some(m) = step.args.get("material_hint") {
                    args.insert("material_hint".into(), m.clone());
                }
                if let Some(p) = step.args.get("process_hint") {
                    args.insert("process_hint".into(), p.clone());
                }
                out.push(crate::types::EvaluatedOp {
                    op: "recommend_nest".to_string(),
                    args,
                });
            }
            other => {
                return Err(WizardReason::new(
                    WizardReasonCode::WizardDslInvalid,
                    format!("unsupported op: {other}"),
                ));
            }
        }
    }

    out.sort_by(|a, b| match a.op.cmp(&b.op) {
        std::cmp::Ordering::Equal => serde_json::to_string(&a.args)
            .unwrap_or_default()
            .cmp(&serde_json::to_string(&b.args).unwrap_or_default()),
        o => o,
    });

    Ok(out)
}
