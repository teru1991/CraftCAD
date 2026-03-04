use std::fs;

#[test]
fn desktop_ui_avoids_selected_hardcoded_labels() {
    let repo = std::env::current_dir().expect("cwd");

    let target_files = [
        repo.join("apps/desktop/src/main.cpp"),
        repo.join("apps/desktop/src/canvas_widget.cpp"),
    ];

    let banned = [
        "Export completed.",
        "Export failed",
        "Diagnostic Pack Options",
        "Save Export",
    ];

    let mut hits = Vec::new();
    for f in target_files {
        if !f.exists() {
            continue;
        }
        let text = fs::read_to_string(&f).expect("read file");
        for term in banned {
            if text.contains(term) {
                hits.push(format!("{} contains {:?}", f.display(), term));
            }
        }
    }

    assert!(
        hits.is_empty(),
        "found hardcoded UI labels (migrate to i18n runtime):\n{}",
        hits.join("\n")
    );
}
