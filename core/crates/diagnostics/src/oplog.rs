use crate::security_iface::{Limits, Redactor};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use time::OffsetDateTime;

fn now_rfc3339_utc() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

fn truncate_str(mut s: String, max_len: usize) -> (String, bool) {
    if s.len() <= max_len {
        return (s, false);
    }
    s.truncate(max_len);
    (s, true)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionResult {
    Ok,
    Failed,
    Cancelled,
    Skipped,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionRecord {
    pub seq: i64,
    pub action_id: String,
    pub kind: String,
    pub params_hash: String,
    pub params_redacted: Value,
    pub affected_ids: Vec<String>,
    pub result: ActionResult,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpLog {
    pub session_id: String,
    pub started_at: String,
    pub actions: Vec<ActionRecord>,
}

pub struct OpLogBuilder<'a> {
    redactor: &'a dyn Redactor,
    limits: Limits,
    session_id: String,
    started_at: String,
    next_seq: i64,
    actions: Vec<ActionRecord>,
}

impl<'a> OpLogBuilder<'a> {
    pub fn start_session(session_id: &str, redactor: &'a dyn Redactor, limits: Limits) -> Self {
        let (sid, _) = truncate_str(redactor.redact_str(session_id), 128);
        Self {
            redactor,
            limits,
            session_id: sid,
            started_at: now_rfc3339_utc(),
            next_seq: 1,
            actions: Vec::new(),
        }
    }

    pub fn record_action(
        &mut self,
        action_id: &str,
        kind: &str,
        params_redacted: &Value,
        affected_ids: &[String],
        result: ActionResult,
        reason_codes: &[String],
    ) {
        if self.actions.len() >= self.limits.max_steps {
            return;
        }

        let (aid, _) = truncate_str(self.redactor.redact_str(action_id), 128);
        let (kind, _) = truncate_str(self.redactor.redact_str(kind), 64);

        let mut pr = self.redactor.redact_json(params_redacted);
        let s = serde_json::to_vec(&pr).unwrap_or_else(|_| b"{}".to_vec());
        let params_hash = sha256_hex(&s);
        if s.len() > self.limits.max_string_len * 4 {
            pr = serde_json::json!({"_truncated": true});
        }

        let mut ids = affected_ids.to_vec();
        ids.sort();
        ids.dedup();
        ids.truncate(self.limits.max_affected_ids);
        ids = ids
            .into_iter()
            .map(|x| truncate_str(self.redactor.redact_str(&x), 128).0)
            .collect();

        let mut rset = BTreeSet::new();
        for rc in reason_codes.iter().take(self.limits.max_reasons_per_step) {
            rset.insert(truncate_str(self.redactor.redact_str(rc), 128).0);
        }
        let reason_codes = rset.into_iter().collect::<Vec<_>>();

        self.actions.push(ActionRecord {
            seq: self.next_seq,
            action_id: aid,
            kind,
            params_hash,
            params_redacted: pr,
            affected_ids: ids,
            result,
            reason_codes,
        });
        self.next_seq += 1;
    }

    pub fn finish(self) -> OpLog {
        OpLog {
            session_id: self.session_id,
            started_at: self.started_at,
            actions: self.actions,
        }
    }
}
