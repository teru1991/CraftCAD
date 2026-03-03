use jsonschema::JSONSchema;
use regex::Regex;
use serde_json::Value;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn reason_catalog_valid_and_links_exist() {
    let schema_raw = std::fs::read_to_string("../../docs/specs/errors/catalog.schema.json")
        .expect("schema read");
    let catalog_raw =
        std::fs::read_to_string("../../docs/specs/errors/catalog.json").expect("catalog read");

    let schema: serde_json::Value = serde_json::from_str(&schema_raw).expect("schema json");
    let catalog: serde_json::Value = serde_json::from_str(&catalog_raw).expect("catalog json");

    let compiled = jsonschema::JSONSchema::compile(&schema).expect("compile schema");
    let result = compiled.validate(&catalog);
    if let Err(errors) = result {
        let issues: Vec<String> = errors.map(|e| e.to_string()).collect();
        panic!("catalog validation failed: {}", issues.join("; "));
    }

    let mut uniq = BTreeSet::new();
    for item in catalog["items"].as_array().expect("items array") {
        let code = item["code"].as_str().expect("code");
        assert!(uniq.insert(code.to_string()), "duplicate code: {code}");

        let link = item["doc_link"].as_str().expect("doc_link");
        assert!(
            Path::new("../..").join(link).exists(),
            "missing doc_link target: {link}"
        );
    }
}

#[test]
fn io_support_matrix_is_machine_readable() {
    let raw =
        std::fs::read_to_string("../../docs/specs/io/support_matrix.json").expect("support matrix");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("support matrix json");
    assert!(value["formats"].is_array());
    assert!(value["matrix"].is_array());
}

#[test]
fn dataset_manifest_references_existing_files() {
    let raw =
        std::fs::read_to_string("../../tests/datasets/manifest.json").expect("dataset manifest");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("dataset manifest json");
    for ds in value["datasets"].as_array().expect("datasets") {
        for key in ["inputs", "expected", "expected_outputs"] {
            let Some(entries) = ds.get(key).and_then(|v| v.as_array()) else {
                continue;
            };
            for p in entries {
                let rel = p
                    .get("path")
                    .and_then(|v| v.as_str())
                    .or_else(|| p.as_str())
                    .expect("path str");
                assert!(
                    Path::new("../..").join(rel).exists(),
                    "missing dataset file: {rel}"
                );
            }
        }
    }
}

#[test]
fn drawing_style_ssot_is_valid_and_named_consistently() {
    let schema_raw = std::fs::read_to_string("../../docs/specs/drawing/style_ssot.schema.json")
        .expect("style schema read");
    let style_raw =
        std::fs::read_to_string("../../docs/specs/drawing/style_ssot.json").expect("style read");

    let schema: serde_json::Value = serde_json::from_str(&schema_raw).expect("style schema json");
    let style: serde_json::Value = serde_json::from_str(&style_raw).expect("style json");

    let compiled = jsonschema::JSONSchema::compile(&schema).expect("compile style schema");
    if let Err(errors) = compiled.validate(&style) {
        let issues: Vec<String> = errors.map(|e| e.to_string()).collect();
        panic!("style_ssot validation failed: {}", issues.join("; "));
    }

    let mut style_names = BTreeSet::new();
    for line_style in style["line_styles"].as_array().expect("line_styles") {
        let name = line_style["name"].as_str().expect("line_style.name");
        assert!(
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "invalid line style name: {name}"
        );
        assert!(
            style_names.insert(name.to_string()),
            "duplicate line style: {name}"
        );
    }

    let weights = style["line_weights"].as_object().expect("line_weights");
    let mut weight_names = BTreeSet::new();
    for name in weights.keys() {
        assert!(
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "invalid line weight name: {name}"
        );
        assert!(
            weight_names.insert(name.to_string()),
            "duplicate line weight: {name}"
        );
    }
}

fn repo_root_from_manifest() -> PathBuf {
    // core/serialize を基準にリポジトリルートへ（想定：<repo>/core/serialize）
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .expect(
            "failed to locate repo root from CARGO_MANIFEST_DIR (expected <repo>/core/serialize)",
        )
}

fn read_json(path: &Path) -> Value {
    let s = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    serde_json::from_str(&s)
        .unwrap_or_else(|e| panic!("failed to parse json {}: {}", path.display(), e))
}

fn compile_schema(schema_path: &Path) -> JSONSchema {
    let schema_json = read_json(schema_path);
    JSONSchema::options()
        .compile(&schema_json)
        .unwrap_or_else(|e| {
            panic!(
                "failed to compile jsonschema {}: {}",
                schema_path.display(),
                e
            )
        })
}

fn validate_instance(schema: &JSONSchema, instance_path: &Path, instance: &Value) {
    if let Err(errors) = schema.validate(instance) {
        let mut msgs: Vec<String> = vec![];
        for err in errors {
            msgs.push(format!("{} @ {}", err, err.instance_path));
        }
        panic!(
            "schema validation failed for {}\n{}",
            instance_path.display(),
            msgs.join("\n")
        );
    }
}

