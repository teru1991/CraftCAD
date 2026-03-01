use std::collections::HashSet;

#[test]
fn shortcuts_have_no_duplicates() {
    let shortcuts = ["Ctrl+O", "Ctrl+S", "Ctrl+Z", "Ctrl+Shift+Z", "F"];
    let uniq: HashSet<_> = shortcuts.iter().collect();
    assert_eq!(uniq.len(), shortcuts.len());
}
