use std::path::PathBuf;

#[path = "../../../src/testing/compat_harness.rs"]
mod compat_harness;

use compat_harness::{run_template_case, CompatCase, CompatKind};

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
fn compat_templates_n_minus_2() {
    let root = repo_root();
    std::env::set_var(
        "CRAFTCAD_FAILURE_ARTIFACTS_DIR",
        root.join("failure_artifacts"),
    );

    let c = CompatCase {
        id: "template_n2_shelf",
        kind: CompatKind::TemplateJson,
        rel_path: "tests/compat/templates/n-2_shelf_template.json",
    };
    run_template_case(&root, &c).unwrap_or_else(|e| panic!("{}", e));
}
