use clap::{Parser, Subcommand};
use diycad_common::collect_basic_diagnostics;
use diycad_project::{create_empty_project, load, save, Manifest};
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "diycad-cli", disable_version_flag = true)]
struct Cli {
    /// Print version sourced from diycad_common.
    #[arg(long = "version", global = true, action = clap::ArgAction::SetTrue)]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Load and validate a .diycad file.
    Validate { path: PathBuf },
    /// Print manifest.json in pretty JSON.
    PrintManifest { path: PathBuf },
    /// Generate a minimal v0 .diycad sample file.
    GenerateSample { outpath: PathBuf },
}

fn main() {
    let cli = Cli::parse();

    if cli.version {
        let diagnostics = collect_basic_diagnostics();
        println!("{}", diagnostics.app_version);
        return;
    }

    let code = match run(cli) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err}");
            1
        }
    };

    std::process::exit(code);
}

fn run(cli: Cli) -> Result<i32, Box<dyn Error>> {
    match cli.command {
        Some(Commands::Validate { path }) => {
            let project = load(path)?;
            match validate_manifest(&project.manifest) {
                Ok(()) => {
                    println!("valid");
                    Ok(0)
                }
                Err(message) => {
                    eprintln!("invalid manifest: {message}");
                    Ok(2)
                }
            }
        }
        Some(Commands::PrintManifest { path }) => {
            let project = load(path)?;
            let manifest = serde_json::to_string_pretty(&project.manifest)?;
            println!("{manifest}");
            Ok(0)
        }
        Some(Commands::GenerateSample { outpath }) => {
            let diagnostics = collect_basic_diagnostics();
            let project = create_empty_project(&diagnostics.app_version, "mm", "2026-01-01T00:00:00Z");
            save(&outpath, &project)?;
            println!("generated {}", outpath.display());
            Ok(0)
        }
        None => {
            eprintln!("no command provided. use --help.");
            Ok(1)
        }
    }
}

fn validate_manifest(manifest: &Manifest) -> Result<(), String> {
    require_field("schema_version", &manifest.schema_version)?;
    require_field("app_version", &manifest.app_version)?;
    require_field("units", &manifest.units)?;
    require_field("created_at", &manifest.created_at)?;
    require_field("modified_at", &manifest.modified_at)?;
    Ok(())
}

fn require_field(field_name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{field_name} is missing or empty"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn validate_manifest_requires_non_empty_fields() {
        let manifest = Manifest {
            schema_version: "0".to_string(),
            app_version: "".to_string(),
            units: "mm".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            modified_at: "2026-01-01T00:00:00Z".to_string(),
        };

        let result = validate_manifest(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn generate_sample_creates_file() {
        let dir = tempdir().expect("tempdir");
        let outpath = dir.path().join("sample_v0.diycad");
        let cli = Cli {
            version: false,
            command: Some(Commands::GenerateSample {
                outpath: outpath.clone(),
            }),
        };

        let code = run(cli).expect("run should succeed");
        assert_eq!(code, 0);
        assert!(outpath.exists());
    }
}
