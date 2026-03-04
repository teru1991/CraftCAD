pub mod error_ux;
pub mod modes;
pub mod navigation;
pub mod onboarding;

pub struct AppState {
    pub onboarding: Option<onboarding::OnboardingController>,
    pub error_ux: Option<error_ux::ErrorUxController>,
    pub modes: Option<modes::ModesController>,
    pub nav: Option<navigation::NavigationController>,
}

impl AppState {
    pub fn init() -> Self {
        let store_path = onboarding::first_run::default_first_run_store_path();
        let kv = onboarding::first_run::FileKvStore::new(store_path);
        let onboarding = onboarding::OnboardingController::new(Box::new(kv)).ok();
        let error_ux = error_ux::ErrorUxController::new().ok();
        let modes = modes::ModesController::new().ok();
        let nav = navigation::NavigationController::new().ok();
        Self {
            onboarding,
            error_ux,
            modes,
            nav,
        }
    }

    pub fn tick_onboarding(&mut self, now_unix_ms: i64) {
        let Some(ctrl) = self.onboarding.as_mut() else {
            return;
        };

        struct EmptyOp;
        impl onboarding::completion::OpLogQuery for EmptyOp {
            fn has_op(
                &self,
                _op_kind: &str,
                _args: &std::collections::BTreeMap<String, serde_yaml::Value>,
            ) -> bool {
                false
            }
        }

        struct EmptyJobs;
        impl onboarding::completion::JobQuery for EmptyJobs {
            fn job_status(&self, _job_kind: &str) -> onboarding::completion::JobStatus {
                onboarding::completion::JobStatus::Unknown
            }
        }

        let _ = ctrl.tick(&EmptyOp, &EmptyJobs, now_unix_ms);
    }

    pub fn show_error(&mut self, e: error_ux::AppError) {
        if let Some(ctrl) = self.error_ux.as_mut() {
            ctrl.show(e);
        }
    }

    pub fn apply_mode_event(&mut self, ev: modes::transitions::ModeEvent) {
        let Some(m) = self.modes.as_mut() else {
            return;
        };
        let r = m.apply(ev);
        let _ = r.effects;
    }
}