fn rect_inside_page(rect: (f64, f64, f64, f64), page_w: f64, page_h: f64) -> bool {
    let (x, y, w, h) = rect;
    x >= 0.0
        && y >= 0.0
        && w > 0.0
        && h > 0.0
        && (x + w) <= page_w + 1e-9
        && (y + h) <= page_h + 1e-9
}

fn get_f64(v: &Value, p: &str) -> f64 {
    v.pointer(p)
        .and_then(|x| x.as_f64())
        .unwrap_or_else(|| panic!("missing or non-number at json pointer: {p}"))
}

fn get_str(v: &Value, p: &str) -> String {
    v.pointer(p)
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| panic!("missing or non-string at json pointer: {p}"))
}

#[test]
fn ssot_lint_drawing_style_specs() {
    let root = repo_root_from_manifest();
    let base = root.join("docs/specs/drawing_style");

    let style_schema_path = base.join("style_ssot.schema.json");
    let style_json_path = base.join("style_ssot.json");

    let sheet_schema_path = base.join("sheet_templates.schema.json");
    let sheet_json_path = base.join("sheet_templates.json");

    let print_schema_path = base.join("print_presets.schema.json");
    let print_json_path = base.join("print_presets.json");

    // 1) schema validation（draft2020-12）
    let style_schema = compile_schema(&style_schema_path);
    let style_json = read_json(&style_json_path);
    validate_instance(&style_schema, &style_json_path, &style_json);

    let sheet_schema = compile_schema(&sheet_schema_path);
    let sheet_json = read_json(&sheet_json_path);
    validate_instance(&sheet_schema, &sheet_json_path, &sheet_json);

    let print_schema = compile_schema(&print_schema_path);
    let print_json = read_json(&print_json_path);
    validate_instance(&print_schema, &print_json_path, &print_json);

    // 2) ID規約（*_vN）と一意性
    let id_re = Regex::new(r"^[a-z][a-z0-9_]*_v[0-9]+$").unwrap();

    let mut style_ids = HashSet::<String>::new();
    let styles = style_json
        .pointer("/styles")
        .and_then(|x| x.as_array())
        .expect("style_ssot.json: /styles must be array");
    for s in styles {
        let id = s.get("id").and_then(|x| x.as_str()).unwrap_or("");
        assert!(id_re.is_match(id), "style id must match *_vN: {id}");
        assert!(style_ids.insert(id.to_string()), "duplicate style id: {id}");
    }

    let mut template_ids = HashSet::<String>::new();
    let templates = sheet_json
        .pointer("/templates")
        .and_then(|x| x.as_array())
        .expect("sheet_templates.json: /templates must be array");
    for t in templates {
        let id = t.get("id").and_then(|x| x.as_str()).unwrap_or("");
        assert!(id_re.is_match(id), "template id must match *_vN: {id}");
        assert!(
            template_ids.insert(id.to_string()),
            "duplicate template id: {id}"
        );
    }

    let mut preset_ids = HashSet::<String>::new();
    let presets = print_json
        .pointer("/presets")
        .and_then(|x| x.as_array())
        .expect("print_presets.json: /presets must be array");
    for p in presets {
        let id = p.get("id").and_then(|x| x.as_str()).unwrap_or("");
        assert!(id_re.is_match(id), "print preset id must match *_vN: {id}");
        assert!(
            preset_ids.insert(id.to_string()),
            "duplicate print preset id: {id}"
        );
    }

    // 3) sheet_templates：A4/A3サイズ・margin・title_block・viewport整合
    // A4: 210x297, A3: 297x420（mm）
    let mut template_map: HashMap<String, (f64, f64, Value)> = HashMap::new();

    for t in templates {
        let id = t.get("id").unwrap().as_str().unwrap().to_string();
        let page_w = get_f64(t, "/page/width_mm");
        let page_h = get_f64(t, "/page/height_mm");
        let size = get_str(t, "/page/size");

        match size.as_str() {
            "A4" => {
                assert!(
                    (page_w - 210.0).abs() < 1e-6 && (page_h - 297.0).abs() < 1e-6,
                    "A4 size must be 210x297mm: {id} = {page_w}x{page_h}"
                );
            }
            "A3" => {
                assert!(
                    (page_w - 297.0).abs() < 1e-6 && (page_h - 420.0).abs() < 1e-6,
                    "A3 size must be 297x420mm: {id} = {page_w}x{page_h}"
                );
            }
            _ => panic!("unknown page.size for template {id}: {size}"),
        }

        let m_top = get_f64(t, "/page/margin_mm/top");
        let m_right = get_f64(t, "/page/margin_mm/right");
        let m_bottom = get_f64(t, "/page/margin_mm/bottom");
        let m_left = get_f64(t, "/page/margin_mm/left");
        assert!(m_left + m_right < page_w, "margins exceed page width: {id}");
        assert!(
            m_top + m_bottom < page_h,
            "margins exceed page height: {id}"
        );

        let tb_x = get_f64(t, "/page/title_block/bbox_mm/x_mm");
        let tb_y = get_f64(t, "/page/title_block/bbox_mm/y_mm");
        let tb_w = get_f64(t, "/page/title_block/bbox_mm/w_mm");
        let tb_h = get_f64(t, "/page/title_block/bbox_mm/h_mm");
        assert!(
            rect_inside_page((tb_x, tb_y, tb_w, tb_h), page_w, page_h),
            "title_block bbox out of page: {id}"
        );

        // title_block は “margin内” に収める（印刷で必須）
        assert!(
            tb_x + 1e-9 >= m_left,
            "title_block must be inside left margin region: {id}"
        );
        assert!(
            tb_y + 1e-9 >= m_top,
            "title_block must be inside top margin region: {id}"
        );
        assert!(
            tb_x + tb_w <= page_w - m_right + 1e-9,
            "title_block must be inside right margin region: {id}"
        );
        assert!(
            tb_y + tb_h <= page_h - m_bottom + 1e-9,
            "title_block must be inside bottom margin region: {id}"
        );

        // viewport
        let vx = get_f64(t, "/viewports/model_view_region/x_mm");
        let vy = get_f64(t, "/viewports/model_view_region/y_mm");
        let vw = get_f64(t, "/viewports/model_view_region/w_mm");
        let vh = get_f64(t, "/viewports/model_view_region/h_mm");
        assert!(
            rect_inside_page((vx, vy, vw, vh), page_w, page_h),
            "model_view_region out of page: {id}"
        );

        // viewportはmargin内に収める（図枠外には描かない）
        assert!(
            vx + 1e-9 >= m_left,
            "model_view_region must be inside left margin region: {id}"
        );
        assert!(
            vy + 1e-9 >= m_top,
            "model_view_region must be inside top margin region: {id}"
        );
        assert!(
            vx + vw <= page_w - m_right + 1e-9,
            "model_view_region must be inside right margin region: {id}"
        );
        assert!(
            vy + vh <= page_h - m_bottom + 1e-9,
            "model_view_region must be inside bottom margin region: {id}"
        );

        // viewport と title_block の重なりは禁止（ここでは厳格に禁止）
        let overlap_x = (vx.max(tb_x)) < ((vx + vw).min(tb_x + tb_w));
        let overlap_y = (vy.max(tb_y)) < ((vy + vh).min(tb_y + tb_h));
        assert!(
            !(overlap_x && overlap_y),
            "model_view_region must not overlap title_block: {id}"
        );

        template_map.insert(id.clone(), (page_w, page_h, t.clone()));
    }

    // 4) print_presets：target_template参照＆margin整合＆基本妥当性
    for p in presets {
        let pid = p.get("id").unwrap().as_str().unwrap();
        let target = p
            .get("target_template_id")
            .and_then(|x| x.as_str())
            .unwrap_or("");
        assert!(
            template_map.contains_key(target),
            "print preset {pid} targets unknown template_id: {target}"
        );

        let (_w, _h, t) = template_map.get(target).unwrap();
        let tm_top = get_f64(t, "/page/margin_mm/top");
        let tm_right = get_f64(t, "/page/margin_mm/right");
        let tm_bottom = get_f64(t, "/page/margin_mm/bottom");
        let tm_left = get_f64(t, "/page/margin_mm/left");

        let pm_top = get_f64(p, "/margins_mm/top");
        let pm_right = get_f64(p, "/margins_mm/right");
        let pm_bottom = get_f64(p, "/margins_mm/bottom");
        let pm_left = get_f64(p, "/margins_mm/left");

        // 最初は“テンプレと同じmargin”を必須（ズレると印刷で事故るため）
        assert!(
            (pm_top - tm_top).abs() < 1e-6,
            "preset margin(top) must match template margin: {pid}"
        );
        assert!(
            (pm_right - tm_right).abs() < 1e-6,
            "preset margin(right) must match template margin: {pid}"
        );
        assert!(
            (pm_bottom - tm_bottom).abs() < 1e-6,
            "preset margin(bottom) must match template margin: {pid}"
        );
        assert!(
            (pm_left - tm_left).abs() < 1e-6,
            "preset margin(left) must match template margin: {pid}"
        );

        let mode = get_str(p, "/scale_policy/mode");
        if mode == "fixed" {
            let fixed_scale = get_f64(p, "/scale_policy/fixed_scale");
            assert!(fixed_scale > 0.0, "fixed_scale must be > 0: {pid}");
        }

        let min_text = get_f64(p, "/min_readable_text_height_mm");
        assert!(
            (2.0..=6.0).contains(&min_text),
            "min_readable_text_height_mm should be practical (2..6): {pid} => {min_text}"
        );

        let lw = get_f64(p, "/line_weight_scale");
        assert!(
            (0.2..=5.0).contains(&lw),
            "line_weight_scale should be practical (0.2..5): {pid} => {lw}"
        );

        let color_mode = get_str(p, "/color_mode");
        assert!(
            color_mode == "color" || color_mode == "grayscale" || color_mode == "bw",
            "invalid color_mode: {pid} => {color_mode}"
        );

        let svg_prec = p
            .pointer("/export/svg_precision")
            .and_then(|x| x.as_i64())
            .unwrap_or(-1);
        assert!(
            (0..=8).contains(&svg_prec),
            "svg_precision out of range: {pid} => {svg_prec}"
        );
    }
}

