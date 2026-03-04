mod shortcuts {
    include!("../shortcuts.rs");
}

use shortcuts::{check_shortcuts_table, format_conflicts};

#[test]
fn shortcuts_table_has_no_conflicts() {
  let check = check_shortcuts_table();
  let msg = format_conflicts(&check);
  assert!(msg.is_none(), "shortcut conflicts detected:\n{}", msg.unwrap());
}
