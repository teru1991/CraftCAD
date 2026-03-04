use crate::reasons::{diag_codes, Severity};
use crate::security_iface::{ConsentProvider, Limits, Redactor};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;
use time::OffsetDateTime;

static NOW_FN: OnceLock<fn() -> String> = OnceLock::new();

fn now_rfc3339_utc() -> String {
    if let Some(f) = NOW_FN.get() {
        return f();
    }
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

pub fn set_now_fn_for_tests(f: fn() -> String) {
    let _ = NOW_FN.set(f);
}

fn now_offset_utc() -> OffsetDateTime {
    let now = now_rfc3339_utc();
    OffsetDateTime::parse(&now, &time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| OffsetDateTime::now_utc())
}

fn truncate_str(mut s: String, max_len: usize) -> (String, bool) {
    if s.len() <= max_len {
        return (s, false);
    }
    s.truncate(max_len);
    (s, true)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeterminismTag {
    pub seed: i64,
    pub epsilon: f64,
    pub rounding: String,
    pub ordering: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Header {
    pub joblog_version: i64,
    pub app_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_id: Option<String>,
    pub schema_version: String,
    pub os: String,
    pub arch: String,
    pub locale: String,
    pub timezone: String,
    pub determinism_tag: DeterminismTag,
    pub limits_profile: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputRef {
    pub kind: String,
    pub id: String,
    pub sha256: String,
    pub size_bytes: i64,
    pub source_hint_redacted: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepResultKind {
    Ok,
    Failed,
    Cancelled,
    Skipped,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelineStep {
    pub ts: String,
    pub action_id: String,
    pub action_kind: String,
    pub params_redacted: Value,
    pub result_kind: StepResultKind,
    pub reason_codes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts_refs: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timeline {
    pub steps: Vec<TimelineStep>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReasonAggregate {
    pub code: String,
    pub severity: Severity,
    pub count: i64,
    pub first_ts: String,
    pub last_ts: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Outputs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exported_files: Option<Vec<ExportedFile>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reports: Option<Vec<ReportRef>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportedFile {
    pub format: String,
    pub sha256: String,
    pub size_bytes: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReportRef {
    pub kind: String,
    pub r#ref: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsentSnapshot {
    pub include_project_snapshot: bool,
    pub include_inputs_copy: bool,
    pub telemetry_opt_in: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobLog {
    pub header: Header,
    pub inputs: Vec<InputRef>,
    pub timeline: Timeline,
    pub reasons: Vec<ReasonAggregate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Outputs>,
    pub consent_snapshot: ConsentSnapshot,
}

impl JobLog {
    pub fn header_to_ctx(&self) -> JobContext {
        JobContext {
            app_version: self.header.app_version.clone(),
            build_id: self.header.build_id.clone(),
            schema_version: self.header.schema_version.clone(),
            os: self.header.os.clone(),
            arch: self.header.arch.clone(),
            locale: self.header.locale.clone(),
            timezone: self.header.timezone.clone(),
            determinism_tag: self.header.determinism_tag.clone(),
            limits_profile: self.header.limits_profile.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct JobContext {
    pub app_version: String,
    pub build_id: Option<String>,
    pub schema_version: String,
    pub os: String,
    pub arch: String,
    pub locale: String,
    pub timezone: String,
    pub determinism_tag: DeterminismTag,
    pub limits_profile: String,
}

#[derive(Clone, Debug)]
struct ReasonState {
    severity: Severity,
    count: i64,
    first_ts: String,
    last_ts: String,
}

pub struct JobLogBuilder<'a> {
    redactor: &'a dyn Redactor,
    consent: &'a dyn ConsentProvider,
    limits: Limits,

    header: Header,
    inputs: Vec<InputRef>,
    steps: Vec<TimelineStep>,
    outputs_opt: Option<Outputs>,

    reasons_state: BTreeMap<String, ReasonState>,
    truncated: bool,
}

impl<'a> JobLogBuilder<'a> {
    pub fn new(
        ctx: JobContext,
        redactor: &'a dyn Redactor,
        consent: &'a dyn ConsentProvider,
        limits: Limits,
    ) -> Self {
        let header = Header {
            joblog_version: 1,
            app_version: ctx.app_version,
            build_id: ctx.build_id,
            schema_version: ctx.schema_version,
            os: ctx.os,
            arch: ctx.arch,
            locale: ctx.locale,
            timezone: ctx.timezone,
            determinism_tag: ctx.determinism_tag,
            limits_profile: ctx.limits_profile,
        };
        Self {
            redactor,
            consent,
            limits,
            header,
            inputs: Vec::new(),
            steps: Vec::new(),
            outputs_opt: None,
            reasons_state: BTreeMap::new(),
            truncated: false,
        }
    }

    pub fn add_input(
        &mut self,
        kind: &str,
        id: &str,
        sha256: &str,
        size_bytes: i64,
        source_hint_redacted: &str,
    ) {
        if self.inputs.len() >= self.limits.max_inputs {
            self.truncated = true;
            self.add_reason(diag_codes::DIAG_INPUTS_TRUNCATED, Severity::Warn);
            return;
        }
        let (kind, _) = truncate_str(self.redactor.redact_str(kind), 32);
        let (id, _) = truncate_str(self.redactor.redact_str(id), 128);
        let (sha, _) = truncate_str(self.redactor.redact_str(sha256), 64);
        let (hint, hint_tr) = truncate_str(self.redactor.redact_str(source_hint_redacted), 128);
        if hint_tr {
            self.add_reason(diag_codes::DIAG_PARAMS_TRUNCATED, Severity::Warn);
        }
        self.inputs.push(InputRef {
            kind,
            id,
            sha256: sha,
            size_bytes,
            source_hint_redacted: hint,
        });
    }

    pub fn begin_step(
        &mut self,
        action_id: &str,
        action_kind: &str,
        params_redacted: &Value,
    ) -> StepGuard<'_, 'a> {
        let start = now_offset_utc();
        let ts = now_rfc3339_utc();

        let (action_id, id_tr) = truncate_str(self.redactor.redact_str(action_id), 128);
        let (action_kind, kind_tr) = truncate_str(self.redactor.redact_str(action_kind), 64);
        if id_tr || kind_tr {
            self.add_reason(diag_codes::DIAG_PARAMS_TRUNCATED, Severity::Warn);
        }

        // Mandatory redaction: caller-provided params are always passed through redactor.
        let mut p = self.redactor.redact_json(params_redacted);
        let serialized = serde_json::to_string(&p).unwrap_or_else(|_| "{}".to_string());
        if serialized.len() > self.limits.max_string_len * 4 {
            self.truncated = true;
            self.add_reason(diag_codes::DIAG_PARAMS_TRUNCATED, Severity::Warn);
            p = serde_json::json!({"_truncated": true});
        }

        StepGuard {
            builder: self,
            ts,
            started_at: start,
            action_id,
            action_kind,
            params_redacted: p,
            result_kind: None,
            reason_codes: BTreeSet::new(),
            artifacts_refs: None,
        }
    }

    pub fn add_reason(&mut self, code: &str, severity: Severity) {
        let code = self.redactor.redact_str(code);
        let (code, _) = truncate_str(code, 128);
        let ts = now_rfc3339_utc();
        let entry = self
            .reasons_state
            .entry(code)
            .or_insert_with(|| ReasonState {
                severity,
                count: 0,
                first_ts: ts.clone(),
                last_ts: ts.clone(),
            });
        entry.count += 1;
        entry.last_ts = ts;
        if severity > entry.severity {
            entry.severity = severity;
        }
    }

    pub fn attach_exported_file(&mut self, format: &str, sha256: &str, size_bytes: i64) {
        let can_push = self
            .outputs_opt
            .as_ref()
            .and_then(|o| o.exported_files.as_ref())
            .map(|v| v.len() < 4096)
            .unwrap_or(true);
        if !can_push {
            self.truncated = true;
            self.add_reason(diag_codes::DIAG_LOG_TRUNCATED, Severity::Warn);
            return;
        }

        let (format, _) = truncate_str(self.redactor.redact_str(format), 32);
        let (sha, _) = truncate_str(self.redactor.redact_str(sha256), 64);
        let out = self.ensure_outputs_mut();
        let list = out.exported_files.get_or_insert_with(Vec::new);
        list.push(ExportedFile {
            format,
            sha256: sha,
            size_bytes,
        });
    }

    pub fn attach_report_ref(&mut self, kind: &str, r#ref: &str) {
        let can_push = self
            .outputs_opt
            .as_ref()
            .and_then(|o| o.reports.as_ref())
            .map(|v| v.len() < 1024)
            .unwrap_or(true);
        if !can_push {
            self.truncated = true;
            self.add_reason(diag_codes::DIAG_LOG_TRUNCATED, Severity::Warn);
            return;
        }

        let (kind, _) = truncate_str(self.redactor.redact_str(kind), 64);
        let (r#ref, _) = truncate_str(self.redactor.redact_str(r#ref), 256);
        let out = self.ensure_outputs_mut();
        let list = out.reports.get_or_insert_with(Vec::new);
        list.push(ReportRef { kind, r#ref });
    }

    fn ensure_outputs_mut(&mut self) -> &mut Outputs {
        if self.outputs_opt.is_none() {
            self.outputs_opt = Some(Outputs {
                exported_files: None,
                reports: None,
            });
        }
        self.outputs_opt.as_mut().expect("outputs must exist")
    }

    pub fn finish(mut self) -> JobLog {
        if self.steps.len() > self.limits.max_steps {
            self.steps.truncate(self.limits.max_steps);
            self.truncated = true;
            self.add_reason(diag_codes::DIAG_STEP_TRUNCATED, Severity::Warn);
        }

        if self.truncated {
            self.add_reason(diag_codes::DIAG_LOG_TRUNCATED, Severity::Warn);
        }

        let reasons = self
            .reasons_state
            .iter()
            .map(|(code, st)| ReasonAggregate {
                code: code.clone(),
                severity: st.severity,
                count: st.count,
                first_ts: st.first_ts.clone(),
                last_ts: st.last_ts.clone(),
            })
            .collect::<Vec<_>>();

        JobLog {
            header: self.header,
            inputs: self.inputs,
            timeline: Timeline { steps: self.steps },
            reasons,
            outputs: self.outputs_opt,
            consent_snapshot: ConsentSnapshot {
                include_project_snapshot: self.consent.include_project_snapshot(),
                include_inputs_copy: self.consent.include_inputs_copy(),
                telemetry_opt_in: self.consent.telemetry_opt_in(),
            },
        }
    }
}

pub struct StepGuard<'b, 'a> {
    builder: &'b mut JobLogBuilder<'a>,
    ts: String,
    started_at: OffsetDateTime,
    action_id: String,
    action_kind: String,
    params_redacted: Value,
    result_kind: Option<StepResultKind>,
    reason_codes: BTreeSet<String>,
    artifacts_refs: Option<Vec<String>>,
}

impl<'b, 'a> StepGuard<'b, 'a> {
    pub fn add_reason_code(&mut self, code: &str) {
        if self.reason_codes.len() >= self.builder.limits.max_reasons_per_step {
            self.builder.truncated = true;
            self.builder
                .add_reason(diag_codes::DIAG_LOG_TRUNCATED, Severity::Warn);
            return;
        }
        let (code, _) = truncate_str(self.builder.redactor.redact_str(code), 128);
        self.reason_codes.insert(code);
    }

    pub fn set_result(&mut self, result: StepResultKind) {
        self.result_kind = Some(result);
    }

    pub fn set_artifacts_refs(&mut self, refs: Vec<String>) {
        let mut out = Vec::new();
        for r in refs.into_iter().take(256) {
            let (rr, _) = truncate_str(self.builder.redactor.redact_str(&r), 256);
            out.push(rr);
        }
        self.artifacts_refs = Some(out);
    }

    pub fn builder(&mut self) -> &mut JobLogBuilder<'a> {
        self.builder
    }
}

impl<'b, 'a> Drop for StepGuard<'b, 'a> {
    fn drop(&mut self) {
        if self.builder.steps.len() >= self.builder.limits.max_steps {
            self.builder.truncated = true;
            self.builder
                .add_reason(diag_codes::DIAG_STEP_TRUNCATED, Severity::Warn);
            return;
        }

        let end = now_offset_utc();
        let dur = end - self.started_at;
        let duration_ms = dur.whole_milliseconds().max(0).min(i64::MAX as i128) as i64;

        let reason_codes = self.reason_codes.iter().cloned().collect::<Vec<_>>();

        for rc in &reason_codes {
            self.builder.add_reason(rc, Severity::Warn);
        }

        let step = TimelineStep {
            ts: self.ts.clone(),
            action_id: self.action_id.clone(),
            action_kind: self.action_kind.clone(),
            params_redacted: self.params_redacted.clone(),
            result_kind: self.result_kind.clone().unwrap_or(StepResultKind::Ok),
            reason_codes,
            duration_ms: Some(duration_ms),
            artifacts_refs: self.artifacts_refs.clone(),
        };
        self.builder.steps.push(step);
    }
}