const SUPPORT_MATRIX_JSON: &str = "docs/specs/io/support_matrix.json";
const MAPPING_RULES_JSON: &str = "docs/specs/io/mapping_rules.json";
const CURVE_APPROX_POLICY_MD: &str = "docs/specs/io/curve_approx_policy.md";
const POSTPROCESS_POLICY_MD: &str = "docs/specs/io/postprocess_policy.md";
const REASON_CATALOG_JSON: &str = "docs/specs/errors/catalog.json";

fn read_repo_file(path: &str) -> String {
    let abs = repo_root_from_manifest().join(path);
    fs::read_to_string(&abs).unwrap_or_else(|e| panic!("failed to read {}: {}", abs.display(), e))
}

fn load_json(path: &str) -> Value {
    let s = read_repo_file(path);
    serde_json::from_str(&s).unwrap_or_else(|e| panic!("invalid json {path}: {e}"))
}

fn read_text(path: &str) -> String {
    read_repo_file(path)
}

fn lint_policy_md_required_keys(path: &str) {
    let text = read_text(path);

    let meta_pos = text
        .find("ssot_meta:")
        .unwrap_or_else(|| panic!("{path}: missing 'ssot_meta:' block"));

    let req_pos = text[meta_pos..]
        .find("required_keys:")
        .unwrap_or_else(|| panic!("{path}: missing 'required_keys' in ssot_meta"));

    let after = &text[meta_pos + req_pos..];
    let lb = after
        .find('[')
        .unwrap_or_else(|| panic!("{path}: required_keys must be array: [..]"));
    let rb = after
        .find(']')
        .unwrap_or_else(|| panic!("{path}: required_keys must be array: [..]"));
    let inside = &after[lb + 1..rb];

    let keys: Vec<String> = inside
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if keys.is_empty() {
        panic!("{path}: required_keys is empty");
    }

    for k in keys {
        if !text.contains(&k) {
            panic!("{path}: required key '{k}' not mentioned anywhere in the document body");
        }
    }
}

