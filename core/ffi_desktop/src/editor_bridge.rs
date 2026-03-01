use craftcad_editor::state::EditorState;

pub fn editor_mode_name(state: &EditorState) -> &'static str {
    match state.mode {
        craftcad_editor::state::Mode::Select => "select",
        craftcad_editor::state::Mode::DrawLine => "line",
        craftcad_editor::state::Mode::DrawCircle => "circle",
        craftcad_editor::state::Mode::DrawArc => "arc",
        craftcad_editor::state::Mode::Offset => "offset",
        craftcad_editor::state::Mode::Trim => "trim",
        craftcad_editor::state::Mode::Extend => "extend",
        craftcad_editor::state::Mode::Move => "move",
        craftcad_editor::state::Mode::Rotate => "rotate",
        craftcad_editor::state::Mode::Scale => "scale",
    }
}
