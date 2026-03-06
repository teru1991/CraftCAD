#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMeta {
    pub id: String,
    pub seed: u64,
    pub epsilon: f64,
    pub round_step: f64,
    pub ordering_tag: String,
    pub limits_ref: String,
    pub inputs: Vec<InputRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputRef {
    pub kind: String,
    pub path: String,
    #[serde(default)]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone)]
pub enum CompareKind {
    JsonStruct,
    SvgHash,
    BytesHash,
    ReasonCodes,
}

#[derive(Debug, Clone)]
pub enum OrderingPolicy {
    Strict,
    SortAllArrays,
    StableTagged(String),
}

#[derive(Debug, Clone)]
pub struct ExpectedEntry {
    pub compare: CompareKind,
    pub expected_path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum ActualData {
    Json(Value),
    Text(String),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct GoldenMismatch {
    pub kind: CompareKind,
    pub message: String,
    pub expected_path: PathBuf,
    pub actual_path_opt: Option<PathBuf>,
    pub diff_path_opt: Option<PathBuf>,
}

impl fmt::Display for GoldenMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Golden mismatch ({:?}): {}\nexpected_path={}",
            self.kind,
            self.message,
            self.expected_path.display()
        )?;
        if let Some(actual) = &self.actual_path_opt {
            write!(f, "\nactual_path={}", actual.display())?;
        }
        if let Some(diff) = &self.diff_path_opt {
            write!(f, "\ndiff_path={}", diff.display())?;
        }
        Ok(())
    }
}

impl std::error::Error for GoldenMismatch {}

fn repro_context(meta: &DatasetMeta) -> String {
    let input_sha = meta
        .inputs
        .iter()
        .map(|i| {
            format!(
                "{}:{}:{}",
                i.kind,
                i.path,
                i.sha256.as_deref().unwrap_or("-")
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "dataset_id={} seed={} eps={} round={} ordering_tag={} limits_ref={} input_sha=[{}]",
        meta.id,
        meta.seed,
        meta.epsilon,
        meta.round_step,
        meta.ordering_tag,
        meta.limits_ref,
        input_sha
    )
}

pub fn normalize_json(
    value: Value,
    round_step: f64,
    ordering: OrderingPolicy,
) -> Result<Value, String> {
    if !(round_step.is_finite() && round_step > 0.0) {
        return Err("round_step must be finite and > 0".to_string());
    }
    normalize_json_inner(value, round_step, &ordering)
}

fn normalize_json_inner(
    value: Value,
    round_step: f64,
    ordering: &OrderingPolicy,
) -> Result<Value, String> {
    match value {
        Value::Null | Value::Bool(_) | Value::String(_) => Ok(value),
        Value::Number(n) => {
            if let Some(v) = n.as_f64() {
                if !v.is_finite() {
                    return Err("json includes NaN/Inf".to_string());
                }
                let q = quantize_f64(v, round_step);
                let num = serde_json::Number::from_f64(q)
                    .ok_or_else(|| "failed to encode finite f64".to_string())?;
                Ok(Value::Number(num))
            } else {
                Ok(Value::Number(n))
            }
        }
        Value::Array(arr) => {
            let mut out = Vec::with_capacity(arr.len());
            for v in arr {
                out.push(normalize_json_inner(v, round_step, ordering)?);
            }
            if matches!(ordering, OrderingPolicy::SortAllArrays) {
                out.sort_by_key(stable_json_repr);
            }
            Ok(Value::Array(out))
        }
        Value::Object(map) => {
            let mut ordered: BTreeMap<String, Value> = BTreeMap::new();
            for (k, v) in map {
                ordered.insert(k, normalize_json_inner(v, round_step, ordering)?);
            }
            let mut out = serde_json::Map::new();
            for (k, v) in ordered {
                out.insert(k, v);
            }
            Ok(Value::Object(out))
        }
    }
}

fn stable_json_repr(v: &Value) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "null".to_string())
}

fn quantize_f64(v: f64, step: f64) -> f64 {
    let q = (v / step).round() * step;
    if q == 0.0 {
        0.0
    } else {
        q
    }
}