#[test]
fn ssot_lint_io_support_matrix_best_effort_has_reason_codes_and_catalog_exists() {
    let sm = load_json(SUPPORT_MATRIX_JSON);
    let rc = load_json(REASON_CATALOG_JSON);

    let mut known: HashSet<String> = HashSet::new();
    if let Some(arr) = rc.get("items").and_then(|v| v.as_array()) {
        for r in arr {
            if let Some(code) = r.get("code").and_then(|v| v.as_str()) {
                known.insert(code.to_string());
            }
        }
    } else if let Some(arr) = rc.get("reasons").and_then(|v| v.as_array()) {
        for r in arr {
            if let Some(code) = r.get("code").and_then(|v| v.as_str()) {
                known.insert(code.to_string());
            }
        }
    } else if let Some(obj) = rc.get("codes").and_then(|v| v.as_object()) {
        for (k, _) in obj {
            known.insert(k.to_string());
        }
    } else {
        panic!(
            "{REASON_CATALOG_JSON}: unknown ReasonCatalog structure (expected 'items' array, 'reasons' array or 'codes' object)"
        );
    }

    let matrix = sm
        .get("matrix")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("{SUPPORT_MATRIX_JSON}: missing 'matrix' array"));

    for entry in matrix {
        let format = entry.get("format").and_then(|v| v.as_str()).unwrap_or("");
        let direction = entry
            .get("direction")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let feature = entry.get("feature").and_then(|v| v.as_str()).unwrap_or("");
        let level = entry.get("level").and_then(|v| v.as_str()).unwrap_or("");

        if format.is_empty() || direction.is_empty() || feature.is_empty() || level.is_empty() {
            panic!(
                "{SUPPORT_MATRIX_JSON}: matrix entry must have format/direction/feature/level: {entry:?}"
            );
        }

        if direction != "import" && direction != "export" {
            panic!("{SUPPORT_MATRIX_JSON}: invalid direction '{direction}', must be import|export");
        }

        if level == "best_effort" || level == "not_supported" {
            let reasons = entry
                .get("reason_codes")
                .and_then(|v| v.as_array())
                .unwrap_or_else(|| {
                    panic!(
                        "{SUPPORT_MATRIX_JSON}: {format} {direction} {feature}: '{level}' requires reason_codes array"
                    )
                });

            if reasons.is_empty() {
                panic!(
                    "{SUPPORT_MATRIX_JSON}: {format} {direction} {feature}: reason_codes must not be empty"
                );
            }

            for r in reasons {
                let code = r.as_str().unwrap_or_else(|| {
                    panic!("{SUPPORT_MATRIX_JSON}: reason_codes must be string: {r:?}")
                });
                if !known.contains(code) {
                    panic!(
                        "{SUPPORT_MATRIX_JSON}: unknown ReasonCode '{code}' (not found in {REASON_CATALOG_JSON})"
                    );
                }
            }
        }

        if level == "best_effort" {
            let action = entry.get("action").and_then(|v| v.as_str()).unwrap_or("");
            if action.is_empty() {
                panic!(
                    "{SUPPORT_MATRIX_JSON}: {format} {direction} {feature}: best_effort requires 'action'"
                );
            }
        }

        if level == "not_supported" {
            let action = entry.get("action").and_then(|v| v.as_str()).unwrap_or("");
            if action.is_empty() {
                panic!(
                    "{SUPPORT_MATRIX_JSON}: {format} {direction} {feature}: not_supported requires 'action'"
                );
            }
        }
    }
}

