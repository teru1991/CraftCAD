use super::shortcuts::{KeyChord, ShortcutScope, SHORTCUTS};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAction {
    FileOpen,
    FileSave,
    Undo,
    Redo,
    Cancel,
    LibrarySearch,
    HelpSupport,
    ViewPanLeft,
    ViewPanRight,
    ViewPanUp,
    ViewPanDown,
    ViewZoomIn,
    ViewZoomOut,
    ToolLine,
    ToolCircle,
    ToolArc,
    ToolDimension,
    ToolText,
    JobNest,
    JobExport,
}

pub fn resolve_action(scope: ShortcutScope, chord: KeyChord) -> Option<AppAction> {
    let ch = chord.normalize();
    match (scope, ch) {
        (ShortcutScope::Global, KeyChord::ModPlus('O')) => Some(AppAction::FileOpen),
        (ShortcutScope::Global, KeyChord::ModPlus('S')) => Some(AppAction::FileSave),
        (ShortcutScope::Global, KeyChord::ModPlus('Z')) => Some(AppAction::Undo),
        (ShortcutScope::Global, KeyChord::ShiftModPlus('Z')) => Some(AppAction::Redo),
        (ShortcutScope::Global, KeyChord::Esc) => Some(AppAction::Cancel),
        (ShortcutScope::Global, KeyChord::ModPlus('F')) => Some(AppAction::LibrarySearch),
        (ShortcutScope::Global, KeyChord::F1) => Some(AppAction::HelpSupport),

        (ShortcutScope::View, KeyChord::ArrowLeft) => Some(AppAction::ViewPanLeft),
        (ShortcutScope::View, KeyChord::ArrowRight) => Some(AppAction::ViewPanRight),
        (ShortcutScope::View, KeyChord::ArrowUp) => Some(AppAction::ViewPanUp),
        (ShortcutScope::View, KeyChord::ArrowDown) => Some(AppAction::ViewPanDown),
        (ShortcutScope::View, KeyChord::Plus) => Some(AppAction::ViewZoomIn),
        (ShortcutScope::View, KeyChord::Minus) => Some(AppAction::ViewZoomOut),

        (ShortcutScope::Cad, KeyChord::Char('L')) => Some(AppAction::ToolLine),
        (ShortcutScope::Cad, KeyChord::Char('C')) => Some(AppAction::ToolCircle),
        (ShortcutScope::Cad, KeyChord::Char('A')) => Some(AppAction::ToolArc),
        (ShortcutScope::Cad, KeyChord::Char('D')) => Some(AppAction::ToolDimension),
        (ShortcutScope::Cad, KeyChord::Char('T')) => Some(AppAction::ToolText),

        (ShortcutScope::Global, KeyChord::Char('N')) => Some(AppAction::JobNest),
        (ShortcutScope::Global, KeyChord::Char('E')) => Some(AppAction::JobExport),

        _ => None,
    }
}

pub fn assert_table_coverage() {
    for sc in SHORTCUTS {
        for &ch in sc.chords {
            let _ = resolve_action(sc.scope, ch);
        }
    }
}

pub fn should_block_for_text_input(chord: KeyChord) -> bool {
    matches!(
        chord.normalize(),
        KeyChord::Char('L')
            | KeyChord::Char('C')
            | KeyChord::Char('A')
            | KeyChord::Char('D')
            | KeyChord::Char('T')
            | KeyChord::Char('N')
            | KeyChord::Char('E')
    )
}
