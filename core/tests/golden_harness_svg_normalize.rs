#[path = "../src/testing/golden_harness.rs"]
mod golden_harness;

use golden_harness::normalize_svg;

#[test]
fn normalize_svg_whitespace_and_attr_order() {
    let svg_a = r#"<svg width="10" height="10" xmlns="http://www.w3.org/2000/svg"><rect stroke="black" width="8" height="8" x="1" y="1" fill="none" stroke-width="0.1000001"/></svg>"#;
    let svg_b = r#"
        <svg xmlns="http://www.w3.org/2000/svg"  height="10"   width="10" >
          <rect  y="1" x="1" height="8" width="8" stroke-width="0.1" stroke="black" fill="none" />
        </svg>
    "#;

    let norm_a = normalize_svg(svg_a, 0.0001);
    let norm_b = normalize_svg(svg_b, 0.0001);

    assert_eq!(norm_a, norm_b);
}
