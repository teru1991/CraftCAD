use crate::metrics::PerfReport;
use craftcad_errors::{AppError, AppResult, ReasonCode, Severity};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct PerfBudgets {
    pub datasets: HashMap<String, DatasetBudget>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetBudget {
    pub open_p95_ms: Option<f64>,
    pub save_p95_ms: Option<f64>,
    pub io_roundtrip_p95_ms: Option<f64>,
    pub render_frame_p95_ms: Option<f64>,
    pub max_rss_mb: Option<u64>,
}

pub fn load_budgets(path: impl AsRef<Path>) -> AppResult<PerfBudgets> {
    let raw = std::fs::read_to_string(path.as_ref()).map_err(|e| {
        AppError::new(
            ReasonCode::new("PERF_BUDGET_LOAD_FAILED"),
            Severity::Error,
            format!("failed to read budgets: {e}"),
        )
    })?;
    serde_json::from_str(&raw).map_err(|e| {
        AppError::new(
            ReasonCode::new("PERF_BUDGET_SCHEMA_INVALID"),
            Severity::Error,
            format!("invalid budgets.json: {e}"),
        )
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
            .fold(None, |acc: Option<f64>, v| {
                Some(acc.map_or(v, |a| a.max(v)))
            })
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
