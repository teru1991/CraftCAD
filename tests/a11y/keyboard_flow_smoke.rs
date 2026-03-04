mod shortcuts {
    include!("../../apps/desktop/ui/a11y/shortcuts.rs");
}
mod shortcut_dispatch {
    include!("../../apps/desktop/ui/a11y/shortcut_dispatch.rs");
}

use shortcut_dispatch::{resolve_action, should_block_for_text_input, AppAction};
use shortcuts::{KeyChord, ShortcutScope};

#[test]
fn shortcut_resolves_core_actions() {
    assert_eq!(
        resolve_action(ShortcutScope::Global, KeyChord::ModPlus('O')),
        Some(AppAction::FileOpen)
    );
    assert_eq!(
        resolve_action(ShortcutScope::Global, KeyChord::Esc),
        Some(AppAction::Cancel)
    );
    assert_eq!(
        resolve_action(ShortcutScope::Cad, KeyChord::Char('L')),
        Some(AppAction::ToolLine)
    );
}

#[test]
fn text_input_blocks_one_key_shortcuts() {
    assert!(should_block_for_text_input(KeyChord::Char('L')));
    assert!(should_block_for_text_input(KeyChord::Char('N')));
    assert!(!should_block_for_text_input(KeyChord::ModPlus('O')));
}
