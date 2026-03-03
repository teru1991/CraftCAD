use core::testing::compat_harness::{run_template_case, CompatCase, CompatKind};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let core_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    core_dir.parent().expect("core parent").to_path_buf()
}

#[test]
fn compat_templates_n_minus_2() {
    let root = repo_root();
    std::env::set_var("CRAFTCAD_FAILURE_ARTIFACTS_DIR", root.join("failure_artifacts"));

    let cases = [CompatCase {
        id: "template_n2_shelf",
        kind: CompatKind::TemplateJson,
        rel_path: "tests/compat/templates/n-2_shelf_template.json",
    }];

    for c in &cases {
        if let Err(e) = run_template_case(&root, c) {
            panic!("{}", e);
        }
    }
}
