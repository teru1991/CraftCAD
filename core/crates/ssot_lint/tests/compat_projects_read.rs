use std::path::PathBuf;

#[path = "../../../src/testing/compat_harness.rs"]
mod compat_harness;

use compat_harness::{run_project_json_case, CompatCase, CompatKind};

fn repo_root() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap()
        .to_path_buf()
}

#[test]
fn compat_projects_n_minus_2() {
    let root = repo_root();
    std::env::set_var(
        "CRAFTCAD_FAILURE_ARTIFACTS_DIR",
        root.join("failure_artifacts"),
    );

    let cases = [
        CompatCase {
            id: "project_n2_small",
            kind: CompatKind::ProjectJson,
            rel_path: "tests/compat/projects/n-2_small_project.json",
        },
        CompatCase {
            id: "project_n2_large",
            kind: CompatKind::ProjectJson,
            rel_path: "tests/compat/projects/n-2_large_project.json",
        },
    ];
    for c in &cases {
        run_project_json_case(&root, c).unwrap_or_else(|e| panic!("{}", e));
    }

    let forward = CompatCase {
        id: "project_forward_incompat",
        kind: CompatKind::ProjectJson,
        rel_path: "tests/compat/projects/forward_incompatible_project.json",
    };
    match run_project_json_case(&root, &forward) {
        Err(e) if e.code == "CP_FORWARD_INCOMPATIBLE" => {}
        Err(e) => panic!("unexpected forward incompat error: {}", e),
        Ok(()) => panic!("forward incompatible case must not succeed silently"),
    }
}
