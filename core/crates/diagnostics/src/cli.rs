use crate::*;
use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::PathBuf;

type ReasonCountsMap = BTreeMap<String, (i64, Option<String>, Option<String>, Option<Severity>)>;

#[derive(Parser, Debug)]
#[command(
    name = "craftcad-diagnostics",
    version,
    about = "CraftCAD diagnostics support workflow (Sprint17)"
)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Generate support ZIP into diagnostics store (with consent flags).
    Zip {
        /// Base directory for diagnostics store (default: ./diagnostics_out)
        #[arg(long, default_value = "diagnostics_out")]
        out: PathBuf,

        /// Repo root for SSOT fingerprint (default: current dir)
        #[arg(long)]
        repo_root: Option<PathBuf>,

        /// Include project snapshot (.diycad) (default: false)
        #[arg(long, default_value_t = false)]
        include_project: bool,

        /// Project snapshot path
        #[arg(long)]
        project_path: Option<PathBuf>,

        /// Include input files copy (default: false, currently preview metadata only)
        #[arg(long, default_value_t = false)]
        include_inputs: bool,

        /// Input file metadata (repeatable): --input <zip_name>:<path>
        #[arg(long)]
        input: Vec<String>,

        /// Print preview (file list + reason top 5) and exit
        #[arg(long, default_value_t = false)]
        preview: bool,
    },

    /// Print repro markdown from given joblog/oplog (paths)
    Repro {
        #[arg(long)]
        joblog: PathBuf,
        #[arg(long)]
        oplog: Option<PathBuf>,
        /// If provided, include these artifacts metadata (zip name/sha)
        #[arg(long)]
        zip_name: Option<String>,
        #[arg(long)]
        zip_sha256: Option<String>,
    },

    /// Cleanup diagnostics store using SSOT default retention policy
    Cleanup {
        #[arg(long, default_value = "diagnostics_out")]
        out: PathBuf,
    },

    /// Delete one item by id
    Delete {
        #[arg(long, default_value = "diagnostics_out")]
        out: PathBuf,
        #[arg(long)]
        id: String,
    },

    /// Delete all items
    DeleteAll {
        #[arg(long, default_value = "diagnostics_out")]
        out: PathBuf,
    },
}

#[derive(Clone)]
struct CliConsent {
    include_project_snapshot: bool,
    include_inputs_copy: bool,
}
impl ConsentProvider for CliConsent {
    fn include_project_snapshot(&self) -> bool {
        self.include_project_snapshot
    }

    fn include_inputs_copy(&self) -> bool {
        self.include_inputs_copy
    }

    fn telemetry_opt_in(&self) -> bool {
        false
    }
}

pub fn run(args: Args) -> anyhow::Result<()> {
    match args.cmd {
        Command::Zip {
            out,
            repo_root,
            include_project,
            project_path,
            include_inputs,
            input,
            preview,
        } => {
            let store = DiagnosticsStore::new(&out)?;
            let consent = CliConsent {
                include_project_snapshot: include_project,
                include_inputs_copy: include_inputs,
            };
            let redactor = StubRedactor;
            let limits = Limits::conservative_default();
            let policy = RetentionPolicy::ssot_default();

            let joblog = build_cli_joblog(&redactor, &consent, limits.clone(), preview, &input);
            let summary = build_reason_summary(&joblog);

            if preview {
                print_preview(
                    include_project,
                    include_inputs,
                    project_path.as_ref(),
                    &input,
                    &joblog,
                    &summary,
                );
                return Ok(());
            }

            let repo_root = repo_root.unwrap_or(std::env::current_dir()?);
            let mut builder = SupportZipBuilder::new()
                .attach_joblog(joblog)
                .attach_ssot_fingerprint(SsotFingerprint::compute(&repo_root));

            if include_project {
                if let Some(pp) = project_path {
                    let snapshot = std::fs::read_to_string(pp).ok();
                    builder = builder.attach_project_snapshot(snapshot);
                }
            }

            let res = builder
                .attach_perf(craftcad_perf::PerfReport {
                    dataset_id: "cli".to_string(),
                    schema_version: None,
                    seed: Some(0),
                    spans: Vec::new(),
                    memory_peak_mb: None,
                })
                .build_into_store(&store, &repo_root, &policy)?;

            let sha256 = sha256_file(&res.path)?;
            println!("OK: support_zip={}", res.path.display());
            println!("sha256={sha256}");
            println!("size_bytes={}", res.size_bytes);
            if !res.warnings.is_empty() {
                println!("warnings={:?}", res.warnings);
            }
            if include_inputs && !input.is_empty() {
                println!(
                    "note=input copy is not embedded yet; metadata accepted for preview/workflow"
                );
            }
            Ok(())
        }
        Command::Repro {
            joblog,
            oplog,
            zip_name,
            zip_sha256,
        } => {
            let jl: JobLog = serde_json::from_slice(&std::fs::read(&joblog)?)?;
            let op = if let Some(p) = oplog {
                Some(serde_json::from_slice::<OpLog>(&std::fs::read(p)?)?)
            } else {
                None
            };
            let artifacts = match (zip_name, zip_sha256) {
                (Some(n), Some(s)) => Some(ReproArtifacts {
                    zip_name: n,
                    zip_sha256: s,
                }),
                _ => None,
            };
            let text = generate_repro_markdown(&jl, op.as_ref(), artifacts);
            print!("{}", text.markdown);
            Ok(())
        }
        Command::Cleanup { out } => {
            let store = DiagnosticsStore::new(&out)?;
            let res = store.cleanup(&RetentionPolicy::ssot_default())?;
            println!("deleted_ids={:?}", res.deleted_ids);
            if !res.warnings.is_empty() {
                println!("warnings={:?}", res.warnings);
            }
            Ok(())
        }
        Command::Delete { out, id } => {
            let store = DiagnosticsStore::new(&out)?;
            store.delete_item(&id)?;
            println!("OK deleted id={id}");
            Ok(())
        }
        Command::DeleteAll { out } => {
            let store = DiagnosticsStore::new(&out)?;
            let res = store.delete_all()?;
            println!("deleted_ids={:?}", res.deleted_ids);
            if !res.warnings.is_empty() {
                println!("warnings={:?}", res.warnings);
            }
            Ok(())
        }
    }
}