#[test]
fn ssot_lint_io_mapping_rules_required_keys_and_forbidden_chars_regex_present() {
    let mr = load_json(MAPPING_RULES_JSON);

    for k in ["layer", "linetype", "units"] {
        if mr.get(k).is_none() {
            panic!("{MAPPING_RULES_JSON}: missing top-level key '{k}'");
        }
    }

    for section in ["layer", "linetype"] {
        let s = mr.get(section).unwrap();
        for k in ["default", "max_len", "forbidden_chars_regex", "normalize"] {
            if s.get(k).is_none() {
                panic!("{MAPPING_RULES_JSON}: '{section}' missing key '{k}'");
            }
        }
        let re = s
            .get("forbidden_chars_regex")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if re.is_empty() {
            panic!("{MAPPING_RULES_JSON}: '{section}' forbidden_chars_regex must not be empty");
        }
    }

    let units = mr.get("units").unwrap();
    for k in ["supported", "default", "import_guess_order"] {
        if units.get(k).is_none() {
            panic!("{MAPPING_RULES_JSON}: 'units' missing key '{k}'");
        }
    }
    let supported = units.get("supported").and_then(|v| v.as_array()).unwrap();
    if supported.is_empty() {
        panic!("{MAPPING_RULES_JSON}: units.supported must not be empty");
    }
}

#[test]
fn ssot_lint_io_policy_docs_have_ssot_meta_and_required_keys() {
    lint_policy_md_required_keys(CURVE_APPROX_POLICY_MD);
    lint_policy_md_required_keys(POSTPROCESS_POLICY_MD);
}

use semver::Version;

fn repo_root_from_manifest_s13() -> PathBuf {
    let start = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for up in 0..=6usize {
        let mut p = start.clone();
        for _ in 0..up {
            p = p.parent().unwrap_or(&p).to_path_buf();
        }
        if p.join("docs").join("specs").exists() {
            return p;
        }
    }
    panic!(
        "repo root not found from CARGO_MANIFEST_DIR={}",
        start.display()
    );
}

fn resolve_local_refs_s13(value: &mut Value, schema_root: &Path) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(r)) = map.get("$ref") {
                if !r.contains("://") && !r.starts_with('#') {
                    let target = schema_root.join(r);
                    let mut inlined = read_json(&target);
                    resolve_local_refs_s13(&mut inlined, schema_root);
                    *value = inlined;
                    return;
                }
            }
            for v in map.values_mut() {
                resolve_local_refs_s13(v, schema_root);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                resolve_local_refs_s13(v, schema_root);
            }
        }
        _ => {}
    }
}

fn compile_schema_with_root_s13(schema_path: &Path, schema_root: &Path) -> JSONSchema {
    let mut schema_json = read_json(schema_path);
    resolve_local_refs_s13(&mut schema_json, schema_root);
    JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft7)
        .compile(&schema_json)
        .unwrap_or_else(|e| panic!("schema compile failed: {}: {}", schema_path.display(), e))
}

fn semver_must_parse_s13(v: &str, ctx: &str) {
    Version::parse(v).unwrap_or_else(|e| panic!("semver invalid ({}): {} ({})", ctx, v, e));
}

fn id_must_match_s13(re: &Regex, id: &str, ctx: &str) {
    if !re.is_match(id) {
        panic!("id invalid ({}): {}", ctx, id);
    }
    let lower = id.to_ascii_lowercase();
    let reserved = [
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
        "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ];
    if reserved.contains(&lower.as_str()) {
        panic!("id is reserved word ({}): {}", ctx, id);
    }
}

fn collect_builtin_preset_ids_s13(bundle: &Value) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for key in ["materials", "processes", "outputs", "hardware"] {
        let arr = bundle
            .get(key)
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("bundle missing array: {}", key));
        for item in arr {
            let id = item
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("preset missing id in {}", key));
            set.insert(id.to_string());
        }
    }
    set
}

fn validate_tags_policy_s13(tags_policy: &Value) {
    let schema_version = tags_policy
        .get("schema_version")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);
    if schema_version < 1 {
        panic!("tags.schema.json: schema_version must be >=1");
    }

    let max_len = tags_policy
        .get("max_len")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);
    if max_len != 32 {
        panic!("tags.schema.json: max_len must be 32 (got {})", max_len);
    }

    let normalize = tags_policy
        .get("normalize")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| panic!("tags.schema.json: normalize missing"));
    let expect_true = |k: &str| {
        let b = normalize.get(k).and_then(|v| v.as_bool()).unwrap_or(false);
        if !b {
            panic!("tags.schema.json: normalize.{} must be true", k);
        }
    };
    expect_true("lowercase");
    expect_true("trim");
    expect_true("collapse_spaces");
    expect_true("remove_zenkaku_spaces");

    let forb = tags_policy
        .get("forbidden_patterns")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("tags.schema.json: forbidden_patterns missing"));
    let patterns: BTreeSet<String> = forb
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let must = ["/", "\\\\", "\\.\\.", ":", "\\0"];
    for m in must {
        if !patterns.contains(m) {
            panic!("tags.schema.json: forbidden_patterns must include '{}'", m);
        }
    }
}

