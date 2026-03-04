use crate::a11y::accessibility::{A11yProps, A11yRole};

pub fn settings_menu_item_a11y() -> A11yProps {
    A11yProps::new(A11yRole::MenuItem, "UI.SETTINGS.LANGUAGE")
}
