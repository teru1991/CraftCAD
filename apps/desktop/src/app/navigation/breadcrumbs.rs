#[derive(Debug, Clone)]
pub struct Breadcrumbs {
    pattern: Vec<String>,
    current: Vec<String>,
}

impl Breadcrumbs {
    pub fn new(pattern: Vec<String>) -> Self {
        Self {
            pattern,
            current: vec![],
        }
    }

    pub fn set(&mut self, items: Vec<String>) {
        let max = self.pattern.len();
        self.current = items.into_iter().take(max).collect();
    }

    pub fn get(&self) -> Vec<String> {
        self.current.clone()
    }
}
