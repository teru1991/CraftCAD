use super::mode_state::*;
use super::spec::{load_mode_policy, ModePolicySsot};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModeEvent {
    KeyEsc,
    KeyEnter,
    ShortcutUndo,
    ShortcutRedo,
    ToolSelectDraw,
    ToolSelectNest,
    ToolSelectExport,
    JobStarted,
    JobFinished,
    JobCancelled,
    DialogOpenConfirmDiscardDirty,
    DialogClose,
    FocusTextInput(bool),
}

impl ModeEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModeEvent::KeyEsc => "KeyEsc",
            ModeEvent::KeyEnter => "KeyEnter",
            ModeEvent::ShortcutUndo => "ShortcutUndo",
            ModeEvent::ShortcutRedo => "ShortcutRedo",
            ModeEvent::ToolSelectDraw => "ToolSelectDraw",
            ModeEvent::ToolSelectNest => "ToolSelectNest",
            ModeEvent::ToolSelectExport => "ToolSelectExport",
            ModeEvent::JobStarted => "JobStarted",
            ModeEvent::JobFinished => "JobFinished",
            ModeEvent::JobCancelled => "JobCancelled",
            ModeEvent::DialogOpenConfirmDiscardDirty => "OpenDialog",
            ModeEvent::DialogClose => "CloseDialog",
            ModeEvent::FocusTextInput(_) => "FocusTextInput",
        }
    }
}

#[derive(Debug, Clone)]
pub enum SideEffect {
    CancelPreview,
    CommitTool,
    Undo,
    Redo,
    BeginTool(ActiveTool),
    OpenPanelNest,
    OpenPanelExport,
    ShowConfirmDiscardDirty,
    SetDialog(DialogKind),
}

#[derive(Debug, Clone)]
pub struct TransitionResult {
    pub new_state: ModeState,
    pub effects: Vec<SideEffect>,
    pub denied: Option<DeniedTransition>,
}

#[derive(Debug, Clone)]
pub struct DeniedTransition {
    pub reason_code: String,
    pub required_actions: Vec<String>,
}

pub struct TransitionTable {
    pol: ModePolicySsot,
}

impl TransitionTable {
    pub fn load_default() -> Result<Self, String> {
        let pol = load_mode_policy(&super::spec::SsotPaths::default())?;
        Ok(Self { pol })
    }

    pub fn apply(&self, state: &ModeState, ev: ModeEvent) -> TransitionResult {
        if state.focus_in_text_input {
            match ev {
                ModeEvent::KeyEsc | ModeEvent::FocusTextInput(_) => {}
                _ => {
                    return TransitionResult {
                        new_state: state.clone(),
                        effects: vec![],
                        denied: None,
                    };
                }
            }
        }

        if state.job_running && matches!(ev, ModeEvent::ToolSelectDraw) {
            return TransitionResult {
                new_state: state.clone(),
                effects: vec![],
                denied: Some(DeniedTransition {
                    reason_code: "MODE_DENIED_JOB_RUNNING".to_string(),
                    required_actions: vec![
                        "CancelActiveJob".to_string(),
                        "ShowJobProgress".to_string(),
                    ],
                }),
            };
        }

        if self.pol.dialogs.confirm_on_discard_dirty
            && state.dirty
            && matches!(ev, ModeEvent::KeyEsc)
            && !matches!(state.dialog, DialogKind::ConfirmDiscardDirty)
        {
            let mut ns = state.clone();
            ns.dialog = DialogKind::ConfirmDiscardDirty;
            return TransitionResult {
                new_state: ns,
                effects: vec![SideEffect::ShowConfirmDiscardDirty],
                denied: None,
            };
        }

        if matches!(ev, ModeEvent::DialogClose) {
            let mut ns = state.clone();
            ns.dialog = DialogKind::None;
            return TransitionResult {
                new_state: ns,
                effects: vec![SideEffect::SetDialog(DialogKind::None)],
                denied: None,
            };
        }

        if let ModeEvent::FocusTextInput(b) = ev {
            let mut ns = state.clone();
            ns.focus_in_text_input = b;
            return TransitionResult {
                new_state: ns,
                effects: vec![],
                denied: None,
            };
        }

        let from = state.current_mode.as_str().to_string();
        let evs = ev.as_str().to_string();
        if let Some(t) = self
            .pol
            .transitions
            .iter()
            .find(|t| t.from == from && t.event == evs)
        {
            return self.apply_ssot_transition(state, &ev, t);
        }
        if let Some(t) = self
            .pol
            .transitions
            .iter()
            .find(|t| t.from == "*" && t.event == evs)
        {
            return self.apply_ssot_transition(state, &ev, t);
        }

        TransitionResult {
            new_state: state.clone(),
            effects: vec![],
            denied: None,
        }
    }

    fn apply_ssot_transition(
        &self,
        state: &ModeState,
        ev: &ModeEvent,
        t: &super::spec::ModeTransition,
    ) -> TransitionResult {
        let mut ns = state.clone();
        if t.to != "*" {
            if let Some(m) = Mode::from_str(&t.to) {
                ns.current_mode = m;
            }
        }

        let mut effects = vec![];
        for se in &t.side_effects {
            match se.as_str() {
                "CancelPreview" => effects.push(SideEffect::CancelPreview),
                "CommitTool" => effects.push(SideEffect::CommitTool),
                "Undo" => effects.push(SideEffect::Undo),
                "Redo" => effects.push(SideEffect::Redo),
                "BeginTool" => {
                    let tool = match ev {
                        ModeEvent::ToolSelectDraw => ActiveTool::DrawLine,
                        _ => ActiveTool::None,
                    };
                    ns.active_tool = tool.clone();
                    effects.push(SideEffect::BeginTool(tool));
                }
                "OpenPanelNest" => effects.push(SideEffect::OpenPanelNest),
                "OpenPanelExport" => effects.push(SideEffect::OpenPanelExport),
                _ => {}
            }
        }

        TransitionResult {
            new_state: ns,
            effects,
            denied: None,
        }
    }
}
