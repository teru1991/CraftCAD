use std::path::PathBuf;

#[path = "../../../src/testing/compat_harness.rs"]
mod compat_harness;

use compat_harness::{run_io_dxf_ascii_case, run_io_svg_case, CompatCase, CompatKind};

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
fn compat_io_import() {
    let root = repo_root();
    std::env::set_var(
        "CRAFTCAD_FAILURE_ARTIFACTS_DIR",
        root.join("failure_artifacts"),
    );

    let svg = CompatCase {
        id: "io_svg_external_rect",
        kind: CompatKind::IoSvg,
        rel_path: "tests/compat/io/external_rect.svg",
    };
    run_io_svg_case(&root, &svg).unwrap_or_else(|e| panic!("{}", e));

    let dxf_path = root.join("tests/compat/io/external_rect_ascii.dxf");
    if dxf_path.exists() {
        let c = CompatCase {
            id: "io_dxf_external_rect_ascii",
            kind: CompatKind::IoDxfAscii,
            rel_path: "tests/compat/io/external_rect_ascii.dxf",
        };
        run_io_dxf_ascii_case(&root, &c).unwrap_or_else(|e| panic!("{}", e));
    }
}
