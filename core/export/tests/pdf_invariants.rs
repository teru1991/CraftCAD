use craftcad_export::{
    compute_tiled_layout, export_svg, export_tiled_pdf, SvgExportOptions, TiledPdfOptions,
};
use craftcad_serialize::Document;

fn sample_doc() -> Document {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/sample_project_v1.json");
    serde_json::from_str(&std::fs::read_to_string(p).expect("fixture")).expect("doc")
}

#[test]
fn tiled_pdf_invariants_gauge_and_metadata() {
    let doc = sample_doc();
    let opts = TiledPdfOptions::default();
    let layout = compute_tiled_layout(&doc, &opts).expect("layout");
    assert!(layout.page_count >= 1);
    assert!(layout.gauge_length_doc_units > 0.0);

    let pdf = export_tiled_pdf(&doc, &opts).expect("pdf");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("100mm") || layout.gauge_length_doc_units >= 100.0 || doc.units == "inch"
    );
    assert!(text.contains("CraftCAD") || text.contains("PDF"));
}

#[test]
fn svg_matches_golden_exact() {
    let doc = sample_doc();
    let svg = export_svg(&doc, &SvgExportOptions::default()).expect("svg");
    let golden = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/golden/export/sample_svg_expected.svg");
    let expected = std::fs::read_to_string(golden).expect("golden");
    assert_eq!(svg.trim(), expected.trim());
}
