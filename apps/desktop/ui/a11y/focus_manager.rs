use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FocusId(pub &'static str);

#[derive(Debug, Clone)]
pub struct FocusChain {
    pub name: &'static str,
    pub ids: Vec<FocusId>,
    pub trap: bool,
}

impl FocusChain {
    pub fn index_of(&self, id: &FocusId) -> Option<usize> {
        self.ids.iter().position(|x| x == id)
    }
}

#[derive(Debug)]
pub struct FocusManager {
    chains: BTreeMap<&'static str, FocusChain>,
    current_chain: &'static str,
    focused: Option<FocusId>,
}

impl FocusManager {
    pub fn new(default_chain: FocusChain) -> Self {
        let mut chains = BTreeMap::new();
        let name = default_chain.name;
        chains.insert(name, default_chain);
        Self {
            chains,
            current_chain: name,
            focused: None,
        }
    }

    pub fn register_chain(&mut self, chain: FocusChain) {
        self.chains.insert(chain.name, chain);
    }

    pub fn set_chain(&mut self, name: &'static str) {
        if self.chains.contains_key(name) {
            self.current_chain = name;
            self.focused = None;
        }
    }

    pub fn focus_first(&mut self) -> Option<FocusId> {
        let chain = self.chains.get(self.current_chain)?;
        let id = chain.ids.first()?.clone();
        self.focused = Some(id.clone());
        Some(id)
    }

    pub fn current(&self) -> Option<&FocusId> {
        self.focused.as_ref()
    }

    pub fn on_tab(&mut self, reverse: bool) -> Option<FocusId> {
        let chain = self.chains.get(self.current_chain)?;
        if chain.ids.is_empty() {
            return None;
        }

        let idx = self
            .focused
            .as_ref()
            .and_then(|id| chain.index_of(id))
            .unwrap_or(0);
        let next = if reverse {
            if idx == 0 {
                if chain.trap { chain.ids.len() - 1 } else { 0 }
            } else {
                idx - 1
            }
        } else if idx + 1 >= chain.ids.len() {
            if chain.trap { 0 } else { chain.ids.len() - 1 }
        } else {
            idx + 1
        };
        let id = chain.ids[next].clone();
        self.focused = Some(id.clone());
        Some(id)
    }

    pub fn set_focused(&mut self, id: FocusId) {
        self.focused = Some(id);
    }
}
