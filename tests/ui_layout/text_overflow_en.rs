#[test]
fn long_en_text_layout_placeholder() {
    let s = "This is a very long English text ".repeat(20);
    assert!(s.len() > 100);
}
