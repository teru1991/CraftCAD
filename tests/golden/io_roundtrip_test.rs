use craftcad_io::options::{ExportOptions, ImportOptions};
use craftcad_io::reasons::ReasonCode;
use craftcad_io::IoEngine;
use craftcad_io_json::JsonIo;

mod utils;
use utils::golden::*;

fn warnings_to_json(warnings: &[craftcad_io::reasons::AppError]) -> String {
    let mut arr = Vec::new();
    for w in warnings {
        arr.push(serde_json::json!({
            "reason": format!("{:?}", w.reason),
            "message": w.message,
            "hint": w.hint,
            "context": w.context,
            "is_fatal": w.is_fatal
        }));
    }
    serde_json::to_string_pretty(&arr).unwrap()
}

#[test]
fn golden_json_roundtrip_01() {
    let eng = IoEngine::new()
        .register_importer(Box::new(JsonIo::new()))
        .register_exporter(Box::new(JsonIo::new()));

    let input_path = "tests/golden/io_roundtrip/inputs/json/sample_01.json";
    let exp_model_path = "tests/golden/io_roundtrip/expected/normalized_internal_model.json";
    let exp_warn_path = "tests/golden/io_roundtrip/expected/warnings.json";
    let exp_out_path = "tests/golden/io_roundtrip/expected/exported_out.json";

    let bytes = read_bytes(input_path);
    let iopts = ImportOptions::default_for_tests();
    let res = eng.import("json", &bytes, &iopts).unwrap();

    let got_model = serde_json::to_string_pretty(&res.model).unwrap();
    assert_or_accept(exp_model_path, &got_model);

    let got_warn = warnings_to_json(&res.warnings);
    assert_or_accept(exp_warn_path, &got_warn);

    let eopts = ExportOptions::default_for_tests();
    let out = eng.export("json", &res.model, &eopts).unwrap();

    let got_out = String::from_utf8(out.bytes).unwrap();
    assert_or_accept(exp_out_path, &got_out);

    assert!(
        !res.warnings
            .iter()
            .any(|w| w.is_fatal && w.reason == ReasonCode::IO_PARSE_JSON_MALFORMED)
    );
}
