use super::tutorial_state::{TutorialModel, TutorialState};

#[derive(Debug, Clone)]
pub struct OnboardingPanelView {
    pub visible: bool,
    pub flow_id: String,
    pub step_index_1based: u32,
    pub step_count: u32,
    pub title_key: String,
    pub body_key: String,
    pub can_back: bool,
    pub can_skip: bool,
    pub can_reset: bool,
    pub is_done: bool,
}

impl OnboardingPanelView {
    pub fn from(model: &TutorialModel, state: &TutorialState, visible: bool) -> Self {
        let step_count = model.steps.len() as u32;
        let mut idx = 1u32;
        let mut title_key = "ux.onboarding.missing.title".to_string();
        let mut body_key = "ux.onboarding.missing.body".to_string();
        let mut can_skip = false;

        for (i, s) in model.steps.iter().enumerate() {
            if s.id.0 == state.current_step.0 {
                idx = (i as u32) + 1;
                title_key = s.title_key.clone();
                body_key = s.body_key.clone();
                can_skip = s.can_skip;
                break;
            }
        }

        Self {
            visible,
            flow_id: model.flow_id.clone(),
            step_index_1based: idx,
            step_count,
            title_key,
            body_key,
            can_back: idx > 1 && !state.is_done,
            can_skip: can_skip && !state.is_done,
            can_reset: true,
            is_done: state.is_done,
        }
    }
}
