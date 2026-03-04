use super::*;

#[test]
fn backstack_respects_cap() {
    let mut b = backstack::Backstack::new(3, true);
    b.push("a".to_string());
    b.push("b".to_string());
    b.push("c".to_string());
    b.push("d".to_string());
    assert_eq!(b.len(), 3);
    assert_eq!(b.pop().unwrap(), "d");
}
