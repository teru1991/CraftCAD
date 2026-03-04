pub mod onboarding;

pub struct AppState {
    pub onboarding: Option<onboarding::OnboardingController>,
}

impl AppState {
    pub fn init() -> Self {
        let store_path = onboarding::first_run::default_first_run_store_path();
        let kv = onboarding::first_run::FileKvStore::new(store_path);
        let onboarding = onboarding::OnboardingController::new(Box::new(kv)).ok();
        Self { onboarding }
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
}
