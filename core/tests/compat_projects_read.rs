use core::testing::compat_harness::{run_project_json_case, CompatCase, CompatKind};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let core_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    core_dir.parent().expect("core parent").to_path_buf()
}

#[test]
fn compat_projects_n_minus_2() {
    let root = repo_root();
    std::env::set_var("CRAFTCAD_FAILURE_ARTIFACTS_DIR", root.join("failure_artifacts"));

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
        if let Err(e) = run_project_json_case(&root, c) {
            panic!("{}", e);
        }
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
