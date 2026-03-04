use std::collections::{BTreeMap, BTreeSet};

use super::spec::OnboardingFlowSsot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepId(pub String);

#[derive(Debug, Clone)]
pub struct TutorialStep {
    pub id: StepId,
    pub title_key: String,
    pub body_key: String,
    pub next: Option<StepId>,
    pub can_skip: bool,
    pub links: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TutorialModel {
    pub flow_id: String,
    pub steps: Vec<TutorialStep>,
    index: BTreeMap<String, usize>,
}

impl TutorialModel {
    pub fn from_ssot(flow: &OnboardingFlowSsot) -> Self {
        let mut steps: Vec<TutorialStep> = Vec::new();
        let mut index = BTreeMap::new();
        for (i, s) in flow.steps.iter().enumerate() {
            let next = if s.next == "done" {
                None
            } else {
                Some(StepId(s.next.clone()))
            };
            let links = s.links.clone().unwrap_or_default();
            let st = TutorialStep {
                id: StepId(s.id.clone()),
                title_key: s.title_key.clone(),
                body_key: s.body_key.clone(),
                next,
                can_skip: s.can_skip,
                links,
            };
            index.insert(s.id.clone(), i);
            steps.push(st);
        }
        Self {
            flow_id: flow.flow_id.clone(),
            steps,
            index,
        }
    }

    pub fn step(&self, id: &StepId) -> Option<&TutorialStep> {
        self.index.get(&id.0).and_then(|i| self.steps.get(*i))
    }

    pub fn first_step_id(&self) -> StepId {
        StepId(self.steps[0].id.0.clone())
    }
}

#[derive(Debug, Clone)]
pub struct TutorialState {
    pub current_step: StepId,
    pub completed_steps: BTreeSet<String>,
    pub started_unix_ms: Option<i64>,
    pub finished_unix_ms: Option<i64>,
    pub is_done: bool,
}

impl TutorialState {
    pub fn new(model: &TutorialModel) -> Self {
        Self {
            current_step: model.first_step_id(),
            completed_steps: BTreeSet::new(),
            started_unix_ms: None,
            finished_unix_ms: None,
            is_done: false,
        }
    }

    pub fn mark_started(&mut self, now_unix_ms: i64) {
        self.started_unix_ms = Some(now_unix_ms);
    }

    pub fn complete_current_and_advance(&mut self, model: &TutorialModel) {
        self.completed_steps.insert(self.current_step.0.clone());
        let Some(step) = model.step(&self.current_step) else {
            return;
        };
        match &step.next {
            None => self.is_done = true,
            Some(n) => self.current_step = StepId(n.0.clone()),
        }
    }

    pub fn go_back(&mut self, model: &TutorialModel) {
        let mut prev: Option<StepId> = None;
        let mut cur = model.first_step_id();
        loop {
            if cur.0 == self.current_step.0 {
                break;
            }
            let s = model
                .step(&cur)
                .expect("flow must contain current step chain");
            if let Some(n) = &s.next {
                prev = Some(cur);
                cur = StepId(n.0.clone());
            } else {
                break;
            }
        }
        if let Some(p) = prev {
            self.current_step = p;
            self.is_done = false;
        }
    }

    pub fn skip_current(&mut self, model: &TutorialModel) {
        let Some(step) = model.step(&self.current_step) else {
            return;
        };
        if !step.can_skip {
            return;
        }
        match &step.next {
            None => self.is_done = true,
            Some(n) => self.current_step = StepId(n.0.clone()),
        }
    }

    pub fn reset(&mut self, model: &TutorialModel) {
        *self = TutorialState::new(model);
    }

    pub fn mark_finished(&mut self, now_unix_ms: i64) {
        self.finished_unix_ms = Some(now_unix_ms);
        self.is_done = true;
    }
}
