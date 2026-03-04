use crate::a11y::shortcut_dispatch::AppAction;
use crate::a11y::shortcuts::ShortcutScope;
use crate::settings::{I18nRuntime, UiSettings};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafeMode {
    Select,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolMode {
    Select,
    Line,
    Circle,
    Arc,
    Dimension,
    Text,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub settings: UiSettings,
    pub i18n: I18nRuntime,
    pub safe_mode: SafeMode,
    pub tool_mode: ToolMode,
    pub in_text_input: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let settings = UiSettings::default();
        let i18n = I18nRuntime::from_settings(&settings);
        Self {
            settings,
            i18n,
            safe_mode: SafeMode::Select,
            tool_mode: ToolMode::Select,
            in_text_input: false,
        }
    }
}

impl AppState {
    pub fn shortcut_scope(&self) -> ShortcutScope {
        match self.tool_mode {
            ToolMode::Select => ShortcutScope::View,
            _ => ShortcutScope::Cad,
        }
    }

    pub fn apply_action(&mut self, action: AppAction) {
        match action {
            AppAction::Cancel => {
                self.safe_mode = SafeMode::Select;
                self.tool_mode = ToolMode::Select;
            }
            AppAction::ToolLine => self.tool_mode = ToolMode::Line,
            AppAction::ToolCircle => self.tool_mode = ToolMode::Circle,
            AppAction::ToolArc => self.tool_mode = ToolMode::Arc,
            AppAction::ToolDimension => self.tool_mode = ToolMode::Dimension,
            AppAction::ToolText => self.tool_mode = ToolMode::Text,
            _ => {}
        }
    }

    pub fn update_settings(&mut self, settings: UiSettings) {
        self.settings = settings;
        self.i18n = I18nRuntime::from_settings(&self.settings);
    }
}
