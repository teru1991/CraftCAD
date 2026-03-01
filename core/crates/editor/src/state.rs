#[derive(Clone, Debug)]
pub enum Mode {
    Select,
    DrawLine,
    DrawCircle,
    DrawArc,
    Offset,
    Trim,
    Extend,
    Move,
    Rotate,
    Scale,
}

pub struct EditorState {
    pub mode: Mode,
    pub selection: Vec<String>,
    pub cursor_world: craftcad_geom2d::Pt,
    pub snap_enabled: bool,
    pub grid_enabled: bool,
    pub angle_snap_enabled: bool,
    pub input_buffer: String,
}
