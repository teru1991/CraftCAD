use ssot_lint::{run_ssot_lint, LintConfig};
use std::path::{Path, PathBuf};

fn detect_repo_root(start: &Path) -> PathBuf {
    let mut cur = start.to_path_buf();
    loop {
        if cur.join("docs/specs/schema/diycad").exists() {
            return cur;
        }
        if !cur.pop() {
            return start.to_path_buf();
        }
    }
}

fn main() {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let repo_root = detect_repo_root(&cwd);
    let cfg = LintConfig { repo_root };
    match run_ssot_lint(&cfg) {
        Ok(findings) => {
            if findings.is_empty() {
                eprintln!("ssot-lint: OK");
                std::process::exit(0);
            }
            eprintln!("ssot-lint: FAILED ({} findings)", findings.len());
            for f in findings {
                eprintln!("- {}: {}", f.code, f.message);
            }
            std::process::exit(2);
        }
        Err(e) => {
            eprintln!("ssot-lint: ERROR: {:#}", e);
            std::process::exit(3);
        }
    }
}
