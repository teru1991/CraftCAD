/// PR3では「器のみ」。PR4で画面別に具体化する。
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
  order: &[FocusArea::Menu, FocusArea::Toolbar, FocusArea::Canvas, FocusArea::SidePanel, FocusArea::StatusBar],
  trap_in_modal: false,
};

pub const SUPPORT_DIALOG_FOCUS: FocusPolicy = FocusPolicy {
  name: "support_dialog",
  order: &[FocusArea::DialogContent, FocusArea::DialogActions],
  trap_in_modal: true,
};
