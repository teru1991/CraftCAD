use craftcad_io::model::{Entity, Segment2D};
use craftcad_io::options::ImportOptions;
use craftcad_io::IoEngine;
use craftcad_io_dxf::DxfIo;

#[test]
fn lwpolyline_bulge_produces_arc_segment() {
    let dxf = "0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n8\nCUT\n70\n0\n10\n0\n20\n0\n42\n0.414213562\n10\n10\n20\n0\n0\nENDSEC\n0\nEOF\n";

    let eng = IoEngine::new().register_importer(Box::new(DxfIo::new()));
    let mut opts = ImportOptions::default_for_tests();
    opts.enable_approx = false;
    let res = eng.import("dxf", dxf.as_bytes(), &opts).unwrap();

    let mut has_arc = false;
    for e in &res.model.entities {
        if let Entity::Path(p) = e {
            if p.segments
                .iter()
                .any(|s| matches!(s, Segment2D::Arc { .. }))
            {
                has_arc = true;
            }
        }
    }
    assert!(has_arc, "expected at least one Arc segment from bulge");
}
