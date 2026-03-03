use core::testing::compat_harness::{
    run_io_dxf_ascii_case, run_io_svg_case, CompatCase, CompatKind,
};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let core_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    core_dir.parent().expect("core parent").to_path_buf()
}

#[test]
fn compat_io_import() {
    let root = repo_root();
    std::env::set_var("CRAFTCAD_FAILURE_ARTIFACTS_DIR", root.join("failure_artifacts"));

    let svg_cases = [CompatCase {
        id: "io_svg_external_rect",
        kind: CompatKind::IoSvg,
        rel_path: "tests/compat/io/external_rect.svg",
    }];
    for c in &svg_cases {
        if let Err(e) = run_io_svg_case(&root, c) {
            panic!("{}", e);
        }
    }

    let dxf_path = root.join("tests/compat/io/external_rect_ascii.dxf");
    if dxf_path.exists() {
        let c = CompatCase {
            id: "io_dxf_external_rect_ascii",
            kind: CompatKind::IoDxfAscii,
            rel_path: "tests/compat/io/external_rect_ascii.dxf",
        };
        if let Err(e) = run_io_dxf_ascii_case(&root, &c) {
            panic!("{}", e);
        }
    }
}
