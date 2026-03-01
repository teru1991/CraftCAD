use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;
use zip::write::FileOptions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticOptions {
    pub include_doc: bool,
    pub include_system: bool,
    pub max_logs: usize,
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default)]
    pub eps: Option<Value>,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default)]
    pub settings_digest: Option<String>,
    #[serde(default)]
    pub nesting_limits: Option<Value>,
    #[serde(default)]
    pub reason_logs: Vec<String>,
}

impl Default for DiagnosticOptions {
    fn default() -> Self {
        Self {
            include_doc: false,
            include_system: false,
            max_logs: 50,
            locale: Some("ja-JP".into()),
            eps: None,
            seed: None,
            settings_digest: None,
            nesting_limits: None,
            reason_logs: vec![],
        }
    }
}

pub fn build_diagnostic_pack(
    doc_json: &str,
    options: &DiagnosticOptions,
) -> Result<Vec<u8>, String> {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(cursor);
    let file_opts = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut write_json = |path: &str, value: &Value| -> Result<(), String> {
        zip.start_file(path, file_opts).map_err(|e| e.to_string())?;
        let s = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
        zip.write_all(&s).map_err(|e| e.to_string())?;
        Ok(())
    };

    let env = serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "app_version": env!("CARGO_PKG_VERSION"),
        "git_hash": option_env!("GITHUB_SHA").unwrap_or("unknown"),
    });
    write_json("env.json", &env)?;

    if options.include_system {
        write_json(
            "system.json",
            &serde_json::json!({
                "os": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
                "cpu_count": std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
            }),
        )?;
    }

    write_json(
        "replay.json",
        &serde_json::json!({
            "locale": options.locale,
            "seed": options.seed,
            "eps": options.eps,
            "settings_digest": options.settings_digest,
            "nesting_limits": options.nesting_limits,
        }),
    )?;

    let mut logs = options.reason_logs.clone();
    if logs.len() > options.max_logs {
        logs = logs[logs.len() - options.max_logs..].to_vec();
    }
    write_json(
        "reason_logs/latest.json",
        &serde_json::json!({"count": logs.len(), "items": logs}),
    )?;

    if options.include_doc {
        zip.start_file("snapshot/document.json", file_opts)
            .map_err(|e| e.to_string())?;
        zip.write_all(doc_json.as_bytes())
            .map_err(|e| e.to_string())?;
    }

    let cursor = zip.finish().map_err(|e| e.to_string())?;
    Ok(cursor.into_inner())
}
