use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LintConfig {
    pub repo_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintFinding {
    pub code: &'static str,
    pub message: String,
}

fn read_text(path: &Path) -> Result<String> {
    let s = fs::read_to_string(path)
        .with_context(|| format!("failed to read text: {}", path.display()))?;
    Ok(s)
}

fn read_json(path: &Path) -> Result<Value> {
    let bytes =
        fs::read(path).with_context(|| format!("failed to read file: {}", path.display()))?;
    let v: Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("invalid json: {}", path.display()))?;
    Ok(v)
}

fn require_file(findings: &mut Vec<LintFinding>, path: &Path) {
    if !path.exists() {
        findings.push(LintFinding {
            code: "SSOT_MISSING_FILE",
            message: format!("missing required SSOT file: {}", path.display()),
        });
    }
}

fn get_required_array(v: &Value, key: &str) -> Result<Vec<String>> {
    let arr = v.get(key).ok_or_else(|| anyhow!("missing key: {}", key))?;
    let arr = arr
        .as_array()
        .ok_or_else(|| anyhow!("{} must be array", key))?;
    let mut out = Vec::new();
    for x in arr {
        out.push(
            x.as_str()
                .ok_or_else(|| anyhow!("{} items must be string", key))?
                .to_string(),
        );
    }
    Ok(out)
}

fn json_has_required_keys(schema: &Value, keys: &[&str]) -> Result<()> {
    let required = get_required_array(schema, "required")?;
    for k in keys {
        if !required.iter().any(|x| x == k) {
            bail!("schema required missing: {}", k);
        }
    }
    Ok(())
}

fn extract_latest_schema_version(versions_md: &str) -> Result<i64> {
    // Expect a line: "- latest_schema_version: <int>"
    let re = Regex::new(r"(?m)^\s*-\s*latest_schema_version:\s*([0-9]+)\s*$")?;
    let caps = re
        .captures(versions_md)
        .ok_or_else(|| anyhow!("cannot find latest_schema_version in versions.md"))?;
    let n: i64 = caps.get(1).unwrap().as_str().parse()?;
    Ok(n)
}

fn format_md_has_required_paths(format_md: &str) -> Result<()> {
    // We require these literal mentions to avoid drift
    let required = [
        "/manifest.json",
        "/document.json",
        "/parts/*.json",
        "/nest_jobs/*.json",
        "/assets/",
        "/_migrations/",
    ];
    for s in required {
        if !format_md.contains(s) {
            bail!("format.md missing required path mention: {}", s);
        }
    }
    Ok(())
}

