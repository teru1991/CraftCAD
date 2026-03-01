#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusSlot { Toolbar, Canvas, Inspector, Timeline, Status }

pub const DEFAULT_TAB_ORDER: &[FocusSlot] = &[
    FocusSlot::Toolbar,
    FocusSlot::Canvas,
    FocusSlot::Inspector,
    FocusSlot::Timeline,
    FocusSlot::Status,
];
