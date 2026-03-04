pub mod completion;
pub mod first_run;
pub mod samples;
pub mod spec;
pub mod tutorial_state;
pub mod ui;

#[cfg(test)]
mod tests;

use completion::{CompletionEngine, JobQuery, OpLogQuery};
use first_run::{FirstRun, KvStore};
use samples::{SampleId, SampleMeta, SampleRegistry};
use spec::{load_onboarding_flow, SsotPaths};
use tutorial_state::{TutorialModel, TutorialState};
use ui::OnboardingPanelView;

#[derive(Debug, Clone)]
pub enum OnboardingUserEvent {
    Show,
    Hide,
    Back,
    Skip,
    Reset,
    OpenSample(SampleId),
}

#[derive(Debug, Clone)]
pub enum OnboardingEffect {
    RequestOpenSample {
        sample_id: SampleId,
        read_only: bool,
    },
    None,
}

pub struct OnboardingController {
    visible: bool,
    model: TutorialModel,
    state: TutorialState,
    engine: CompletionEngine,
    first_run: FirstRun,
    samples: SampleRegistry,
}

impl OnboardingController {
    pub fn new(first_run_store: Box<dyn KvStore>) -> Result<Self, String> {
        let flow = load_onboarding_flow(&SsotPaths::default())?;
        let model = TutorialModel::from_ssot(&flow);
        let state = TutorialState::new(&model);
        let engine = CompletionEngine::new(flow);
        let first_run = FirstRun::new(first_run_store);
        let samples = SampleRegistry::new_default();
        Ok(Self {
            visible: false,
            model,
            state,
            engine,
            first_run,
            samples,
        })
    }

    pub fn should_auto_start(&self) -> bool {
        self.first_run.should_auto_start()
    }

    pub fn view(&self) -> OnboardingPanelView {
        OnboardingPanelView::from(&self.model, &self.state, self.visible)
    }

    pub fn list_samples(&self) -> Vec<SampleMeta> {
        self.samples.list_samples()
    }

    pub fn handle_event(
        &mut self,
        ev: OnboardingUserEvent,
        now_unix_ms: i64,
    ) -> Result<OnboardingEffect, String> {
        match ev {
            OnboardingUserEvent::Show => {
                self.visible = true;
                self.first_run.mark_started(now_unix_ms)?;
                Ok(OnboardingEffect::None)
            }
            OnboardingUserEvent::Hide => {
                self.visible = false;
                Ok(OnboardingEffect::None)
            }
            OnboardingUserEvent::Back => {
                self.state.go_back(&self.model);
                Ok(OnboardingEffect::None)
            }
            OnboardingUserEvent::Skip => {
                self.state.skip_current(&self.model);
                self.first_run.mark_skipped()?;
                Ok(OnboardingEffect::None)
            }
            OnboardingUserEvent::Reset => {
                self.state.reset(&self.model);
                self.first_run.reset_for_rerun()?;
                Ok(OnboardingEffect::None)
            }
            OnboardingUserEvent::OpenSample(sample_id) => Ok(OnboardingEffect::RequestOpenSample {
                sample_id,
                read_only: true,
            }),
        }
    }

    pub fn tick(
        &mut self,
        oplog: &dyn OpLogQuery,
        jobs: &dyn JobQuery,
        now_unix_ms: i64,
    ) -> Result<(), String> {
        if !self.visible {
            return Ok(());
        }
        self.engine
            .tick(&self.model, &mut self.state, oplog, jobs, now_unix_ms);
        if self.state.is_done {
            self.first_run.mark_completed(now_unix_ms)?;
        }
        Ok(())
    }
}
