use crate::a11y::accessibility::{A11yProps, A11yRole};
use crate::a11y::focus_manager::{FocusId, FocusManager};
use crate::a11y::focus_order::settings_focus_chain;
use crate::settings::{UiFontSize, UiLanguage, UiScale, UiSettings, UiUnitSystem};

pub struct SettingsScreen {
    pub focus: FocusManager,
    pub draft: UiSettings,
}

impl SettingsScreen {
    pub fn new(current: UiSettings) -> Self {
        Self {
            focus: FocusManager::new(settings_focus_chain()),
            draft: current,
        }
    }

    pub fn on_tab(&mut self, reverse: bool) -> Option<FocusId> {
        self.focus.on_tab(reverse)
    }

    pub fn set_language(&mut self, language: UiLanguage) {
        self.draft.language = language;
    }

    pub fn set_unit_system(&mut self, unit_system: UiUnitSystem) {
        self.draft.unit_system = unit_system;
    }

    pub fn set_ui_scale(&mut self, scale: u16) {
        self.draft.ui_scale = UiScale(scale);
    }

    pub fn set_font_size(&mut self, font_size: UiFontSize) {
        self.draft.font_size = font_size;
    }

    pub fn language_label_a11y() -> A11yProps {
        A11yProps::new(A11yRole::TextField, "UI.SETTINGS.LANGUAGE")
    }

    pub fn unit_label_a11y() -> A11yProps {
        A11yProps::new(A11yRole::TextField, "UI.SETTINGS.UNIT_SYSTEM")
    }
}
