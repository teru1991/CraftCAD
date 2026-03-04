#[derive(Debug, Clone)]
pub struct Backstack {
    enabled: bool,
    cap: usize,
    items: std::collections::VecDeque<String>,
}

impl Backstack {
    pub fn new(cap: usize, enabled: bool) -> Self {
        Self {
            enabled,
            cap,
            items: std::collections::VecDeque::new(),
        }
    }

    pub fn push(&mut self, route: String) {
        if !self.enabled {
            return;
        }
        if self.items.len() == self.cap {
            self.items.pop_back();
        }
        self.items.push_front(route);
    }

    pub fn pop(&mut self) -> Option<String> {
        if !self.enabled {
            return None;
        }
        self.items.pop_front()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}