pub fn run_ssot_lint(cfg: &LintConfig) -> Result<Vec<LintFinding>> {
    let mut findings: Vec<LintFinding> = Vec::new();

    let base = cfg.repo_root.join("docs/specs/schema/diycad");
    let format_md = base.join("format.md");
    let versions_md = base.join("versions.md");
    let migration_policy_md = base.join("migration_policy.md");
    let integrity_md = base.join("integrity.md");
    let recovery_md = base.join("recovery.md");
    let manifest_schema = base.join("manifest.schema.json");
    let document_schema = base.join("document.schema.json");
    let part_schema = base.join("part.schema.json");
    let nest_job_schema = base.join("nest_job.schema.json");

    for p in [
        &format_md,
        &versions_md,
        &migration_policy_md,
        &integrity_md,
        &recovery_md,
        &manifest_schema,
        &document_schema,
        &part_schema,
        &nest_job_schema,
    ] {
        require_file(&mut findings, p);
    }
    if !findings.is_empty() {
        return Ok(findings);
    }

    // format.md required path mentions
    match read_text(&format_md).and_then(|t| format_md_has_required_paths(&t)) {
        Ok(_) => {}
        Err(e) => findings.push(LintFinding {
            code: "SSOT_FORMAT_MD_INVALID",
            message: e.to_string(),
        }),
    }

    // versions.md latest_schema_version
    let versions_text = match read_text(&versions_md) {
        Ok(t) => t,
        Err(e) => {
            findings.push(LintFinding {
                code: "SSOT_VERSIONS_READ_FAIL",
                message: e.to_string(),
            });
            return Ok(findings);
        }
    };
    let latest = match extract_latest_schema_version(&versions_text) {
        Ok(v) => v,
        Err(e) => {
            findings.push(LintFinding {
                code: "SSOT_VERSIONS_INVALID",
                message: e.to_string(),
            });
            return Ok(findings);
        }
    };
    if latest < 1 {
        findings.push(LintFinding {
            code: "SSOT_VERSION_RANGE",
            message: "latest_schema_version must be >= 1".to_string(),
        });
    }

    // migration_policy.md must mention N-2 and stepwise vN -> vN+1
    match read_text(&migration_policy_md) {
        Ok(t) => {
            if !t.contains("N-2") {
                findings.push(LintFinding {
                    code: "SSOT_MIGRATION_POLICY_MISSING_N2",
                    message: "migration_policy.md must mention N-2".to_string(),
                });
            }
            if !t.contains("vN -> vN+1") {
                findings.push(LintFinding {
                    code: "SSOT_MIGRATION_POLICY_MISSING_STEPWISE",
                    message: "migration_policy.md must mention stepwise vN -> vN+1".to_string(),
                });
            }
        }
        Err(e) => findings.push(LintFinding {
            code: "SSOT_MIGRATION_POLICY_READ_FAIL",
            message: e.to_string(),
        }),
    }

    // manifest.schema.json required keys and minimum schema_version alignment with latest
    match read_json(&manifest_schema).and_then(|v| {
        json_has_required_keys(
            &v,
            &[
                "schema_version",
                "app_version",
                "created_at",
                "updated_at",
                "unit",
                "entrypoints",
            ],
        )
        .map(|_| v)
    }) {
        Ok(v) => {
            let min = v
                .pointer("/properties/schema_version/minimum")
                .and_then(|x| x.as_i64())
                .unwrap_or(1);
            if min > latest {
                findings.push(LintFinding{
          code: "SSOT_MANIFEST_SCHEMA_MIN_GT_LATEST",
          message: format!("manifest.schema.json schema_version.minimum ({}) must be <= latest_schema_version ({})", min, latest),
        });
            }
        }
        Err(e) => findings.push(LintFinding {
            code: "SSOT_MANIFEST_SCHEMA_INVALID",
            message: e.to_string(),
        }),
    }

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn lint_accepts_minimal_valid_tree() {
        let td = tempdir().unwrap();
        let root = td.path();

        let dir = root.join("docs/specs/schema/diycad");
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(dir.join("format.md"), "/manifest.json\n/document.json\n/parts/*.json\n/nest_jobs/*.json\n/assets/\n/_migrations/\n").unwrap();
        std::fs::write(dir.join("versions.md"), "- latest_schema_version: 1\n").unwrap();
        std::fs::write(dir.join("migration_policy.md"), "N-2\nvN -> vN+1\n").unwrap();
        std::fs::write(dir.join("integrity.md"), "content_manifest\n").unwrap();
        std::fs::write(dir.join("recovery.md"), "autosave\n").unwrap();

        std::fs::write(dir.join("manifest.schema.json"), r#"{"type":"object","required":["schema_version","app_version","created_at","updated_at","unit","entrypoints"],"properties":{"schema_version":{"minimum":1}}}"#).unwrap();
        std::fs::write(dir.join("document.schema.json"), r#"{"type":"object","required":["id","name","unit","entities","parts_index","nest_jobs_index"]}"#).unwrap();
        std::fs::write(
            dir.join("part.schema.json"),
            r#"{"type":"object","required":["id","name","quantity","material","geometry"]}"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("nest_job.schema.json"),
            r#"{"type":"object","required":["id","status","inputs"]}"#,
        )
        .unwrap();

        let findings = run_ssot_lint(&LintConfig {
            repo_root: root.to_path_buf(),
        })
        .unwrap();
        assert!(findings.is_empty(), "findings: {:?}", findings);
    }
}