pub fn diff_json(expected: &Value, actual: &Value) -> String {
    let exp = serde_json::to_string_pretty(expected)
        .unwrap_or_else(|_| "<expected pretty print failed>".to_string());
    let act = serde_json::to_string_pretty(actual)
        .unwrap_or_else(|_| "<actual pretty print failed>".to_string());
    diff_text(&exp, &act)
}

pub fn diff_text(expected: &str, actual: &str) -> String {
    let exp: Vec<&str> = expected.lines().collect();
    let act: Vec<&str> = actual.lines().collect();
    let mut out = String::from("--- expected\n+++ actual\n");

    let max = exp.len().max(act.len());
    for i in 0..max {
        match (exp.get(i), act.get(i)) {
            (Some(x), Some(y)) if x == y => {}
            (Some(x), Some(y)) => {
                out.push_str(&format!("-{}\n+{}\n", x, y));
            }
            (Some(x), None) => out.push_str(&format!("-{}\n", x)),
            (None, Some(y)) => out.push_str(&format!("+{}\n", y)),
            (None, None) => {}
        }
    }

    out
}

pub fn normalize_svg(text: &str, round_step: f64) -> String {
    let ws = normalize_whitespace(text);
    match normalize_svg_tags(&ws, round_step) {
        Ok(v) => v,
        Err(_) => ws,
    }
}

fn normalize_whitespace(text: &str) -> String {
    let mut out = String::new();
    let mut prev_space = false;

    for ch in text.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }

    out.replace("> <", "><").trim().to_string()
}

fn normalize_svg_tags(text: &str, round_step: f64) -> Result<String, ()> {
    let mut out = String::new();
    let mut i = 0usize;
    let bytes = text.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'<' {
            let start = i;
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != b'>' {
                j += 1;
            }
            if j >= bytes.len() {
                return Err(());
            }
            let tag = &text[start..=j];
            out.push_str(&normalize_one_tag(tag, round_step)?);
            i = j + 1;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }

    Ok(out)
}

fn normalize_one_tag(tag: &str, round_step: f64) -> Result<String, ()> {
    if !tag.starts_with('<') || !tag.ends_with('>') {
        return Err(());
    }
    if tag.starts_with("<!--") || tag.starts_with("<!") || tag.starts_with("<?") {
        return Ok(tag.to_string());
    }
    if tag.starts_with("</") {
        return Ok(tag.to_string());
    }

    let inner = &tag[1..tag.len() - 1];
    let mut parts = inner.splitn(2, char::is_whitespace);
    let name = parts.next().ok_or(())?;
    let rest = parts.next().unwrap_or("");

    let self_close = rest.trim_end().ends_with('/');
    let attrs_src = if self_close {
        rest.trim_end().trim_end_matches('/').trim()
    } else {
        rest.trim()
    };

    let attrs = parse_attrs(attrs_src)?;

    let mut out = String::new();
    out.push('<');
    out.push_str(name);
    for (k, v) in attrs {
        out.push(' ');
        out.push_str(&k);
        out.push_str("=\"");
        out.push_str(&normalize_attr_value(&v, round_step));
        out.push('"');
    }
    if self_close {
        out.push_str("/>");
    } else {
        out.push('>');
    }

    Ok(out)
}

fn parse_attrs(src: &str) -> Result<BTreeMap<String, String>, ()> {
    let mut map = BTreeMap::new();
    let bytes = src.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }

        let key_start = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'=' {
            i += 1;
        }
        let key = src[key_start..i].to_string();

        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b'=' {
            map.insert(key, String::new());
            continue;
        }
        i += 1;

        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b'"' {
            return Err(());
        }
        i += 1;

        let val_start = i;
        while i < bytes.len() && bytes[i] != b'"' {
            i += 1;
        }
        if i >= bytes.len() {
            return Err(());
        }

        let value = src[val_start..i].to_string();
        i += 1;

        map.insert(key, value);
    }

    Ok(map)
}

fn normalize_attr_value(value: &str, round_step: f64) -> String {
    let mut out = String::new();
    let mut token = String::new();

    let flush = |token: &mut String, out: &mut String| {
        if token.is_empty() {
            return;
        }
        if let Ok(v) = token.parse::<f64>() {
            if v.is_finite() {
                let quantized = quantize_f64(v, round_step);
                let formatted = trim_float_string(&format!("{:.8}", quantized));
                out.push_str(&formatted);
            } else {
                out.push_str(token);
            }
        } else {
            out.push_str(token);
        }
        token.clear();
    };

    for ch in value.chars() {
        if ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+' | 'e' | 'E') {
            token.push(ch);
        } else {
            flush(&mut token, &mut out);
            out.push(ch);
        }
    }
    flush(&mut token, &mut out);

    out
}

