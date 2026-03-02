use crate::{
    reasons::MigrateReason,
    report::{ChangeSet, CountDelta, MigrateReport, StepReport},
    step::MigrationStep,
};
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::BTreeMap;

pub struct Registry {
    steps: BTreeMap<i64, Box<dyn MigrationStep>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            steps: BTreeMap::new(),
        }
    }

    pub fn register(mut self, step: Box<dyn MigrationStep>) -> Self {
        let from = step.from_version();
        self.steps.insert(from, step);
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn migrate_stepwise(
        &self,
        from: i64,
        to: i64,
        manifest: &mut Value,
        document: &mut Value,
        parts: &mut [Value],
        nest_jobs: &mut [Value],
        counts_before: (usize, usize, usize),
        counts_after: (usize, usize, usize),
        dry_run: bool,
        validate_input: &mut dyn FnMut(i64, &Value, &Value) -> Result<()>,
        validate_output: &mut dyn FnMut(i64, &Value, &Value) -> Result<()>,
    ) -> Result<MigrateReport> {
        if from < 0 {
            return Err(anyhow!(
                "{}: from < 0",
                MigrateReason::InvalidFromVersion.as_str()
            ));
        }
        if to < from {
            return Err(anyhow!(
                "{}: to < from",
                MigrateReason::InvalidToVersion.as_str()
            ));
        }
        if from == to {
            let cd = CountDelta {
                parts: counts_after.0 as i64 - counts_before.0 as i64,
                nest_jobs: counts_after.1 as i64 - counts_before.1 as i64,
                assets: counts_after.2 as i64 - counts_before.2 as i64,
            };
            return Ok(MigrateReport {
                overall_from: from,
                overall_to: to,
                steps: vec![],
                count_delta: cd,
            });
        }

        let mut cur = from;
        let mut step_reports: Vec<StepReport> = Vec::new();

        while cur < to {
            let step = self.steps.get(&cur).ok_or_else(|| {
                anyhow!(
                    "{}: from_version={}",
                    MigrateReason::RegistryMissingStep.as_str(),
                    cur
                )
            })?;
            let expected_to = cur + 1;
            if step.to_version() != expected_to {
                return Err(anyhow!(
                    "{}: step {} -> {} expected {} -> {}",
                    MigrateReason::SkipNotAllowed.as_str(),
                    step.from_version(),
                    step.to_version(),
                    cur,
                    expected_to
                ));
            }

            validate_input(cur, manifest, document).map_err(|e| {
                anyhow!(
                    "{}: {}: {}",
                    MigrateReason::ValidateInputFailed.as_str(),
                    cur,
                    e
                )
            })?;

            let mut changes = ChangeSet::default();
            if !dry_run {
                step.transform_manifest(manifest, &mut changes)
                    .map_err(|e| {
                        anyhow!(
                            "{}: manifest: {}",
                            MigrateReason::TransformFailed.as_str(),
                            e
                        )
                    })?;
                step.transform_document(document, &mut changes)
                    .map_err(|e| {
                        anyhow!(
                            "{}: document: {}",
                            MigrateReason::TransformFailed.as_str(),
                            e
                        )
                    })?;
                for p in parts.iter_mut() {
                    step.transform_part(p, &mut changes).map_err(|e| {
                        anyhow!("{}: part: {}", MigrateReason::TransformFailed.as_str(), e)
                    })?;
                }
                for n in nest_jobs.iter_mut() {
                    step.transform_nest_job(n, &mut changes).map_err(|e| {
                        anyhow!(
                            "{}: nest_job: {}",
                            MigrateReason::TransformFailed.as_str(),
                            e
                        )
                    })?;
                }
                step.logical_validate(manifest, document).map_err(|e| {
                    anyhow!(
                        "{}: logical: {}",
                        MigrateReason::ValidateOutputFailed.as_str(),
                        e
                    )
                })?;
                validate_output(cur + 1, manifest, document).map_err(|e| {
                    anyhow!(
                        "{}: {}: {}",
                        MigrateReason::ValidateOutputFailed.as_str(),
                        cur + 1,
                        e
                    )
                })?;
            } else {
                let mut m2 = manifest.clone();
                let mut d2 = document.clone();
                let mut ps: Vec<Value> = parts.to_vec();
                let mut ns: Vec<Value> = nest_jobs.to_vec();
                step.transform_manifest(&mut m2, &mut changes)?;
                step.transform_document(&mut d2, &mut changes)?;
                for p in ps.iter_mut() {
                    step.transform_part(p, &mut changes)?;
                }
                for n in ns.iter_mut() {
                    step.transform_nest_job(n, &mut changes)?;
                }
                step.logical_validate(&m2, &d2)?;
                validate_output(cur + 1, &m2, &d2).map_err(|e| {
                    anyhow!(
                        "{}: {}: {}",
                        MigrateReason::ValidateOutputFailed.as_str(),
                        cur + 1,
                        e
                    )
                })?;
            }

            step_reports.push(StepReport {
                bump: step.bump(),
                changes,
            });
            cur += 1;
        }

        let cd = CountDelta {
            parts: counts_after.0 as i64 - counts_before.0 as i64,
            nest_jobs: counts_after.1 as i64 - counts_before.1 as i64,
            assets: counts_after.2 as i64 - counts_before.2 as i64,
        };

        Ok(MigrateReport {
            overall_from: from,
            overall_to: to,
            steps: step_reports,
            count_delta: cd,
        })
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}
