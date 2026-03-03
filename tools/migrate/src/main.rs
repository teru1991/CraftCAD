mod args;
mod commands;
mod report_json;

use anyhow::{anyhow, Result};
use args::Args;
use clap::Parser;

fn main() -> Result<()> {
    let args = Args::parse();

    if args.in_place {
        return Err(anyhow!(
            "--in-place is disabled for operational safety in Step6. Use explicit --output path."
        ));
    }

    if args.batch.is_some() {
        return commands::batch::run_batch(&args);
    }
    if args.dry_run {
        return commands::dry_run::run_dry_run(&args);
    }
    if args.diff {
        return commands::diff::run_diff(&args);
    }
    if args.verify && args.output.is_none() && args.input.is_some() {
        return commands::verify::run_verify(&args);
    }

    commands::migrate::run_migrate(&args)
}
