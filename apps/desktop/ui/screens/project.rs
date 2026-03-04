use crate::a11y::accessibility::{A11yProps, A11yRole};
use crate::a11y::focus_manager::{FocusId, FocusManager};
use crate::a11y::focus_order::project_focus_chain;

pub struct ProjectScreen {
    pub focus: FocusManager,
}

impl ProjectScreen {
    pub fn new() -> Self {
        Self {
            focus: FocusManager::new(project_focus_chain()),
        }
    }

    pub fn on_tab(&mut self, reverse: bool) -> Option<FocusId> {
        self.focus.on_tab(reverse)
    }

    pub fn export_button_a11y() -> A11yProps {
        A11yProps::new(A11yRole::Button, "UI.BUTTON.EXPORT")
    }

    pub fn nest_button_a11y() -> A11yProps {
        A11yProps::new(A11yRole::Button, "UI.BUTTON.NEST")
    }
}