fn validate_template_required_presets_s13(template: &Value, builtin_ids: &BTreeSet<String>) {
    let req = template
        .get("required_presets")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| panic!("template.required_presets missing"));
    let keys = [
        "material_preset_ids",
        "process_preset_ids",
        "output_preset_ids",
        "hardware_preset_ids",
    ];
    for k in keys {
        if let Some(arr) = req.get(k).and_then(|v| v.as_array()) {
            for idv in arr {
                let id = idv.as_str().unwrap_or_else(|| {
                    panic!("template.required_presets.{} contains non-string", k)
                });
                if !builtin_ids.contains(id) {
                    panic!("template requires missing preset: {} (field={})", id, k);
                }
            }
        }
    }
}

#[test]
fn spec_ssot_lint_presets_templates_library() {
    let root = repo_root_from_manifest_s13();

    let presets_dir = root.join("docs").join("specs").join("presets");
    let templates_dir = root.join("docs").join("specs").join("templates");
    let library_dir = root.join("docs").join("specs").join("library");

    let material_schema = presets_dir.join("material_preset.schema.json");
    let process_schema = presets_dir.join("process_preset.schema.json");
    let output_schema = presets_dir.join("output_preset.schema.json");
    let hardware_schema = presets_dir.join("hardware_preset.schema.json");
    let bundle_schema = presets_dir.join("presets_bundle.schema.json");
    let template_schema = templates_dir.join("wizard_template.schema.json");
    let tags_schema = library_dir.join("tags.schema.json");

    for p in [
        &material_schema,
        &process_schema,
        &output_schema,
        &hardware_schema,
        &bundle_schema,
        &template_schema,
        &tags_schema,
    ] {
        assert!(p.exists(), "missing SSOT file: {}", p.display());
    }

    let _ = compile_schema_with_root_s13(&material_schema, &presets_dir);
    let _ = compile_schema_with_root_s13(&process_schema, &presets_dir);
    let _ = compile_schema_with_root_s13(&output_schema, &presets_dir);
    let _ = compile_schema_with_root_s13(&hardware_schema, &presets_dir);
    let _ = compile_schema_with_root_s13(&bundle_schema, &presets_dir);
    let compiled_template_schema = compile_schema_with_root_s13(&template_schema, &templates_dir);
    let _ = compile_schema_with_root_s13(&tags_schema, &library_dir);

    let tags_policy = read_json(&tags_schema);
    validate_tags_policy_s13(&tags_policy);

    let built_in = presets_dir.join("built_in_presets.json");
    assert!(built_in.exists(), "missing: {}", built_in.display());
    let bundle = read_json(&built_in);

    let compiled_bundle_schema = compile_schema_with_root_s13(&bundle_schema, &presets_dir);
    if let Err(errors) = compiled_bundle_schema.validate(&bundle) {
        let mut msg = String::new();
        for e in errors {
            msg.push_str(&format!("- {} at {}\n", e, e.instance_path));
        }
        panic!("built_in_presets.json schema validation failed:\n{}", msg);
    }

    let id_re = Regex::new(r"^[a-z0-9][a-z0-9_\-]*$").unwrap();
    let mut seen_ids = BTreeSet::new();

    for (group, key) in [
        ("materials", "materials"),
        ("processes", "processes"),
        ("outputs", "outputs"),
        ("hardware", "hardware"),
    ] {
        let arr = bundle.get(key).and_then(|v| v.as_array()).unwrap();
        for item in arr {
            let id = item.get("id").and_then(|v| v.as_str()).unwrap();
            let ver = item.get("version").and_then(|v| v.as_str()).unwrap();
            id_must_match_s13(&id_re, id, group);
            semver_must_parse_s13(ver, &format!("preset:{}:{}", group, id));
            if !seen_ids.insert(id.to_string()) {
                panic!("duplicate preset id across bundle: {}", id);
            }
            if let Some(t) = item.get("thickness_mm").and_then(|v| v.as_f64()) {
                if t <= 0.0 {
                    panic!("thickness_mm must be > 0: {}", id);
                }
            }
            if let Some(k) = item.get("kerf_mm").and_then(|v| v.as_f64()) {
                if k < 0.0 {
                    panic!("kerf_mm must be >= 0: {}", id);
                }
            }
            if let Some(m) = item.get("margin_mm").and_then(|v| v.as_f64()) {
                if m < 0.0 {
                    panic!("margin_mm must be >= 0: {}", id);
                }
            }
        }
    }

    let builtin_ids = collect_builtin_preset_ids_s13(&bundle);

    let template_files = [
        templates_dir.join("shelf_wizard.template.json"),
        templates_dir.join("box_wizard.template.json"),
        templates_dir.join("leather_pouch_wizard.template.json"),
    ];
    for tf in template_files {
        assert!(tf.exists(), "missing template: {}", tf.display());
        let t = read_json(&tf);

        if let Err(errors) = compiled_template_schema.validate(&t) {
            let mut msg = String::new();
            for e in errors {
                msg.push_str(&format!("- {} at {}\n", e, e.instance_path));
            }
            panic!(
                "template schema validation failed: {}\n{}",
                tf.display(),
                msg
            );
        }

        let tid = t.get("template_id").and_then(|v| v.as_str()).unwrap();
        let tver = t.get("template_version").and_then(|v| v.as_str()).unwrap();
        id_must_match_s13(&id_re, tid, "template_id");
        semver_must_parse_s13(tver, &format!("template:{}", tid));

        validate_template_required_presets_s13(&t, &builtin_ids);
    }

    let processes: BTreeSet<String> = bundle
        .get("processes")
        .and_then(|v| v.as_array())
        .unwrap()
        .iter()
        .map(|x| x.get("id").and_then(|v| v.as_str()).unwrap().to_string())
        .collect();

    for m in bundle.get("materials").and_then(|v| v.as_array()).unwrap() {
        if let Some(arr) = m.get("recommended_process_ids").and_then(|v| v.as_array()) {
            let mid = m.get("id").and_then(|v| v.as_str()).unwrap();
            for pidv in arr {
                let pid = pidv.as_str().unwrap();
                if !processes.contains(pid) {
                    panic!(
                        "material {} recommends missing process preset: {}",
                        mid, pid
                    );
                }
            }
        }
    }
}

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DatasetManifest {
    datasets: Vec<DatasetEntry>,
}

