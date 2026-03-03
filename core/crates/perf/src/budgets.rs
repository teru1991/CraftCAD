use crate::metrics::PerfReport;
use craftcad_errors::{AppError, AppResult, ReasonCode, Severity};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct PerfBudgets {
    pub datasets: HashMap<String, DatasetBudget>,
    #[serde(default)]
    policy: PerfBudgetPolicy,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetBudget {
    pub open_p95_ms: Option<f64>,
    pub save_p95_ms: Option<f64>,
    pub io_roundtrip_p95_ms: Option<f64>,
    pub io_import_p95_ms: Option<f64>,
    pub io_export_p95_ms: Option<f64>,
    pub render_frame_p95_ms: Option<f64>,
    pub max_rss_mb: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PerfBudgetPolicy {
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_warn_in_pr")]
    pub warn_in_pr: bool,
    #[serde(default)]
    pub error_on_main: bool,
    #[serde(default = "default_min_samples")]
    pub min_samples: u32,
}

fn default_mode() -> String {
    "warn".to_string()
}

const fn default_warn_in_pr() -> bool {
    true
}

const fn default_min_samples() -> u32 {
    1
}

impl Default for PerfBudgetPolicy {
    fn default() -> Self {
        Self {
            mode: default_mode(),
            warn_in_pr: default_warn_in_pr(),
            error_on_main: false,
            min_samples: default_min_samples(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct SsotBudgetFile {
    #[serde(default)]
    policy: PerfBudgetPolicy,
    datasets: Vec<SsotBudgetDataset>,
}

#[derive(Debug, Clone, Deserialize)]
struct SsotBudgetDataset {
    dataset_id: String,
    budgets: DatasetBudget,
}

impl PerfBudgets {
    pub fn policy(&self) -> &PerfBudgetPolicy {
        &self.policy
    }
}

pub fn load_budgets(path: impl AsRef<Path>) -> AppResult<PerfBudgets> {
    let raw = std::fs::read_to_string(path.as_ref()).map_err(|e| {
        AppError::new(
            ReasonCode::new("PERF_BUDGET_LOAD_FAILED"),
            Severity::Error,
            format!("failed to read budgets: {e}"),
        )
    })?;

    if let Ok(v) = serde_json::from_str::<PerfBudgets>(&raw) {
        return Ok(v);
    }

    let ssot: SsotBudgetFile = serde_json::from_str(&raw).map_err(|e| {
        AppError::new(
            ReasonCode::new("PERF_BUDGET_SCHEMA_INVALID"),
            Severity::Error,
            format!("invalid budgets.json: {e}"),
        )
    })?;

    let datasets = ssot
        .datasets
        .into_iter()
        .map(|d| (d.dataset_id, d.budgets))
        .collect::<HashMap<_, _>>();

    Ok(PerfBudgets {
        datasets,
        policy: ssot.policy,
    })
}

pub fn check_report_against_budgets(report: &PerfReport, budgets: &PerfBudgets) -> Vec<AppError> {
    let mut out = vec![];
    let Some(ds) = budgets.datasets.get(&report.dataset_id) else {
        return out;
    };

    let find = |name: &str| {
        report
            .spans
            .iter()
            .filter(|s| s.name == name)
            .map(|s| s.duration_ms)
            .fold(None, |acc: Option<f64>, v| Some(acc.map_or(v, |a| a.max(v))))
    };

    if let (Some(limit), Some(actual)) = (ds.open_p95_ms, find("open")) {
        if actual > limit {
            out.push(
                AppError::new(
                    ReasonCode::new("PERF_BUDGET_EXCEEDED_OPEN_P95"),
                    Severity::Warn,
                    format!("open span exceeded budget: {actual:.2}ms > {limit:.2}ms"),
                )
                .with_context("dataset_id", &report.dataset_id),
            );
        }
    }

    if let (Some(limit), Some(actual)) = (ds.render_frame_p95_ms, find("render.frame")) {
        if actual > limit {
            out.push(
                AppError::new(
                    ReasonCode::new("PERF_BUDGET_EXCEEDED_RENDER_P95"),
                    Severity::Warn,
                    format!("render span exceeded budget: {actual:.2}ms > {limit:.2}ms"),
                )
                .with_context("dataset_id", &report.dataset_id),
            );
        }
    }

    if let Some(limit) = ds.io_import_p95_ms.or(ds.io_roundtrip_p95_ms) {
        if let Some(actual) = find("io.import.total") {
            if actual > limit {
                out.push(
                    AppError::new(
                        ReasonCode::new("PERF_BUDGET_EXCEEDED_IO_IMPORT_P95"),
                        Severity::Warn,
                        format!("io.import span exceeded budget: {actual:.2}ms > {limit:.2}ms"),
                    )
                    .with_context("dataset_id", &report.dataset_id),
                );
            }
        }
    }

    if let Some(limit) = ds.io_export_p95_ms.or(ds.io_roundtrip_p95_ms) {
        if let Some(actual) = find("io.export.total") {
            if actual > limit {
                out.push(
                    AppError::new(
                        ReasonCode::new("PERF_BUDGET_EXCEEDED_IO_EXPORT_P95"),
                        Severity::Warn,
                        format!("io.export span exceeded budget: {actual:.2}ms > {limit:.2}ms"),
                    )
                    .with_context("dataset_id", &report.dataset_id),
                );
            }
        }
    }

    if let (Some(limit), Some(actual)) = (ds.max_rss_mb, report.memory_peak_mb) {
        if actual > limit {
            out.push(
                AppError::new(
                    ReasonCode::new("PERF_BUDGET_EXCEEDED_MAX_RSS"),
                    Severity::Warn,
                    format!("memory exceeded budget: {actual}MB > {limit}MB"),
                )
                .with_context("dataset_id", &report.dataset_id),
            );
        }
    }

    out
}
