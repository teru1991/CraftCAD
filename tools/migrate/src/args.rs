use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
pub enum ToVersion {
    Latest,
    #[clap(name = "1")]
    V1,
    #[clap(name = "2")]
    V2,
}

#[derive(Debug, Parser)]
#[command(name = "diycad-migrate", about = "DIYCAD project migration tool")]
pub struct Args {
    /// Input .diycad path (omit when using --batch)
    pub input: Option<PathBuf>,

    /// Target schema version
    #[arg(long, value_enum, default_value = "latest")]
    pub to: ToVersion,

    /// Output .diycad path
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Dry-run: show report only (no output)
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Verify after migrate (schema validate + integrity)
    #[arg(long, default_value_t = false)]
    pub verify: bool,

    /// Print stable diff summary
    #[arg(long, default_value_t = false)]
    pub diff: bool,

    /// Batch directory mode
    #[arg(long)]
    pub batch: Option<PathBuf>,

    /// Output directory for batch
    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    /// JSON summary output path (batch only)
    #[arg(long)]
    pub json_summary: Option<PathBuf>,

    /// Disabled for operational safety. Kept only for explicit hard-fail guidance.
    #[arg(long, default_value_t = false, hide = true)]
    pub in_place: bool,

    /// Disabled safety override placeholder (not implemented in Step6)
    #[arg(long, default_value_t = false, hide = true)]
    pub i_understand_in_place_risk: bool,

    /// Disabled safety override placeholder (not implemented in Step6)
    #[arg(long, default_value_t = false, hide = true)]
    pub confirm_in_place: bool,
}