fn build_cli_joblog(
    redactor: &dyn Redactor,
    consent: &dyn ConsentProvider,
    limits: Limits,
    preview: bool,
    inputs: &[String],
) -> JobLog {
    let ctx = JobContext {
        app_version: "cli".into(),
        build_id: None,
        schema_version: "unknown".into(),
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        locale: "unknown".into(),
        timezone: "UTC".into(),
        determinism_tag: DeterminismTag {
            seed: 0,
            epsilon: 1e-6,
            rounding: "bankers".into(),
            ordering: "btree".into(),
        },
        limits_profile: "default".into(),
    };
    let mut jb = JobLogBuilder::new(ctx, redactor, consent, limits);

    for (i, raw) in inputs.iter().enumerate() {
        if let Ok((name, _path)) = parse_name_path(raw) {
            jb.add_input(
                "manual",
                &format!("input-{i}-{name}"),
                "",
                0,
                "user_provided",
            );
        }
    }

    {
        let mut g = jb.begin_step(
            "cli",
            "CreateSupportZip",
            &serde_json::json!({"preview": preview, "inputs_count": inputs.len()}),
        );
        g.set_result(StepResultKind::Ok);
    }

    jb.finish()
}

fn build_reason_summary(joblog: &JobLog) -> ReasonSummary {
    let mut counts: ReasonCountsMap = BTreeMap::new();
    for r in &joblog.reasons {
        counts.insert(
            r.code.clone(),
            (
                r.count,
                Some(r.first_ts.clone()),
                Some(r.last_ts.clone()),
                Some(r.severity),
            ),
        );
    }
    ReasonSummary::from_reason_counts_stable(&counts, &EmptyCatalogLookup, 10)
}

fn parse_name_path(s: &str) -> anyhow::Result<(String, PathBuf)> {
    let parts: Vec<&str> = s.splitn(2, ':').collect();
    if parts.len() != 2 {
        anyhow::bail!("--input must be '<zip_name>:<path>'");
    }
    Ok((parts[0].to_string(), PathBuf::from(parts[1])))
}

fn print_preview(
    include_project: bool,
    include_inputs: bool,
    project_path: Option<&PathBuf>,
    inputs: &[String],
    joblog: &JobLog,
    summary: &ReasonSummary,
) {
    println!("=== Preview (SSOT layout) ===");
    println!("- will include: joblog.json");
    println!("- will include: ssot_fingerprint.json");
    println!("- will include: consent.json");
    println!("- will include: perf_report.json");
    println!("--- optional (consent) ---");
    println!("- include_project_snapshot={include_project}");
    if include_project {
        println!("  project_path={project_path:?}");
    }
    println!("- include_inputs_copy={include_inputs}");
    if include_inputs {
        println!("  inputs={inputs:?}");
    }
    println!("--- top reasons (up to 5) ---");
    for it in summary.top_reasons.iter().take(5) {
        println!("- {} (count={})", it.code, it.count);
    }
    println!("--- joblog summary ---");
    println!("steps={}", joblog.timeline.steps.len());
}

fn sha256_file(path: &PathBuf) -> anyhow::Result<String> {
    let data = std::fs::read(path)?;
    let mut h = Sha256::new();
    h.update(&data);
    Ok(hex::encode(h.finalize()))
}
