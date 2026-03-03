use crate::args::Args;
use crate::report_json::{stable_json_value, BatchSummary, FileSummary};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn run_batch(args: &Args) -> Result<()> {
    let dir = args
        .batch
        .as_ref()
        .ok_or_else(|| anyhow!("--batch DIR is required"))?;
    let outdir = args
        .output_dir
        .as_ref()
        .ok_or_else(|| anyhow!("--output-dir is required"))?;
    fs::create_dir_all(outdir)?;

    let mut files: Vec<PathBuf> = Vec::new();
    for e in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if e.file_type().is_file()
            && e.path().extension().and_then(|x| x.to_str()) == Some("diycad")
        {
            files.push(e.path().to_path_buf());
        }
    }
    files.sort();

    let mut summaries: Vec<FileSummary> = Vec::new();
    let mut ok = 0u64;
    let mut failed = 0u64;

    for inp in files {
        let rel = inp
            .strip_prefix(dir)
            .unwrap_or(&inp)
            .to_string_lossy()
            .to_string();
        let outp = outdir.join(Path::new(&rel).file_name().unwrap_or_default());
        let mut s = FileSummary {
            input: inp.to_string_lossy().to_string(),
            output: Some(outp.to_string_lossy().to_string()),
            ok: false,
            reason: None,
            warnings: vec![],
            migrate_overall_from: None,
            migrate_overall_to: None,
        };

        match crate::commands::migrate::run_migrate(&Args {
            input: Some(inp.clone()),
            output: Some(outp.clone()),
            to: args.to.clone(),
            dry_run: false,
            verify: false,
            diff: false,
            batch: None,
            output_dir: None,
            json_summary: None,
            in_place: false,
            i_understand_in_place_risk: false,
            confirm_in_place: false,
        }) {
            Ok(_) => {
                s.ok = true;
                ok += 1;
            }
            Err(e) => {
                s.ok = false;
                s.reason = Some(e.to_string());
                failed += 1;
            }
        }
        summaries.push(s);
    }

    summaries.sort_by(|a, b| a.input.cmp(&b.input));

    let mut totals = BTreeMap::new();
    totals.insert("failed".to_string(), failed);
    totals.insert("ok".to_string(), ok);

    let batch = BatchSummary {
        version: format!(
            "target={}",
            crate::commands::migrate::resolve_target(&args.to)
        ),
        totals,
        files: summaries,
    };

    if let Some(p) = &args.json_summary {
        let v = serde_json::to_value(batch)?;
        let stable = stable_json_value(&v);
        fs::write(p, serde_json::to_vec_pretty(&stable)?)?;
    } else {
        println!("ok={} failed={}", ok, failed);
    }
    Ok(())
}
