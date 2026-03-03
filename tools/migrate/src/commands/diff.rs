use crate::args::Args;
use anyhow::{anyhow, Context, Result};
use diycad_format::{open_package, OpenOptions};
use serde_json::Value;
use std::collections::BTreeSet;

fn collect_top_keys(v: &Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if let Some(m) = v.as_object() {
        for k in m.keys() {
            out.insert(format!("/{}", k));
        }
    }
    out
}

pub fn run_diff(args: &Args) -> Result<()> {
    let input = args
        .input
        .as_ref()
        .ok_or_else(|| anyhow!("input is required"))?;
    let open = open_package(input, OpenOptions::default()).with_context(|| "open failed")?;
    let man_v = serde_json::to_value(
        open.manifest
            .as_ref()
            .ok_or_else(|| anyhow!("manifest missing"))?,
    )?;
    let doc_v = serde_json::to_value(&open.document)?;

    println!("[manifest keys]");
    for k in collect_top_keys(&man_v) {
        println!("{}", k);
    }
    println!("[document keys]");
    for k in collect_top_keys(&doc_v) {
        println!("{}", k);
    }

    println!("[counts]");
    println!("parts_loaded={}", open.parts_loaded.len());
    println!("parts_failed={}", open.parts_failed.len());
    println!("nest_jobs_loaded={}", open.nest_jobs_loaded.len());
    println!("nest_jobs_failed={}", open.nest_jobs_failed.len());
    Ok(())
}
