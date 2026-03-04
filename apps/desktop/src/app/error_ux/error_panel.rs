use super::*;

#[derive(Debug, Clone)]
pub struct ErrorPanelView {
    pub visible: bool,
    pub actions: Vec<UserFacingAction>,
    pub title_key: String,
    pub detail_key: String,
    pub why_key: Option<String>,
    pub doc_link: Option<String>,
    pub debug_ref: DebugRef,
    pub display_hash: String,
    pub detail_collapsed: bool,
}

impl ErrorPanelView {
    pub fn from_error(e: &UserFacingError, visible: bool) -> Self {
        Self {
            visible,
            actions: e.actions.clone(),
            title_key: e.title_key.clone(),
            detail_key: e.detail_key.clone(),
            why_key: e.why_key.clone(),
            doc_link: e.doc_link.clone(),
            debug_ref: e.debug_ref.clone(),
            display_hash: e.display_hash.clone(),
            detail_collapsed: true,
        }
    }

    pub fn copy_repro_text(&self) -> String {
        let mut s = String::new();
        s.push_str("CraftCAD Error Report\n");
        s.push_str(&format!("- reason_code: {}\n", self.debug_ref.reason_code));
        s.push_str(&format!("- severity: {:?}\n", self.debug_ref.severity));
        if let Some(j) = &self.debug_ref.job_id {
            s.push_str(&format!("- job_id: {}\n", j));
        }
        if let Some(o) = &self.debug_ref.op_id {
            s.push_str(&format!("- op_id: {}\n", o));
        }
        s.push_str(&format!("- display_hash: {}\n", self.display_hash));
        if !self.debug_ref.context.is_empty() {
            s.push_str("- context:\n");
            for (k, v) in &self.debug_ref.context {
                s.push_str(&format!("  - {}: {}\n", k, v));
            }
        }
        s
    }
}
