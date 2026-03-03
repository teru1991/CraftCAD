use crate::args::Args;
use anyhow::{anyhow, Context, Result};
use diycad_format::{open_package, OpenOptions};

pub fn run_dry_run(args: &Args) -> Result<()> {
    let input = args
        .input
        .as_ref()
        .ok_or_else(|| anyhow!("input is required"))?;
    let open = open_package(input, OpenOptions::default()).with_context(|| "open failed")?;

    println!("read_only={}", open.read_only);
    if let Some(m) = &open.manifest {
        println!("schema_version={}", m.schema_version);
    } else {
        println!("schema_version=<missing>");
    }

    if let Some(rep) = &open.migrate_report {
        println!("migrate: {} -> {}", rep.overall_from, rep.overall_to);
        for s in &rep.steps {
            println!(" step: {} -> {}", s.bump.from, s.bump.to);
            for p in &s.changes.added {
                println!("  added {}", p);
            }
            for p in &s.changes.removed {
                println!("  removed {}", p);
            }
            for p in &s.changes.changed {
                println!("  changed {}", p);
            }
        }
    } else {
        println!("migrate: <none>");
    }

    let mut warnings = open.warnings;
    warnings.sort_by(|a, b| {
        (
            a.path.as_deref().unwrap_or(""),
            a.code,
            format!("{:?}", a.kind),
        )
            .cmp(&(
                b.path.as_deref().unwrap_or(""),
                b.code,
                format!("{:?}", b.kind),
            ))
    });
    for w in &warnings {
        println!("warn {} {:?} {:?}", w.code.as_str(), w.path, w.kind);
    }

    Ok(())
}