#[derive(Debug, Deserialize)]
struct DatasetEntry {
    #[serde(alias = "dataset_id", alias = "id")]
    dataset_id: String,
}

#[test]
fn ssot_perf_budgets_json_is_valid_and_consistent() {
    let root = repo_root_from_manifest();

    // 1) 必須SSOTファイルの存在チェック（最小）
    let required_md = [
        "docs/specs/perf/profiling.md",
        "docs/specs/perf/job_queue.md",
        "docs/specs/perf/cache_policy.md",
        "docs/specs/perf/lod_policy.md",
        "docs/specs/perf/memory_policy.md",
    ];
    for p in required_md {
        assert!(root.join(p).exists(), "Missing perf SSOT doc: {p}");
    }

    // 2) datasets manifest を読み込む（tests/datasets/manifest.json）
    let manifest_path = root.join("tests/datasets/manifest.json");
    assert!(
        manifest_path.exists(),
        "Missing dataset manifest SSOT: {}",
        manifest_path.display()
    );

    let manifest_text = std::fs::read_to_string(&manifest_path)
        .expect("Failed to read tests/datasets/manifest.json");
    let manifest: DatasetManifest =
        serde_json::from_str(&manifest_text).expect("Invalid tests/datasets/manifest.json schema");

    let mut dataset_ids = BTreeSet::<String>::new();
    for d in manifest.datasets {
        assert!(
            !d.dataset_id.trim().is_empty(),
            "dataset_id must not be empty in manifest"
        );
        assert!(
            dataset_ids.insert(d.dataset_id.clone()),
            "Duplicate dataset_id in manifest: {}",
            d.dataset_id
        );
    }
    assert!(
        !dataset_ids.is_empty(),
        "manifest datasets must not be empty"
    );

    // 3) budgets.json を schema で検証
    let budgets_path = root.join("docs/specs/perf/budgets.json");
    let schema_path = root.join("docs/specs/perf/budgets.schema.json");
    assert!(
        budgets_path.exists(),
        "Missing budgets.json: {}",
        budgets_path.display()
    );
    assert!(
        schema_path.exists(),
        "Missing budgets.schema.json: {}",
        schema_path.display()
    );

    let budgets_text = std::fs::read_to_string(&budgets_path).expect("Failed to read budgets.json");
    let schema_text =
        std::fs::read_to_string(&schema_path).expect("Failed to read budgets.schema.json");

    let budgets_json: serde_json::Value =
        serde_json::from_str(&budgets_text).expect("budgets.json must be valid JSON");
    let schema_json: serde_json::Value =
        serde_json::from_str(&schema_text).expect("budgets.schema.json must be valid JSON");

    let compiled = jsonschema::JSONSchema::compile(&schema_json)
        .expect("Failed to compile budgets.schema.json");
    if let Err(errors) = compiled.validate(&budgets_json) {
        let mut msgs = Vec::new();
        for e in errors {
            msgs.push(format!("{} at {}", e, e.instance_path));
        }
        panic!(
            "budgets.json failed schema validation:\n{}",
            msgs.join("\n")
        );
    }

    // policy sanity check (must have at least one enforcement path)
    let policy = budgets_json
        .get("policy")
        .and_then(|v| v.as_object())
        .expect("policy must be object");
    let warn_in_pr = policy
        .get("warn_in_pr")
        .and_then(|v| v.as_bool())
        .expect("warn_in_pr must be bool");
    let error_on_main = policy
        .get("error_on_main")
        .and_then(|v| v.as_bool())
        .expect("error_on_main must be bool");
    assert!(
        warn_in_pr || error_on_main,
        "budgets policy must enforce at least one path (warn_in_pr or error_on_main)"
    );

    // 4) dataset_id 整合（budgets.json ⊆ manifest.json）
    let datasets = budgets_json
        .get("datasets")
        .and_then(|v| v.as_array())
        .expect("budgets.json.datasets must be array");

    assert!(
        !datasets.is_empty(),
        "budgets.json.datasets must not be empty"
    );

    for entry in datasets {
        let id = entry
            .get("dataset_id")
            .and_then(|v| v.as_str())
            .expect("dataset_id must be string");
        assert!(
            dataset_ids.contains(id),
            "budgets.json dataset_id not found in tests/datasets/manifest.json: {}",
            id
        );

        // 5) 追加の妥当性チェック（schema だけでは拾いにくい “極端値” を抑止）
        let b = entry
            .get("budgets")
            .and_then(|v| v.as_object())
            .expect("budgets must be object");

        let must_pos_int = |k: &str, min: i64, max: i64| {
            let v = b
                .get(k)
                .unwrap_or_else(|| panic!("missing budget key: {k}"));
            let n = v
                .as_i64()
                .unwrap_or_else(|| panic!("budget {k} must be integer"));
            assert!(n >= min, "budget {k} too small: {n} < {min}");
            assert!(n <= max, "budget {k} too large: {n} > {max}");
        };

        // “0禁止” を強制（契約）
        must_pos_int("open_p95_ms", 1, 600000);
        must_pos_int("save_p95_ms", 1, 600000);
        must_pos_int("io_import_p95_ms", 1, 600000);
        must_pos_int("io_export_p95_ms", 1, 600000);
        must_pos_int("render_frame_p95_ms", 1, 600000);
        must_pos_int("max_rss_mb", 32, 65536);

        let temp = b
            .get("max_temp_bytes_mb")
            .and_then(|v| v.as_i64())
            .expect("max_temp_bytes_mb must be integer");
        assert!(temp >= 0, "max_temp_bytes_mb must be >= 0");
        assert!(temp <= 65536, "max_temp_bytes_mb must be <= 65536");
    }
}

