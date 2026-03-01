#[test]
fn long_ja_text_layout_placeholder() {
    let s = "これは非常に長い日本語テキストです".repeat(20);
    assert!(s.len() > 100);
}
