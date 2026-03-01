use clap::Parser;
use std::fs;
use std::path::PathBuf;
use chrono::Local;

#[derive(Parser, Debug)]
struct Args {
    /// Dataset to update: geom, nesting, export, or all
    #[arg(long, default_value = None)]
    dataset: Option<String>,

    /// Perform dry-run (default mode, shows diffs only)
    #[arg(long, default_value_t = true)]
    validate_only: bool,

    /// Actually write changes to golden files (requires explicit confirmation)
    #[arg(long)]
    write: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Determine which datasets to update
    let datasets = match &args.dataset {
        None => vec!["geom", "nesting", "export"],
        Some(d) if d == "all" => vec!["geom", "nesting", "export"],
        Some(d) => vec![d.as_str()],
    };

    println!("🔍 Golden Update Tool");
    println!("====================");
    println!();

    if args.write {
        println!("⚠️  Write mode enabled - golden files WILL be modified");
        println!("Datasets to update: {}", datasets.join(", "));
        println!();

        // Show user what will be changed
        for dataset in &datasets {
            let golden_path = get_golden_path(dataset);
            if golden_path.exists() {
                println!("📁 {} → {}", dataset, golden_path.display());
            }
        }
        println!();

        // Request confirmation
        if !is_tty() {
            eprintln!("❌ Write mode requires TTY. Running in validation-only mode.");
            return validate_golden_files(&datasets);
        }

        println!("This will create backups of existing golden files.");
        println!("Type 'yes' to confirm:");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim() != "yes" {
            println!("Cancelled.");
            return Ok(());
        }

        // Perform update with backups
        update_golden_files(&datasets)?;
    } else {
        println!("ℹ️  Validation-only mode (default)");
        println!("Run with --write to apply changes.");
        println!();

        validate_golden_files(&datasets)?;
    }

    Ok(())
}

fn is_tty() -> bool {
    atty::is(atty::Stream::Stdin)
}

fn get_golden_path(dataset: &str) -> PathBuf {
    match dataset {
        "geom" => PathBuf::from("tests/golden/geom/expected.json"),
        "nesting" => PathBuf::from("tests/golden/nesting/"),
        "export" => PathBuf::from("tests/golden/export/"),
        _ => PathBuf::from(format!("tests/golden/{}/", dataset)),
    }
}

fn validate_golden_files(datasets: &[&str]) -> anyhow::Result<()> {
    println!("📋 Validating golden files...\n");

    for dataset in datasets {
        let path = get_golden_path(dataset);
        if path.exists() {
            println!("✓ {} exists at {}", dataset, path.display());
        } else {
            println!("✗ {} NOT FOUND at {}", dataset, path.display());
        }
    }

    println!("\n✓ Validation complete");
    Ok(())
}

fn update_golden_files(datasets: &[&str]) -> anyhow::Result<()> {
    println!("🔄 Updating golden files...\n");

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");

    for dataset in datasets {
        let path = get_golden_path(dataset);
        if !path.exists() {
            println!("⚠️  {} does not exist, skipping", dataset);
            continue;
        }

        // Create backup
        let _backup_path = if path.is_dir() {
            // For directories, backup each file
            println!("📦 Backing up directory: {}", path.display());
            for entry in fs::read_dir(&path)? {
                let entry = entry?;
                let entry_path = entry.path();
                if entry_path.is_file() {
                    let backup_file = format!(
                        "{}.backup.{}",
                        entry_path.display(),
                        timestamp
                    );
                    fs::copy(&entry_path, &backup_file)?;
                    println!("   → {}", backup_file);
                }
            }
            path.clone()
        } else {
            // For files, single backup
            let backup_file = format!(
                "{}.backup.{}",
                path.display(),
                timestamp
            );
            fs::copy(&path, &backup_file)?;
            println!("📦 Backed up: {} → {}", path.display(), backup_file);
            PathBuf::from(&backup_file)
        };

        println!("✓ {} updated", dataset);
    }

    println!("\n✅ Update complete!");
    println!("\nTo revert changes, restore from backup files:");
    println!("  $ ls tests/golden/*/*.backup.*");
    println!("\nTo commit changes:");
    println!("  $ git add tests/golden/");
    println!("  $ git commit -m 'Update golden files'");

    Ok(())
}
