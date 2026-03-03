use crate::joblog::{JobLog, TimelineStep};
use crate::oplog::{ActionRecord, OpLog};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReproArtifacts {
    pub zip_name: String,
    pub zip_sha256: String,
}

#[derive(Clone, Debug)]
pub struct ReproText {
    pub markdown: String,
}

pub fn generate_repro_markdown(
    joblog: &JobLog,
    oplog: Option<&OpLog>,
    artifacts: Option<ReproArtifacts>,
) -> ReproText {
    let mut s = String::new();

    let _ = writeln!(&mut s, "# Repro Template (Generated)\n");
    let _ = writeln!(&mut s, "## Environment");
    let _ = writeln!(&mut s, "- app_version: {}", joblog.header.app_version);
    if let Some(bid) = &joblog.header.build_id {
        let _ = writeln!(&mut s, "- build_id: {}", bid);
    } else {
        let _ = writeln!(&mut s, "- build_id: (none)");
    }
    let _ = writeln!(&mut s, "- schema_version: {}", joblog.header.schema_version);
    let _ = writeln!(
        &mut s,
        "- os: {} / {}",
        joblog.header.os, joblog.header.arch
    );
    let _ = writeln!(&mut s, "- locale: {}", joblog.header.locale);
    let _ = writeln!(&mut s, "- timezone: {}", joblog.header.timezone);
    let _ = writeln!(
        &mut s,
        "- determinism_tag: seed={}, epsilon={}, rounding={}, ordering={}",
        joblog.header.determinism_tag.seed,
        joblog.header.determinism_tag.epsilon,
        joblog.header.determinism_tag.rounding,
        joblog.header.determinism_tag.ordering
    );
    let _ = writeln!(&mut s, "- limits_profile: {}", joblog.header.limits_profile);

    let _ = writeln!(&mut s, "\n## Inputs (no paths)");
    let mut inputs = joblog.inputs.clone();
    inputs.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then_with(|| a.id.cmp(&b.id))
            .then_with(|| a.sha256.cmp(&b.sha256))
    });
    for i in &inputs {
        let _ = writeln!(
            &mut s,
            "- kind: {}, sha256: {}, size_bytes: {}",
            i.kind, i.sha256, i.size_bytes
        );
    }

    let _ = writeln!(&mut s, "\n## Steps");
    if let Some(op) = oplog {
        let mut actions = op.actions.clone();
        actions.sort_by(|a, b| a.seq.cmp(&b.seq));
        for a in &actions {
            let _ = writeln!(&mut s, "{}. {}", a.seq, action_to_text(a));
        }
    } else {
        for (idx, st) in joblog.timeline.steps.iter().enumerate() {
            let _ = writeln!(
                &mut s,
                "{}. {} ({})",
                idx + 1,
                st.action_kind,
                st.result_kind_as_str()
            );
        }
    }

    let _ = writeln!(&mut s, "\n## Expected Result");
    let _ = writeln!(&mut s, "(What you expected to happen)\n");
    let _ = writeln!(&mut s, "## Actual Result");
    let _ = writeln!(&mut s, "(What happened instead)\n");

    let _ = writeln!(&mut s, "## Reason Codes");
    for r in &joblog.reasons {
        let _ = writeln!(&mut s, "- {} (count={})", r.code, r.count);
    }

    let _ = writeln!(&mut s, "\n## Attachments");
    if let Some(a) = artifacts {
        let _ = writeln!(&mut s, "- diagnostics_zip: {}", a.zip_name);
        let _ = writeln!(&mut s, "- diagnostics_zip_sha256: {}", a.zip_sha256);
    } else {
        let _ = writeln!(&mut s, "- diagnostics_zip: (not generated)");
        let _ = writeln!(&mut s, "- diagnostics_zip_sha256: (n/a)");
    }

    let _ = writeln!(
        &mut s,
        "\nNOTE: Do not include file paths or personal data. All fields are redacted/hashed."
    );

    ReproText { markdown: s }
}

fn action_to_text(a: &ActionRecord) -> String {
    let short = &a.params_hash[..8.min(a.params_hash.len())];
    format!("{} (params_hash={}…)", a.kind, short)
}

trait StepResultKindStr {
    fn result_kind_as_str(&self) -> &'static str;
}
impl StepResultKindStr for TimelineStep {
    fn result_kind_as_str(&self) -> &'static str {
        match self.result_kind {
            crate::joblog::StepResultKind::Ok => "ok",
            crate::joblog::StepResultKind::Failed => "failed",
            crate::joblog::StepResultKind::Cancelled => "cancelled",
            crate::joblog::StepResultKind::Skipped => "skipped",
        }
    }
}
