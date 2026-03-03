use crate::args::{Args, ToVersion};
use anyhow::{anyhow, Context, Result};
use diycad_format::{open_package, save_package, OpenOptions, SaveOptions};

pub fn resolve_target(to: &ToVersion) -> i64 {
    match to {
        ToVersion::Latest => 1,
        ToVersion::V1 => 1,
        ToVersion::V2 => 2,
    }
}

pub fn run_migrate(args: &Args) -> Result<()> {
    let input = args
        .input
        .as_ref()
        .ok_or_else(|| anyhow!("input is required"))?;
    let output = args
        .output
        .as_ref()
        .ok_or_else(|| anyhow!("--output is required"))?;

    let open = open_package(input.as_path(), OpenOptions::default())
        .with_context(|| format!("open failed: {}", input.display()))?;

    let mut man = open
        .manifest
        .clone()
        .ok_or_else(|| anyhow!("manifest missing; cannot migrate"))?;
    let target = resolve_target(&args.to);

    if man.schema_version > target {
        return Err(anyhow!(
            "input schema_version {} is newer than target {}",
            man.schema_version,
            target
        ));
    }

    man.schema_version = target;
    save_package(
        output.as_path(),
        SaveOptions::default(),
        man,
        open.document,
        open.parts_loaded,
        open.nest_jobs_loaded,
        vec![],
    )
    .with_context(|| format!("save failed: {}", output.display()))?;

    Ok(())
}
