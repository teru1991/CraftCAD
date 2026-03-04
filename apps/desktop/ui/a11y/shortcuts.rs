use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShortcutScope {
  Global,
  View,
  Cad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
  Windows,
  Mac,
  Linux,
}

/// OS差を吸収する "Mod" キー（Win/Linux=Ctrl, macOS=Cmd）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyChord {
  Esc,
  F1,
  Plus,
  Minus,
  ArrowLeft,
  ArrowRight,
  ArrowUp,
  ArrowDown,
  Char(char),
  ModPlus(char),        // Mod + <char>
  ShiftModPlus(char),   // Shift + Mod + <char>
}

impl KeyChord {
  pub fn normalize(self) -> Self {
    match self {
      KeyChord::Char(c) => KeyChord::Char(c.to_ascii_uppercase()),
      KeyChord::ModPlus(c) => KeyChord::ModPlus(c.to_ascii_uppercase()),
      KeyChord::ShiftModPlus(c) => KeyChord::ShiftModPlus(c.to_ascii_uppercase()),
      x => x,
    }
  }
}

#[derive(Debug, Clone)]
pub struct Shortcut {
  pub id: &'static str,
  pub scope: ShortcutScope,
  pub chords: &'static [KeyChord],
  /// テキスト入力中は無効など、モード依存の説明（人間向け）
  pub note: &'static str,
}

pub const SHORTCUTS: &[Shortcut] = &[
  // Global
  Shortcut { id: "file.open", scope: ShortcutScope::Global, chords: &[KeyChord::ModPlus('O')], note: "" },
  Shortcut { id: "file.save", scope: ShortcutScope::Global, chords: &[KeyChord::ModPlus('S')], note: "" },
  Shortcut { id: "edit.undo", scope: ShortcutScope::Global, chords: &[KeyChord::ModPlus('Z')], note: "" },
  Shortcut { id: "edit.redo", scope: ShortcutScope::Global, chords: &[KeyChord::ShiftModPlus('Z')], note: "" },
  Shortcut { id: "ui.cancel", scope: ShortcutScope::Global, chords: &[KeyChord::Esc], note: "Cancel current action and return to safe mode" },
  Shortcut { id: "library.search", scope: ShortcutScope::Global, chords: &[KeyChord::ModPlus('F')], note: "" },
  Shortcut { id: "help.support", scope: ShortcutScope::Global, chords: &[KeyChord::F1], note: "" },

  // View
  Shortcut { id: "view.pan", scope: ShortcutScope::View, chords: &[
    KeyChord::ArrowLeft, KeyChord::ArrowRight, KeyChord::ArrowUp, KeyChord::ArrowDown
  ], note: "Pan by keyboard" },
  Shortcut { id: "view.zoom_in", scope: ShortcutScope::View, chords: &[KeyChord::Plus], note: "" },
  Shortcut { id: "view.zoom_out", scope: ShortcutScope::View, chords: &[KeyChord::Minus], note: "" },

  // CAD
  Shortcut { id: "tool.line", scope: ShortcutScope::Cad, chords: &[KeyChord::Char('L')], note: "Disabled while typing in text input" },
  Shortcut { id: "tool.circle", scope: ShortcutScope::Cad, chords: &[KeyChord::Char('C')], note: "Disabled while typing in text input" },
  Shortcut { id: "tool.arc", scope: ShortcutScope::Cad, chords: &[KeyChord::Char('A')], note: "Disabled while typing in text input" },
  Shortcut { id: "tool.dimension", scope: ShortcutScope::Cad, chords: &[KeyChord::Char('D')], note: "Disabled while typing in text input" },
  Shortcut { id: "tool.text", scope: ShortcutScope::Cad, chords: &[KeyChord::Char('T')], note: "Disabled while typing in text input" },

  // Jobs (kept global)
  Shortcut { id: "job.nest", scope: ShortcutScope::Global, chords: &[KeyChord::Char('N')], note: "Disabled while typing in text input" },
  Shortcut { id: "job.export", scope: ShortcutScope::Global, chords: &[KeyChord::Char('E')], note: "Disabled while typing in text input" },
];

#[derive(Debug, Clone)]
pub struct ShortcutConflict {
  pub scope: ShortcutScope,
  pub chord: KeyChord,
  pub ids: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct ShortcutCheck {
  pub conflicts: Vec<ShortcutConflict>,
  pub duplicate_ids: Vec<&'static str>,
}

pub fn check_shortcuts_table() -> ShortcutCheck {
  let mut conflicts = Vec::new();
  let mut id_seen = BTreeSet::new();
  let mut duplicate_ids = Vec::new();

  // scope -> chord -> ids
  let mut map: BTreeMap<ShortcutScope, BTreeMap<KeyChord, Vec<&'static str>>> = BTreeMap::new();

  for sc in SHORTCUTS {
    if !id_seen.insert(sc.id) {
      duplicate_ids.push(sc.id);
    }
    let entry = map.entry(sc.scope).or_default();
    for &ch in sc.chords {
      let ch = ch.clone().normalize();
      entry.entry(ch).or_default().push(sc.id);
    }
  }

  for (scope, chords) in map {
    for (ch, ids) in chords {
      if ids.len() > 1 {
        conflicts.push(ShortcutConflict { scope, chord: ch, ids });
      }
    }
  }

  ShortcutCheck { conflicts, duplicate_ids }
}

/// v1: 起動時に呼び出し、衝突があればエラー相当として扱うための文字列を作る。
pub fn format_conflicts(check: &ShortcutCheck) -> Option<String> {
  if check.conflicts.is_empty() && check.duplicate_ids.is_empty() {
    return None;
  }
  let mut lines = Vec::new();
  if !check.duplicate_ids.is_empty() {
    lines.push(format!("duplicate shortcut ids: {:?}", check.duplicate_ids));
  }
  for c in &check.conflicts {
    lines.push(format!("conflict scope={:?} chord={:?} ids={:?}", c.scope, c.chord, c.ids));
  }
  Some(lines.join("\n"))
}


pub fn is_text_sensitive_single_key(chord: KeyChord) -> bool {
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
