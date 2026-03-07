use serde_json::Value;
use std::process::Command;

#[test]
fn cli_outputs_ok_json_summary() {
    let exe = env!("CARGO_BIN_EXE_craftcad-determinism-check");
    let out = Command::new(exe).output().expect("run binary");
    assert!(out.status.success());

    let stdout = String::from_utf8(out.stdout).unwrap();
    let line = stdout.lines().find(|l| !l.trim().is_empty()).unwrap();
    let v: Value = serde_json::from_str(line).unwrap();
    assert_eq!(v.get("ok").and_then(Value::as_bool), Some(true));
    assert!(v.get("projection").is_some());
    assert!(v.get("estimate").and_then(Value::as_str).is_some());
    assert!(v.get("fastener_bom").and_then(Value::as_str).is_some());
    assert!(v.get("input_ssot_hash").and_then(Value::as_str).is_some());
}
