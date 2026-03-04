mod focus_manager {
    include!("../../apps/desktop/ui/a11y/focus_manager.rs");
}

use focus_manager::{FocusChain, FocusId, FocusManager};

#[test]
fn focus_manager_tabs_in_order_and_traps_when_modal() {
    let mut fm = FocusManager::new(FocusChain {
        name: "dialog",
        ids: vec![FocusId("a"), FocusId("b"), FocusId("c")],
        trap: true,
    });
    assert_eq!(fm.focus_first().unwrap().0, "a");
    assert_eq!(fm.on_tab(false).unwrap().0, "b");
    assert_eq!(fm.on_tab(false).unwrap().0, "c");
    assert_eq!(fm.on_tab(false).unwrap().0, "a");
    assert_eq!(fm.on_tab(true).unwrap().0, "c");
}

#[test]
fn focus_manager_non_modal_does_not_wrap() {
    let mut fm = FocusManager::new(FocusChain {
        name: "project",
        ids: vec![FocusId("menu"), FocusId("toolbar")],
        trap: false,
    });
    fm.focus_first();
    assert_eq!(fm.on_tab(false).unwrap().0, "toolbar");
    assert_eq!(fm.on_tab(false).unwrap().0, "toolbar");
}
