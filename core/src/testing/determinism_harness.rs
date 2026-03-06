#![allow(dead_code)]
use crate::golden_harness::{
    canonical_reason_codes, hash_bytes, normalize_json, normalize_svg, DatasetMeta, OrderingPolicy,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Fingerprint {
    pub model_sha256: String,
    pub warnings_codes: Vec<String>,
    #[serde(default)]
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct DeterminismFailure {
    pub dataset_id: String,
    pub run_index: usize,
    pub expected: Fingerprint,
    pub actual: Fingerprint,
    pub artifacts_dir: PathBuf,
}

fn artifacts_base_dir(repo_root: &Path) -> PathBuf {
    if let Ok(p) = std::env::var("CRAFTCAD_FAILURE_ARTIFACTS_DIR") {
        return PathBuf::from(p).join("determinism");
    }
    repo_root.join("failure_artifacts").join("determinism")
}

pub fn fingerprint_from_outputs(
    meta: &DatasetMeta,
    normalized_model: &Value,
    warnings_json: &Value,
    exported_svg_opt: Option<&str>,
    exported_json_opt: Option<&Value>,
) -> Fingerprint {
    let norm = normalize_json(
        normalized_model.clone(),
        meta.round_step,
        OrderingPolicy::Strict,
    )
    .unwrap_or_else(|_| Value::String("__NORM_FAIL__".to_string()));
    let bytes = serde_json::to_vec(&norm).unwrap_or_else(|_| b"{}".to_vec());
    let model_sha256 = hash_bytes(&bytes);

    let warnings_codes = canonical_reason_codes(warnings_json);

    let mut extra = BTreeMap::new();
    if let Some(svg) = exported_svg_opt {
        let s = normalize_svg(svg, meta.round_step);
        extra.insert("exported_svg_sha256".to_string(), hash_bytes(s.as_bytes()));
    }
    if let Some(j) = exported_json_opt {
        let jn = normalize_json(j.clone(), meta.round_step, OrderingPolicy::Strict)
            .unwrap_or_else(|_| Value::String("__NORM_FAIL__".to_string()));
        let jb = serde_json::to_vec(&jn).unwrap_or_else(|_| b"{}".to_vec());
        extra.insert("exported_json_sha256".to_string(), hash_bytes(&jb));
    }

    Fingerprint {
        model_sha256,
        warnings_codes,
        extra,
    }
}

#[allow(clippy::result_large_err)]
pub fn assert_deterministic<F>(
    repo_root: &Path,
    meta: &DatasetMeta,
    runs: usize,
    mut runner: F,
) -> Result<(), DeterminismFailure>
where
    F: FnMut() -> (Value, Value, Option<String>, Option<Value>),
{
    let (m0, w0, svg0, j0) = runner();
    let expected_fp = fingerprint_from_outputs(meta, &m0, &w0, svg0.as_deref(), j0.as_ref());

    for i in 1..runs {
        let (m, w, svg, j) = runner();
        let actual_fp = fingerprint_from_outputs(meta, &m, &w, svg.as_deref(), j.as_ref());
        if expected_fp != actual_fp {
            let dir = write_failure(repo_root, meta, i, &expected_fp, &actual_fp, &m, &w);
            return Err(DeterminismFailure {
                dataset_id: meta.id.clone(),
                run_index: i,
                expected: expected_fp,
                actual: actual_fp,
                artifacts_dir: dir,
            });
        }
    }
    Ok(())
}

fn write_failure(
    repo_root: &Path,
    meta: &DatasetMeta,
    run_index: usize,
    expected: &Fingerprint,
    actual: &Fingerprint,
    model_json: &Value,
    warnings_json: &Value,
) -> PathBuf {
    let base = artifacts_base_dir(repo_root);
    let dir = base.join(&meta.id);
    let _ = fs::create_dir_all(&dir);

    let meta_json = serde_json::json!({
        "dataset_id": meta.id,
        "seed": meta.seed,
        "epsilon": meta.epsilon,
        "round_step": meta.round_step,
        "ordering_tag": meta.ordering_tag,
        "limits_ref": meta.limits_ref,
        "runs": run_index + 1,
        "binary_free": true
    });
    let _ = fs::write(
        dir.join("meta.json"),
        serde_json::to_vec_pretty(&meta_json).unwrap_or_else(|_| b"{}".to_vec()),
    );

    let _ = fs::write(
        dir.join("expected_fingerprint.json"),
        serde_json::to_vec_pretty(expected).unwrap_or_else(|_| b"{}".to_vec()),
    );
    let _ = fs::write(
        dir.join(format!("actual_fingerprint_run_{}.json", run_index)),
        serde_json::to_vec_pretty(actual).unwrap_or_else(|_| b"{}".to_vec()),
    );

    let norm_model = normalize_json(model_json.clone(), meta.round_step, OrderingPolicy::Strict)
        .unwrap_or_else(|_| Value::String("__NORM_FAIL__".to_string()));
    let _ = fs::write(
        dir.join(format!("actual_model_run_{}.json", run_index)),
        serde_json::to_vec_pretty(&norm_model).unwrap_or_else(|_| b"{}".to_vec()),
    );

    let codes = canonical_reason_codes(warnings_json);
    let _ = fs::write(
        dir.join(format!("reason_codes_run_{}.json", run_index)),
        serde_json::to_vec_pretty(&serde_json::json!({ "codes": codes }))
            .unwrap_or_else(|_| b"{}".to_vec()),
    );

    let diff = format!(
        "expected: {}\nactual: {}\n",
        serde_json::to_string_pretty(expected).unwrap_or_default(),
        serde_json::to_string_pretty(actual).unwrap_or_default()
    );
    let _ = fs::write(dir.join("diff.txt"), diff.as_bytes());

    dir
}
