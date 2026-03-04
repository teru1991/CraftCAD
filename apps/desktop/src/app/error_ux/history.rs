use super::*;

#[derive(Debug, Clone)]
pub struct ErrorHistory {
    cap: usize,
    items: std::collections::VecDeque<UserFacingError>,
}

impl ErrorHistory {
    pub fn new(cap: usize) -> Self {
        Self {
            cap,
            items: std::collections::VecDeque::new(),
        }
    }

    pub fn push(&mut self, e: UserFacingError) {
        if self.items.len() == self.cap {
            self.items.pop_back();
        }
        self.items.push_front(e);
    }

    pub fn list(&self) -> Vec<(String, String)> {
        self.items
            .iter()
            .map(|e| (e.display_hash.clone(), e.debug_ref.reason_code.clone()))
            .collect()
    }

    pub fn find_by_hash(&self, hash: &str) -> Option<UserFacingError> {
        self.items.iter().find(|e| e.display_hash == hash).cloned()
    }
}
