use std::collections::HashMap;

pub fn default_shortcuts() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("file.open", "Ctrl+O"),
        ("file.save", "Ctrl+S"),
        ("edit.undo", "Ctrl+Z"),
        ("edit.redo", "Ctrl+Shift+Z"),
        ("view.zoom_fit", "F"),
    ])
}
