use super::transitions::ModeEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsKind {
    Windows,
    Mac,
    Linux,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub cmd: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Default for Modifiers {
    fn default() -> Self {
        Self {
            ctrl: false,
            cmd: false,
            shift: false,
            alt: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Esc,
    Enter,
    Tab,
    Z,
}

pub struct InputRouter {
    os: OsKind,
}

impl InputRouter {
    pub fn new(os: OsKind) -> Self {
        Self { os }
    }

    pub fn route_key(&self, key: Key, mods: Modifiers) -> Option<ModeEvent> {
        match key {
            Key::Esc => Some(ModeEvent::KeyEsc),
            Key::Enter => Some(ModeEvent::KeyEnter),
            Key::Z => {
                let primary = self.primary_modifier(mods);
                if primary && !mods.shift {
                    Some(ModeEvent::ShortcutUndo)
                } else if primary && mods.shift {
                    Some(ModeEvent::ShortcutRedo)
                } else {
                    None
                }
            }
            Key::Tab => None,
        }
    }

    fn primary_modifier(&self, mods: Modifiers) -> bool {
        match self.os {
            OsKind::Mac => mods.cmd,
            _ => mods.ctrl,
        }
    }

    pub fn validate_shortcuts_table() -> Result<(), String> {
        Ok(())
    }
}
