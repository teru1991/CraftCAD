use crate::engine::ast::{Template, UiInput};
use crate::reasons::{WizardReason, WizardReasonCode};
use serde_json::Value;
use std::collections::BTreeMap;

fn as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        _ => None,
    }
}
fn as_i64(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64(),
        _ => None,
    }
}
fn as_bool(v: &Value) -> Option<bool> {
    match v {
        Value::Bool(b) => Some(*b),
        _ => None,
    }
}

pub fn validate_inputs(
    tpl: &Template,
    inputs: &BTreeMap<String, Value>,
) -> Result<(), WizardReason> {
    for def in &tpl.ui_inputs {
        if let Some(v) = inputs.get(&def.key) {
            validate_one_input(def, v)?;
        }
    }
    for k in inputs.keys() {
        if !tpl.ui_inputs.iter().any(|d| &d.key == k) {
            return Err(WizardReason::new(
                WizardReasonCode::WizardInputInvalid,
                format!("unknown input key: {k}"),
            )
            .with_key(k.clone()));
        }
    }
    Ok(())
}

fn validate_one_input(def: &UiInput, v: &Value) -> Result<(), WizardReason> {
    match def.ty.as_str() {
        "number" => {
            let x = as_f64(v).ok_or_else(|| {
                WizardReason::new(
                    WizardReasonCode::WizardInputInvalid,
                    format!("{} must be number", def.key),
                )
                .with_key(def.key.clone())
            })?;
            let min = as_f64(&def.min).ok_or_else(|| {
                WizardReason::new(
                    WizardReasonCode::WizardTemplateInvalid,
                    format!("{} min must be number", def.key),
                )
            })?;
            let max = as_f64(&def.max).ok_or_else(|| {
                WizardReason::new(
                    WizardReasonCode::WizardTemplateInvalid,
                    format!("{} max must be number", def.key),
                )
            })?;
            if x < min || x > max {
                return Err(WizardReason::new(
                    WizardReasonCode::WizardInputInvalid,
                    format!("{} out of range [{min},{max}]: {x}", def.key),
                )
                .with_key(def.key.clone()));
            }
            Ok(())
        }
        "integer" => {
            let x = as_i64(v).ok_or_else(|| {
                WizardReason::new(
                    WizardReasonCode::WizardInputInvalid,
                    format!("{} must be integer", def.key),
                )
                .with_key(def.key.clone())
            })?;
            let min = as_i64(&def.min).ok_or_else(|| {
                WizardReason::new(
                    WizardReasonCode::WizardTemplateInvalid,
                    format!("{} min must be integer", def.key),
                )
            })?;
            let max = as_i64(&def.max).ok_or_else(|| {
                WizardReason::new(
                    WizardReasonCode::WizardTemplateInvalid,
                    format!("{} max must be integer", def.key),
                )
            })?;
            if x < min || x > max {
                return Err(WizardReason::new(
                    WizardReasonCode::WizardInputInvalid,
                    format!("{} out of range [{min},{max}]: {x}", def.key),
                )
                .with_key(def.key.clone()));
            }
            Ok(())
        }
        "bool" => {
            let _ = as_bool(v).ok_or_else(|| {
                WizardReason::new(
                    WizardReasonCode::WizardInputInvalid,
                    format!("{} must be bool", def.key),
                )
                .with_key(def.key.clone())
            })?;
            Ok(())
        }
        "enum" => {
            let s = v.as_str().ok_or_else(|| {
                WizardReason::new(
                    WizardReasonCode::WizardInputInvalid,
                    format!("{} must be string(enum)", def.key),
                )
                .with_key(def.key.clone())
            })?;
            if !def.enum_values.iter().any(|e| e == s) {
                return Err(WizardReason::new(
                    WizardReasonCode::WizardInputInvalid,
                    format!("{} invalid enum: {}", def.key, s),
                )
                .with_key(def.key.clone()));
            }
            Ok(())
        }
        other => Err(WizardReason::new(
            WizardReasonCode::WizardTemplateInvalid,
            format!("unknown ui input type: {other}"),
        )
        .with_key(def.key.clone())),
    }
}

pub fn canonicalize_inputs_json(
    tpl: &Template,
    inputs: &BTreeMap<String, Value>,
) -> Result<String, WizardReason> {
    let mut map = BTreeMap::<String, Value>::new();
    for def in &tpl.ui_inputs {
        let v = inputs
            .get(&def.key)
            .cloned()
            .unwrap_or_else(|| def.default.clone());
        validate_one_input(def, &v)?;
        map.insert(def.key.clone(), v);
    }
    serde_json::to_string(&map).map_err(|e| {
        WizardReason::new(
            WizardReasonCode::WizardDeterminismError,
            format!("canonicalize failed: {e}"),
        )
    })
}
