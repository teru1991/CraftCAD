pub mod error_ux;
pub mod jobs_ux;
pub mod modes {
    pub mod transitions;
}
pub mod onboarding;

use std::collections::BTreeMap;

pub struct AppState {
    pub onboarding: Option<onboarding::OnboardingController>,
    pub error_ux: Option<error_ux::ErrorUxController>,
    pub jobs_ux: jobs_ux::job_ux_controller::JobUxController,
    pub job_queue: jobs_ux::adapters::InMemoryJobQueue,
    pub job_status_cache: BTreeMap<String, onboarding::completion::JobStatus>,
    pub mode_state: modes::transitions::ModeState,
}

impl AppState {
    pub fn init() -> Self {
        let store_path = onboarding::first_run::default_first_run_store_path();
        let kv = onboarding::first_run::FileKvStore::new(store_path);
        let onboarding = onboarding::OnboardingController::new(Box::new(kv)).ok();
        let error_ux = error_ux::ErrorUxController::new().ok();
        Self {
            onboarding,
            error_ux,
            jobs_ux: jobs_ux::job_ux_controller::JobUxController::new(),
            job_queue: jobs_ux::adapters::InMemoryJobQueue::default(),
            job_status_cache: Default::default(),
            mode_state: Default::default(),
        }
    }

    pub fn enqueue_nest_job(&mut self) -> Result<(), String> {
        let req = jobs_ux::adapters::EnqueueRequest {
            kind: jobs_ux::adapters::JobKind::NestJob,
            args: BTreeMap::new(),
            priority: 0,
        };
        let _ = self.jobs_ux.handle_event(
            &mut self.job_queue,
            jobs_ux::job_ux_controller::JobUxEvent::Enqueue(req),
        )?;
        Ok(())
    }

    pub fn enqueue_export_job(&mut self) -> Result<(), String> {
        let req = jobs_ux::adapters::EnqueueRequest {
            kind: jobs_ux::adapters::JobKind::ExportJob,
            args: BTreeMap::new(),
            priority: 0,
        };
        let _ = self.jobs_ux.handle_event(
            &mut self.job_queue,
            jobs_ux::job_ux_controller::JobUxEvent::Enqueue(req),
        )?;
        Ok(())
    }

    pub fn enqueue_open_job(&mut self, path: String) -> Result<(), String> {
        let req = jobs_ux::adapters::EnqueueRequest {
            kind: jobs_ux::adapters::JobKind::OpenProject,
            args: BTreeMap::from([("path".to_string(), path)]),
            priority: 0,
        };
        let _ = self.jobs_ux.handle_event(
            &mut self.job_queue,
            jobs_ux::job_ux_controller::JobUxEvent::Enqueue(req),
        )?;
        Ok(())
    }

    pub fn enqueue_save_job(&mut self, path: String) -> Result<(), String> {
        let req = jobs_ux::adapters::EnqueueRequest {
            kind: jobs_ux::adapters::JobKind::SaveProject,
            args: BTreeMap::from([("path".to_string(), path)]),
            priority: 0,
        };
        let _ = self.jobs_ux.handle_event(
            &mut self.job_queue,
            jobs_ux::job_ux_controller::JobUxEvent::Enqueue(req),
        )?;
        Ok(())
    }

    pub fn tick_jobs_ux(&mut self, now_unix_ms: i64) {
        let effects = self.jobs_ux.tick(&self.job_queue);
        for eff in effects {
            match eff {
                jobs_ux::job_ux_controller::JobUxEffect::NotifyModesJobRunning { running } => {
                    modes::transitions::sync_job_running_from_jobs_ux(
                        &mut self.mode_state,
                        running,
                    );
                }
                jobs_ux::job_ux_controller::JobUxEffect::ShowError {
                    reason_code,
                    job_id,
                    context,
                } => {
                    if let Some(ctrl) = self.error_ux.as_mut() {
                        ctrl.show(error_ux::AppError {
                            reason_code,
                            severity: error_ux::Severity::Error,
                            context,
                            job_id,
                            op_id: None,
                        });
                    }
                }
                jobs_ux::job_ux_controller::JobUxEffect::NotifyOnboardingJobSucceeded {
                    kind,
                    job_id,
                } => {
                    self.job_status_cache.insert(
                        kind.as_str().to_string(),
                        onboarding::completion::JobStatus::Succeeded,
                    );
                    if let Some(onb) = self.onboarding.as_mut() {
                        onb.notify_job_succeeded(kind.as_str(), &job_id);
                    }
                }
                jobs_ux::job_ux_controller::JobUxEffect::None => {}
            }
        }

        if let Some(onb) = self.onboarding.as_mut() {
            struct CacheJobs<'a> {
                m: &'a BTreeMap<String, onboarding::completion::JobStatus>,
            }
            impl<'a> onboarding::completion::JobQuery for CacheJobs<'a> {
                fn job_status(&self, job_kind: &str) -> onboarding::completion::JobStatus {
                    self.m
                        .get(job_kind)
                        .cloned()
                        .unwrap_or(onboarding::completion::JobStatus::Unknown)
                }
            }
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
            let cj = CacheJobs {
                m: &self.job_status_cache,
            };
            let _ = onb.tick(&EmptyOp, &cj, now_unix_ms);
        }
    }

    pub fn tick_onboarding(&mut self, now_unix_ms: i64) {
        self.tick_jobs_ux(now_unix_ms);
    }

    pub fn show_error(&mut self, e: error_ux::AppError) {
        if let Some(ctrl) = self.error_ux.as_mut() {
            ctrl.show(e);
        }
    }

    pub fn handle_error_action_effect(&mut self, eff: error_ux::actions::ActionEffect) {
        match eff {
            error_ux::actions::ActionEffect::RetryLastJob => {
                let _ = self.jobs_ux.handle_event(
                    &mut self.job_queue,
                    jobs_ux::job_ux_controller::JobUxEvent::RetryLastFailed,
                );
            }
            error_ux::actions::ActionEffect::CreateSupportZip => {
                // Step5: consent-gated support-zip execution is handled by upper UI flow.
            }
            _ => {}
        }
    }
}
