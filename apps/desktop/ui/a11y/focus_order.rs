use super::focus_manager::{FocusChain, FocusId};

/// PR4: 画面ごとのフォーカス順序を固定する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusArea {
    Menu,
    Toolbar,
    Canvas,
    SidePanel,
    StatusBar,
    DialogContent,
    DialogActions,
}

#[derive(Debug, Clone)]
pub struct FocusPolicy {
    pub name: &'static str,
    pub order: &'static [FocusArea],
    pub trap_in_modal: bool,
}

pub const PROJECT_SCREEN_FOCUS: FocusPolicy = FocusPolicy {
    name: "project_screen",
    order: &[
        FocusArea::Menu,
        FocusArea::Toolbar,
        FocusArea::Canvas,
        FocusArea::SidePanel,
        FocusArea::StatusBar,
    ],
    trap_in_modal: false,
};

pub const SUPPORT_DIALOG_FOCUS: FocusPolicy = FocusPolicy {
    name: "support_dialog",
    order: &[FocusArea::DialogContent, FocusArea::DialogActions],
    trap_in_modal: true,
};

pub const SETTINGS_DIALOG_FOCUS: FocusPolicy = FocusPolicy {
    name: "settings_dialog",
    order: &[FocusArea::DialogContent, FocusArea::DialogActions],
    trap_in_modal: true,
};

pub fn project_focus_chain() -> FocusChain {
    FocusChain {
        name: "project",
        ids: vec![
            FocusId("menu"),
            FocusId("toolbar"),
            FocusId("canvas"),
            FocusId("side_panel"),
            FocusId("status_bar"),
        ],
        trap: false,
    }
}

pub fn support_focus_chain() -> FocusChain {
    FocusChain {
        name: "support_dialog",
        ids: vec![FocusId("dialog_content"), FocusId("dialog_actions")],
        trap: true,
    }
}

pub fn settings_focus_chain() -> FocusChain {
    FocusChain {
        name: "settings_dialog",
        ids: vec![FocusId("settings_content"), FocusId("settings_actions")],
        trap: true,
    }
}
