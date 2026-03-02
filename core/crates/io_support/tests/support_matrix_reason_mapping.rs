use craftcad_io::reasons::ReasonCode;
use craftcad_io_support::SupportMatrix;

#[test]
fn test_support_matrix_reason_code_mapping_is_lossless() {
    let sm = SupportMatrix::load_from_ssot().expect("support matrix load");

    let dxf_text = sm.reasons("dxf", "entity_text", "import");
    assert!(dxf_text.contains(&ReasonCode::IO_TEXT_FALLBACK_FONT));
    assert!(dxf_text.contains(&ReasonCode::IO_FALLBACK_024));

    let dxf_spline = sm.reasons("dxf", "entity_spline", "import");
    assert!(dxf_spline.contains(&ReasonCode::IO_CURVE_APPROX_APPLIED));
    assert!(dxf_spline.contains(&ReasonCode::IO_UNSUPPORTED_ENTITY_DXF_SPLINE));

    let dxf_hatch = sm.reasons("dxf", "entity_hatch", "import");
    assert!(dxf_hatch.contains(&ReasonCode::IO_HATCH_SIMPLIFIED));

    let svg_text = sm.reasons("svg", "entity_text", "import");
    assert!(svg_text.contains(&ReasonCode::IO_TEXT_FALLBACK_FONT));

    let svg_ext = sm.reasons("svg", "external_reference", "import");
    assert!(svg_ext.contains(&ReasonCode::IO_IMAGE_REFERENCE_DROPPED));
}
