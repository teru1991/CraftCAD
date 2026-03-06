use crate::reasons::Severity;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

type ReasonCountsMap = BTreeMap<String, (i64, Option<String>, Option<String>, Option<Severity>)>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReasonSummaryItem {
    pub code: String,
    pub count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_ts: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_ts: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuggestedAction {
    pub code: String,
    pub doc_link: String,
    pub user_actions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReasonSummary {
    pub top_reasons: Vec<ReasonSummaryItem>,
    pub suggested_actions: Vec<SuggestedAction>,
}

pub trait ReasonCatalogLookup {
    fn lookup(&self, code: &str) -> Option<(String, Vec<String>)>;
}

pub struct EmptyCatalogLookup;
impl ReasonCatalogLookup for EmptyCatalogLookup {
    fn lookup(&self, _code: &str) -> Option<(String, Vec<String>)> {
        None
    }
}

impl ReasonSummary {
    pub fn from_reason_counts_stable(
        counts: &ReasonCountsMap,
        catalog: &dyn ReasonCatalogLookup,
        top_n: usize,
    ) -> Self {
        let mut items = counts
            .iter()
            .map(|(code, (count, first, last, sev))| ReasonSummaryItem {
                code: code.clone(),
                count: *count,
                first_ts: first.clone(),
                last_ts: last.clone(),
                severity: *sev,
            })
            .collect::<Vec<_>>();

        items.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.code.cmp(&b.code)));

        if items.len() > top_n {
            items.truncate(top_n);
        }

        let mut suggested_actions = Vec::new();
        for it in &items {
            if let Some((doc_link, user_actions)) = catalog.lookup(&it.code) {
                suggested_actions.push(SuggestedAction {
                    code: it.code.clone(),
                    doc_link,
                    user_actions,
                });
            }
        }
        suggested_actions.sort_by(|a, b| a.code.cmp(&b.code));

        Self {
            top_reasons: items,
            suggested_actions,
        }
    }
}
