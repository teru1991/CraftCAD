use craftcad_determinism_harness::{fixture_ssot, run_check, CheckResult};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn load_ssot_from_args(args: &[String]) -> Result<craftcad_ssot::SsotV1, String> {
    if args.len() >= 3 && args[1] == "--project" {
        let project = diycad_project::load(Path::new(&args[2])).map_err(|e| e.to_string())?;
        return project
            .ssot_v1
            .ok_or_else(|| "project has no ssot_v1".to_string());
    }
    Ok(fixture_ssot())
}

fn rustc_version() -> String {
    Command::new("rustc")
        .arg("-V")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "rustc unknown".to_string())
        .trim()
        .to_string()
}

fn git_sha() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string()
}

fn write_repro_bundle(ssot: &craftcad_ssot::SsotV1, result: &CheckResult) -> Option<PathBuf> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis();
    let dir = PathBuf::from("build")
        .join("determinism_failures")
        .join(ts.to_string());
    fs::create_dir_all(&dir).ok()?;

    let ssot_json = serde_json::to_vec_pretty(ssot).ok()?;
    fs::write(dir.join("input_ssot.json"), ssot_json).ok()?;

    let hashes_json = serde_json::to_vec_pretty(&result.runs).ok()?;
    fs::write(dir.join("hashes.json"), hashes_json).ok()?;

    let env_text = format!("{}\ngit_sha={}\n", rustc_version(), git_sha());
    fs::write(dir.join("environment.txt"), env_text).ok()?;
    Some(dir)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ssot = match load_ssot_from_args(&args) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("determinism_check failed to load input: {e}");
            std::process::exit(2);
        }
    };

    let result = match run_check(&ssot, 3) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("determinism_check compute failed: {e}");
            std::process::exit(2);
        }
    };

    println!(
        "{}",
        serde_json::to_string(&result.summary).expect("summary serialize must not fail")
    );

    if !result.summary.ok {
        if let Some(path) = write_repro_bundle(&ssot, &result) {
            eprintln!("determinism mismatch repro bundle: {}", path.display());
        }
        std::process::exit(1);
    }
}
