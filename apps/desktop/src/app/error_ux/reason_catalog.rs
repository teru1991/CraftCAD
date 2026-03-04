use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonCatalog {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub reasons: BTreeMap<String, ReasonEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonEntry {
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub title_key: Option<String>,
    #[serde(default)]
    pub detail_key: Option<String>,
    #[serde(default)]
    pub why_key: Option<String>,
    #[serde(default)]
    pub actions: Vec<ActionSpec>,
    #[serde(default)]
    pub doc_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSpec {
    pub kind: String,
    #[serde(default)]
    pub args: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub label_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CatalogPaths {
    pub catalog_json: PathBuf,
}

impl Default for CatalogPaths {
    fn default() -> Self {
        Self {
            catalog_json: PathBuf::from("docs/specs/errors/catalog.json"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct LegacyCatalog {
    #[serde(default)]
    version: serde_json::Value,
    #[serde(default)]
    items: Vec<LegacyItem>,
}

#[derive(Debug, Clone, Deserialize)]
struct LegacyItem {
    code: String,
    #[serde(default)]
    severity: Option<String>,
    #[serde(default)]
    doc_link: Option<String>,
}

impl ReasonCatalog {
    pub fn load(paths: &CatalogPaths) -> Result<Self, String> {
        let s = fs::read_to_string(&paths.catalog_json)
            .map_err(|e| format!("failed to read {}: {}", paths.catalog_json.display(), e))?;

        if let Ok(cat) = serde_json::from_str::<ReasonCatalog>(&s) {
            return Ok(cat);
        }

        let legacy: LegacyCatalog =
            serde_json::from_str(&s).map_err(|e| format!("catalog json parse error: {}", e))?;
        Ok(from_legacy(legacy))
    }

    pub fn get(&self, code: &str) -> Option<&ReasonEntry> {
        self.reasons.get(code)
    }

    pub fn fallback_title_key(&self) -> &'static str {
        "ux.error.generic.title"
    }
    pub fn fallback_detail_key(&self) -> &'static str {
        "ux.error.generic.detail"
    }
    pub fn fallback_why_key(&self) -> &'static str {
        "ux.error.generic.why"
    }
}

fn from_legacy(legacy: LegacyCatalog) -> ReasonCatalog {
    let mut reasons = BTreeMap::new();
    for i in legacy.items {
        reasons.insert(
            i.code,
            ReasonEntry {
                severity: i.severity,
                title_key: None,
                detail_key: None,
                why_key: None,
                actions: vec![],
                doc_link: i.doc_link,
            },
        );
    }
    ReasonCatalog {
        version: legacy
            .version
            .as_u64()
            .map(|v| v as u32)
            .unwrap_or_default(),
        reasons,
    }
}
