use craftcad_screw_lite::{eval_screw_points, ScrewEvalError, ScrewPoint};
use craftcad_ssot::{ManufacturingOutline2dV1, SsotV1};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RuleSeverity {
    Fatal,
    Warn,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleFinding {
    pub reason_code: String,
    pub severity: RuleSeverity,
    pub message: String,
    pub fix_hints: Vec<String>,
    pub part_id: Option<Uuid>,
    pub feature_id: Option<Uuid>,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleReport {
    pub findings: Vec<RuleFinding>,
}

impl RuleReport {
    pub fn canonicalize(mut self) -> Self {
        self.findings.sort_by(|a, b| {
            (a.severity, &a.reason_code, a.part_id, a.feature_id).cmp(&(
                b.severity,
                &b.reason_code,
                b.part_id,
                b.feature_id,
            ))
        });
        self
    }

    pub fn has_fatal(&self) -> bool {
        self.findings
            .iter()
            .any(|f| f.severity == RuleSeverity::Fatal)
    }
}

#[derive(Debug, Clone)]
pub struct RuleConfig {
    pub min_edge_distance_mm: f64,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            min_edge_distance_mm: 10.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PreflightError {
    pub reason_code: String,
    pub report: RuleReport,
}

fn bbox_min_distance(o: &ManufacturingOutline2dV1, x: f64, y: f64) -> f64 {
    let left = x - o.min_x;
    let right = o.max_x - x;
    let bottom = y - o.min_y;
    let top = o.max_y - y;
    left.min(right).min(bottom).min(top)
}

fn find_outline(ssot: &SsotV1, part_id: Uuid) -> Option<&ManufacturingOutline2dV1> {
    ssot.parts
        .iter()
        .find(|p| p.part_id == part_id)
        .and_then(|p| p.manufacturing_outline_2d.as_ref())
}

pub fn run_rules_edge_distance(
    ssot: &SsotV1,
    cfg: RuleConfig,
) -> Result<RuleReport, ScrewEvalError> {
    let pts = eval_screw_points(ssot)?;
    Ok(run_rules_edge_distance_with_points(ssot, cfg, &pts))
}

pub fn run_rules_edge_distance_with_points(
    ssot: &SsotV1,
    cfg: RuleConfig,
    pts: &[ScrewPoint],
) -> RuleReport {
    let mut findings: Vec<RuleFinding> = Vec::new();

    for p in pts {
        let Some(outline) = find_outline(ssot, p.part_id) else {
            findings.push(RuleFinding {
                reason_code: "RULE_INPUT_MISSING".to_string(),
                severity: RuleSeverity::Warn,
                message: "manufacturing_outline_2d missing; edge distance rule skipped".to_string(),
                fix_hints: vec![
                    "Generate manufacturing outline for this part".to_string(),
                    "Ensure part has a valid bbox before running safety checks".to_string(),
                ],
                part_id: Some(p.part_id),
                feature_id: Some(p.feature_id),
                details: json!({ "part_id": p.part_id, "feature_id": p.feature_id }),
            });
            continue;
        };

        let min_d = bbox_min_distance(outline, p.x, p.y);
        if !min_d.is_finite() {
            continue;
        }
        if min_d < cfg.min_edge_distance_mm {
            findings.push(RuleFinding {
                reason_code: "RULE_EDGE_DISTANCE_VIOLATION".to_string(),
                severity: RuleSeverity::Fatal,
                message: format!(
                    "edge distance too small: {:.3}mm < {:.3}mm",
                    min_d, cfg.min_edge_distance_mm
                ),
                fix_hints: vec![
                    "Increase edge_offset_mm".to_string(),
                    "Increase part size (width/height)".to_string(),
                    "Reduce screw count or increase pitch_mm".to_string(),
                ],
                part_id: Some(p.part_id),
                feature_id: Some(p.feature_id),
                details: json!({
                    "part_id": p.part_id,
                    "feature_id": p.feature_id,
                    "x": p.x,
                    "y": p.y,
                    "min_distance_mm": min_d,
                    "threshold_mm": cfg.min_edge_distance_mm
                }),
            });
        }
    }

    RuleReport { findings }.canonicalize()
}

pub fn preflight_rules(ssot: &SsotV1, cfg: RuleConfig) -> Result<RuleReport, PreflightError> {
    let report = match run_rules_edge_distance(ssot, cfg) {
        Ok(r) => r,
        Err(err) => RuleReport {
            findings: vec![RuleFinding {
                reason_code: "RULE_INPUT_MISSING".to_string(),
                severity: RuleSeverity::Warn,
                message: format!("screw points unavailable: {err}"),
                fix_hints: vec!["Provide valid ScrewFeature points".to_string()],
                part_id: None,
                feature_id: None,
                details: json!({"error": err.to_string()}),
            }],
        },
    }
    .canonicalize();

    if report.has_fatal() {
        return Err(PreflightError {
            reason_code: "RULE_EDGE_DISTANCE_VIOLATION".to_string(),
            report,
        });
    }

    Ok(report)
}
