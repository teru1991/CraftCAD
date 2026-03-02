use craftcad_io::options::ImportOptions;
use craftcad_io::reasons::ReasonCode;
use craftcad_io::IoEngine;
use craftcad_io_svg::SvgIo;

#[test]
fn svg_respects_limits_max_entities() {
    let svg = r#"<svg>
      <g><path d="M0 0 L1 1"/></g>
      <g><path d="M0 0 L1 1"/></g>
      <g><path d="M0 0 L1 1"/></g>
      <g><path d="M0 0 L1 1"/></g>
      <g><path d="M0 0 L1 1"/></g>
      <g><path d="M0 0 L1 1"/></g>
    </svg>"#;

    let eng = IoEngine::new().register_importer(Box::new(SvgIo::new()));
    let mut opts = ImportOptions::default_for_tests();
    opts.limits.max_entities = 5;

    let err = eng.import("svg", svg.as_bytes(), &opts).unwrap_err();
    assert_eq!(err.reason, ReasonCode::IO_SVG_LIMIT_NODES_EXCEEDED);
}

#[test]
fn svg_blocks_external_reference_href() {
    let svg = r#"<svg><image href="http://example.com/x.png" /></svg>"#;

    let eng = IoEngine::new().register_importer(Box::new(SvgIo::new()));
    let opts = ImportOptions::default_for_tests();

    let res = eng.import("svg", svg.as_bytes(), &opts).unwrap();
    assert!(
        res.warnings
            .iter()
            .any(|w| w.reason == ReasonCode::IO_IMAGE_REFERENCE_DROPPED),
        "expected IO_IMAGE_REFERENCE_DROPPED warning"
    );
}
