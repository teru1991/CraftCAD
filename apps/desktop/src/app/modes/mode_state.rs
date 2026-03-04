use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Select,
    Draw,
    Edit,
    Dimension,
    Annotate,
    Nest,
    Export,
    Settings,
    Support,
}

impl Mode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Select" => Some(Self::Select),
            "Draw" => Some(Self::Draw),
            "Edit" => Some(Self::Edit),
            "Dimension" => Some(Self::Dimension),
            "Annotate" => Some(Self::Annotate),
            "Nest" => Some(Self::Nest),
            "Export" => Some(Self::Export),
            "Settings" => Some(Self::Settings),
            "Support" => Some(Self::Support),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Select => "Select",
            Mode::Draw => "Draw",
            Mode::Edit => "Edit",
            Mode::Dimension => "Dimension",
            Mode::Annotate => "Annotate",
            Mode::Nest => "Nest",
            Mode::Export => "Export",
            Mode::Settings => "Settings",
            Mode::Support => "Support",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SelectionState {
    pub selected_entity_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ActiveTool {
    #[default]
    None,
    DrawLine,
    DrawRect,
    DimensionLinear,
    AnnotateLeader,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DialogKind {
    #[default]
    None,
    ConfirmDiscardDirty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeState {
    pub current_mode: Mode,
    pub selection: SelectionState,
    pub active_tool: ActiveTool,
    pub job_running: bool,
    pub dirty: bool,
    pub dialog: DialogKind,
    pub focus_in_text_input: bool,
}

impl Default for ModeState {
    fn default() -> Self {
        Self {
            current_mode: Mode::Select,
            selection: SelectionState::default(),
            active_tool: ActiveTool::default(),
            job_running: false,
            dirty: false,
            dialog: DialogKind::default(),
            focus_in_text_input: false,
        }
    }
}
