use crate::a11y::accessibility::{A11yProps, A11yRole};
use crate::a11y::focus_manager::{FocusId, FocusManager};
use crate::a11y::focus_order::support_focus_chain;

pub struct SupportDialog {
    pub focus: FocusManager,
}

impl SupportDialog {
    pub fn new() -> Self {
        Self {
            focus: FocusManager::new(support_focus_chain()),
        }
    }

    pub fn on_tab(&mut self, reverse: bool) -> Option<FocusId> {
        self.focus.on_tab(reverse)
    }

    pub fn dialog_a11y() -> A11yProps {
        A11yProps::new(A11yRole::Dialog, "UI.DIALOG.SUPPORT.TITLE")
    }
}