fn read_text_path(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_else(|e| panic!("failed to read {}: {e}", p.display()))
}

fn parse_retention_numbers(md: &str) -> (u64, u64, u64) {
    fn extract_u64(md: &str, key: &str) -> u64 {
        for line in md.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix(key) {
                let rest = rest.trim();
                if let Some(num) = rest.strip_prefix(':') {
                    let num = num.trim().split_whitespace().next().unwrap_or("");
                    return num
                        .parse::<u64>()
                        .unwrap_or_else(|e| panic!("invalid {key} value '{num}': {e}"));
                }
            }
        }
        panic!("retention key not found: {key}");
    }

    let keep_days = extract_u64(md, "- default_keep_days");
    let max_total_bytes = extract_u64(md, "- max_total_bytes");
    let max_items = extract_u64(md, "- max_items");
    (keep_days, max_total_bytes, max_items)
}

#[test]
fn ssot_diagnostics_contracts_exist_and_valid() {
    let root = repo_root_from_manifest();
    let dir = root.join("docs").join("specs").join("diagnostics");
    assert!(
        dir.exists(),
        "missing diagnostics ssot dir: {}",
        dir.display()
    );

    let required = [
        "README.md",
        "privacy.md",
        "retention_policy.md",
        "repro_template.md",
        "support_zip.md",
        "joblog.schema.json",
        "oplog.schema.json",
    ];
    for f in required {
        let p = dir.join(f);
        assert!(
            p.exists(),
            "missing required diagnostics spec file: {}",
            p.display()
        );
    }

    let joblog_schema = read_json(&dir.join("joblog.schema.json"));
    let oplog_schema = read_json(&dir.join("oplog.schema.json"));

    let joblog_compiled = jsonschema::JSONSchema::compile(&joblog_schema)
        .unwrap_or_else(|e| panic!("joblog.schema.json is not a valid JSON Schema: {e:?}"));
    let empty = serde_json::json!({});
    assert!(
        joblog_compiled.validate(&empty).is_err(),
        "joblog schema unexpectedly validates empty object (required fields missing)"
    );

    let oplog_compiled = jsonschema::JSONSchema::compile(&oplog_schema)
        .unwrap_or_else(|e| panic!("oplog.schema.json is not a valid JSON Schema: {e:?}"));
    assert!(
        oplog_compiled.validate(&empty).is_err(),
        "oplog schema unexpectedly validates empty object (required fields missing)"
    );

    let retention = read_text_path(&dir.join("retention_policy.md"));
    let (keep_days, max_total_bytes, max_items) = parse_retention_numbers(&retention);

    assert!(
        (1..=365).contains(&keep_days),
        "default_keep_days out of range: {keep_days}"
    );
    let min_bytes: u64 = 64 * 1024 * 1024;
    let max_bytes: u64 = 64 * 1024 * 1024 * 1024;
    assert!(
        (min_bytes..=max_bytes).contains(&max_total_bytes),
        "max_total_bytes out of range: {max_total_bytes}"
    );
    assert!(
        (1..=1000).contains(&max_items),
        "max_items out of range: {max_items}"
    );

    let zip_md = read_text_path(&dir.join("support_zip.md"));
    assert!(
        zip_md.contains("joblog.json"),
        "missing required marker in support_zip.md: needle=joblog.json"
    );
    assert!(
        zip_md.contains("reason_summary.json"),
        "missing required marker in support_zip.md: needle=reason_summary.json"
    );
    assert!(
        zip_md.contains("ssot_fingerprint.json"),
        "missing required marker in support_zip.md: needle=ssot_fingerprint.json"
    );
}