fn trim_float_string(s: &str) -> String {
    let mut out = s.to_string();
    if out.contains('.') {
        while out.ends_with('0') {
            out.pop();
        }
        if out.ends_with('.') {
            out.push('0');
        }
    }
    if out == "-0" || out == "-0.0" {
        "0.0".to_string()
    } else {
        out
    }
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn canonical_reason_codes(warnings_json: &Value) -> Vec<String> {
    let mut codes = BTreeSet::new();

    if let Some(obj) = warnings_json.as_object() {
        if let Some(arr) = obj.get("codes").and_then(|v| v.as_array()) {
            for code in arr {
                if let Some(s) = code.as_str() {
                    codes.insert(s.to_string());
                }
            }
        }

        if let Some(arr) = obj.get("warnings").and_then(|v| v.as_array()) {
            for warning in arr {
                if let Some(s) = warning.get("code").and_then(|v| v.as_str()) {
                    codes.insert(s.to_string());
                }
                if let Some(s) = warning
                    .get("reason")
                    .and_then(|v| v.get("code"))
                    .and_then(|v| v.as_str())
                {
                    codes.insert(s.to_string());
                }
            }
        }
    }

    codes.into_iter().collect()
}

pub fn artifacts_base_dir(repo_root: &Path) -> PathBuf {
    if let Ok(value) = std::env::var("CRAFTCAD_FAILURE_ARTIFACTS_DIR") {
        PathBuf::from(value)
    } else {
        repo_root.join("failure_artifacts")
    }
}

pub fn write_failure_artifacts(
    repo_root: &Path,
    dataset_meta: &DatasetMeta,
    expected_path: &Path,
    actual_name: &str,
    actual_bytes: &[u8],
    diff_text_value: &str,
    reason_codes: Option<&[String]>,
) -> Result<PathBuf, std::io::Error> {
    let base = artifacts_base_dir(repo_root);
    let out_dir = base.join(&dataset_meta.id);
    fs::create_dir_all(&out_dir)?;

    fs::write(
        out_dir.join("meta.json"),
        serde_json::to_vec_pretty(dataset_meta).unwrap_or_else(|_| b"{}".to_vec()),
    )?;

    if expected_path.exists() {
        let expected_out = if expected_path.extension().and_then(|s| s.to_str()) == Some("txt") {
            out_dir.join("expected.txt")
        } else {
            out_dir.join("expected.bin")
        };
        let _ = fs::copy(expected_path, expected_out);
    }

    fs::write(out_dir.join(actual_name), actual_bytes)?;
    fs::write(out_dir.join("diff.txt"), diff_text_value.as_bytes())?;

    if let Some(codes) = reason_codes {
        let payload = serde_json::json!({ "codes": codes });
        fs::write(
            out_dir.join("warnings_codes.json"),
            serde_json::to_vec_pretty(&payload).unwrap_or_else(|_| b"{}".to_vec()),
        )?;
    }

    Ok(out_dir)
}

pub fn compare_expected(
    repo_root: &Path,
    dataset_meta: &DatasetMeta,
    expected_entry: &ExpectedEntry,
    actual_data: ActualData,
) -> Result<(), GoldenMismatch> {
    match (&expected_entry.compare, actual_data) {
        (CompareKind::JsonStruct, ActualData::Json(value)) => compare_json_struct(
            repo_root,
            dataset_meta,
            &expected_entry.expected_path,
            value,
        ),
        (CompareKind::SvgHash, ActualData::Text(value)) => compare_svg_hash(
            repo_root,
            dataset_meta,
            &expected_entry.expected_path,
            &value,
        ),
        (CompareKind::BytesHash, ActualData::Bytes(value)) => compare_bytes_hash(
            repo_root,
            dataset_meta,
            &expected_entry.expected_path,
            &value,
        ),
        (CompareKind::ReasonCodes, ActualData::Json(value)) => compare_reason_codes(
            repo_root,
            dataset_meta,
            &expected_entry.expected_path,
            value,
        ),
        (kind, _) => Err(GoldenMismatch {
            kind: kind.clone(),
            message: "actual payload type does not match compare mode".to_string(),
            expected_path: expected_entry.expected_path.clone(),
            actual_path_opt: None,
            diff_path_opt: None,
        }),
    }
}

pub fn compare_json_struct(
    repo_root: &Path,
    dataset_meta: &DatasetMeta,
    expected_path: &Path,
    actual_value: Value,
) -> Result<(), GoldenMismatch> {
    let expected_bytes = fs::read(expected_path).map_err(|e| GoldenMismatch {
        kind: CompareKind::JsonStruct,
        message: format!(
            "{} failed to read expected json: {}",
            repro_context(dataset_meta),
            e
        ),
        expected_path: expected_path.to_path_buf(),
        actual_path_opt: None,
        diff_path_opt: None,
    })?;

    let expected_value: Value =
        serde_json::from_slice(&expected_bytes).map_err(|e| GoldenMismatch {
            kind: CompareKind::JsonStruct,
            message: format!(
                "{} expected json parse error: {}",
                repro_context(dataset_meta),
                e
            ),
            expected_path: expected_path.to_path_buf(),
            actual_path_opt: None,
            diff_path_opt: None,
        })?;

    let expected_norm = normalize_json(
        expected_value,
        dataset_meta.round_step,
        OrderingPolicy::Strict,
    )
    .map_err(|e| GoldenMismatch {
        kind: CompareKind::JsonStruct,
        message: format!(
            "{} normalize expected failed: {}",
            repro_context(dataset_meta),
            e
        ),
        expected_path: expected_path.to_path_buf(),
        actual_path_opt: None,
        diff_path_opt: None,
    })?;

    let actual_norm = normalize_json(
        actual_value,
        dataset_meta.round_step,
        OrderingPolicy::Strict,
    )
    .map_err(|e| GoldenMismatch {
        kind: CompareKind::JsonStruct,
        message: format!(
            "{} normalize actual failed: {}",
            repro_context(dataset_meta),
            e
        ),
        expected_path: expected_path.to_path_buf(),
        actual_path_opt: None,
        diff_path_opt: None,
    })?;

    if expected_norm != actual_norm {
        let diff = diff_json(&expected_norm, &actual_norm);
        let actual_bytes =
            serde_json::to_vec_pretty(&actual_norm).unwrap_or_else(|_| b"{}".to_vec());
        let out_dir = write_failure_artifacts(
            repo_root,
            dataset_meta,
            expected_path,
            "actual.json",
            &actual_bytes,
            &diff,
            None,
        )
        .unwrap_or_else(|_| artifacts_base_dir(repo_root));

        return Err(GoldenMismatch {
            kind: CompareKind::JsonStruct,
            message: format!("{} json_struct mismatch", repro_context(dataset_meta)),
            expected_path: expected_path.to_path_buf(),
            actual_path_opt: Some(out_dir.join("actual.json")),
            diff_path_opt: Some(out_dir.join("diff.txt")),
        });
    }

    Ok(())
}

pub fn compare_reason_codes(
    repo_root: &Path,
    dataset_meta: &DatasetMeta,
    expected_path: &Path,
    actual_warnings_json: Value,
) -> Result<(), GoldenMismatch> {
    let expected_bytes = fs::read(expected_path).map_err(|e| GoldenMismatch {
        kind: CompareKind::ReasonCodes,
        message: format!(
            "dataset_id={} failed to read expected warnings: {}",
            dataset_meta.id, e
        ),
        expected_path: expected_path.to_path_buf(),
        actual_path_opt: None,
        diff_path_opt: None,
    })?;

    let expected_value: Value =
        serde_json::from_slice(&expected_bytes).map_err(|e| GoldenMismatch {
            kind: CompareKind::ReasonCodes,
            message: format!(
                "{} expected warnings parse error: {}",
                repro_context(dataset_meta),
                e
            ),
            expected_path: expected_path.to_path_buf(),
            actual_path_opt: None,
            diff_path_opt: None,
        })?;

    let expected_codes = canonical_reason_codes(&expected_value);
    let actual_codes = canonical_reason_codes(&actual_warnings_json);

    if expected_codes != actual_codes {
        let expected_codes_json = serde_json::json!({ "codes": expected_codes });
        let actual_codes_json = serde_json::json!({ "codes": actual_codes.clone() });

        let diff = diff_text(
            &serde_json::to_string_pretty(&expected_codes_json)
                .unwrap_or_else(|_| "{}".to_string()),
            &serde_json::to_string_pretty(&actual_codes_json).unwrap_or_else(|_| "{}".to_string()),
        );
        let actual_bytes =
            serde_json::to_vec_pretty(&actual_codes_json).unwrap_or_else(|_| b"{}".to_vec());

        let out_dir = write_failure_artifacts(
            repo_root,
            dataset_meta,
            expected_path,
            "actual_reason_codes.json",
            &actual_bytes,
            &diff,
            Some(&actual_codes),
        )
        .unwrap_or_else(|_| artifacts_base_dir(repo_root));

        return Err(GoldenMismatch {
            kind: CompareKind::ReasonCodes,
            message: format!("{} reason_codes mismatch", repro_context(dataset_meta)),
            expected_path: expected_path.to_path_buf(),
            actual_path_opt: Some(out_dir.join("actual_reason_codes.json")),
            diff_path_opt: Some(out_dir.join("diff.txt")),
        });
    }

    Ok(())
}

pub fn compare_svg_hash(
    repo_root: &Path,
    dataset_meta: &DatasetMeta,
    expected_path: &Path,
    actual_svg_text: &str,
) -> Result<(), GoldenMismatch> {
    let expected_text = fs::read_to_string(expected_path).map_err(|e| GoldenMismatch {
        kind: CompareKind::SvgHash,
        message: format!(
            "{} failed to read expected svg: {}",
            repro_context(dataset_meta),
            e
        ),
        expected_path: expected_path.to_path_buf(),
        actual_path_opt: None,
        diff_path_opt: None,
    })?;

    let expected_norm = normalize_svg(&expected_text, dataset_meta.round_step);
    let actual_norm = normalize_svg(actual_svg_text, dataset_meta.round_step);

    if expected_norm != actual_norm {
        let diff = diff_text(&expected_norm, &actual_norm);
        let out_dir = write_failure_artifacts(
            repo_root,
            dataset_meta,
            expected_path,
            "actual.svg",
            actual_norm.as_bytes(),
            &diff,
            None,
        )
        .unwrap_or_else(|_| artifacts_base_dir(repo_root));

        return Err(GoldenMismatch {
            kind: CompareKind::SvgHash,
            message: format!("{} svg mismatch", repro_context(dataset_meta)),
            expected_path: expected_path.to_path_buf(),
            actual_path_opt: Some(out_dir.join("actual.svg")),
            diff_path_opt: Some(out_dir.join("diff.txt")),
        });
    }

    Ok(())
}

pub fn compare_bytes_hash(
    repo_root: &Path,
    dataset_meta: &DatasetMeta,
    expected_path: &Path,
    actual_bytes: &[u8],
) -> Result<(), GoldenMismatch> {
    let expected_bytes = fs::read(expected_path).map_err(|e| GoldenMismatch {
        kind: CompareKind::BytesHash,
        message: format!(
            "{} failed to read expected bytes: {}",
            repro_context(dataset_meta),
            e
        ),
        expected_path: expected_path.to_path_buf(),
        actual_path_opt: None,
        diff_path_opt: None,
    })?;

    let expected_hash = hash_bytes(&expected_bytes);
    let actual_hash = hash_bytes(actual_bytes);

    if expected_hash != actual_hash {
        let diff = diff_text(
            &format!("expected_sha256={expected_hash}"),
            &format!("actual_sha256={actual_hash}"),
        );
        let out_dir = write_failure_artifacts(
            repo_root,
            dataset_meta,
            expected_path,
            "actual.bin",
            actual_bytes,
            &diff,
            None,
        )
        .unwrap_or_else(|_| artifacts_base_dir(repo_root));

        return Err(GoldenMismatch {
            kind: CompareKind::BytesHash,
            message: format!("{} bytes_hash mismatch", repro_context(dataset_meta)),
            expected_path: expected_path.to_path_buf(),
            actual_path_opt: Some(out_dir.join("actual.bin")),
            diff_path_opt: Some(out_dir.join("diff.txt")),
        });
    }

    Ok(())
}
