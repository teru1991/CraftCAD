use crate::retention::RetentionPolicy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoreIndexEntry {
    pub id: String,
    pub created_at: String,
    pub rel_dir: String,
    pub zip_rel_path: Option<String>,
    pub size_bytes: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct DiagnosticsStore {
    pub base_dir: PathBuf,
}

#[derive(Clone, Debug)]
pub struct CleanupResult {
    pub deleted_ids: Vec<String>,
    pub warnings: Vec<String>,
}

impl DiagnosticsStore {
    pub fn new(base_dir: impl AsRef<Path>) -> io::Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        fs::create_dir_all(base_dir.join("items"))?;
        Ok(Self { base_dir })
    }

    pub fn create_item_dir(&self) -> io::Result<(String, PathBuf)> {
        let id = Uuid::new_v4().to_string();
        let dir = self.base_dir.join("items").join(&id);
        fs::create_dir_all(&dir)?;
        Ok((id, dir))
    }

    pub fn index_path(&self) -> PathBuf {
        self.base_dir.join("index.jsonl")
    }

    pub fn append_index(&self, entry: &StoreIndexEntry) -> io::Result<()> {
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.index_path())?;
        let line = serde_json::to_string(entry).unwrap_or_else(|_| "{}".to_string());
        f.write_all(line.as_bytes())?;
        f.write_all(b"\n")?;
        f.sync_all()?;
        Ok(())
    }

    pub fn list_items(&self) -> io::Result<Vec<StoreIndexEntry>> {
        let p = self.index_path();
        if !p.exists() {
            return Ok(Vec::new());
        }
        let text = fs::read_to_string(p)?;
        let mut out = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(e) = serde_json::from_str::<StoreIndexEntry>(line) {
                let abs_dir = self.base_dir.join(&e.rel_dir);
                if abs_dir.exists() {
                    out.push(e);
                }
            }
        }
        out.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(out)
    }

    pub fn delete_item(&self, id: &str) -> io::Result<()> {
        let dir = self.base_dir.join("items").join(id);
        if dir.exists() {
            fs::remove_dir_all(dir)?;
        }
        Ok(())
    }

    pub fn delete_all(&self) -> io::Result<CleanupResult> {
        let items = self.list_items()?;
        let mut deleted = Vec::new();
        let mut warnings = Vec::new();
        for it in items {
            if let Err(e) = self.delete_item(&it.id) {
                warnings.push(format!("WARN_DELETE_FAILED:{}:{}", it.id, e));
            } else {
                deleted.push(it.id);
            }
        }
        Ok(CleanupResult {
            deleted_ids: deleted,
            warnings,
        })
    }

    pub fn cleanup(&self, policy: &RetentionPolicy) -> io::Result<CleanupResult> {
        let mut deleted = Vec::new();
        let mut warnings = Vec::new();

        let mut items = self.list_items()?;
        items.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.id.cmp(&b.id))
        });

        // delete expired (keep_days) first
        let cutoff = OffsetDateTime::now_utc() - Duration::days(policy.default_keep_days as i64);
        let mut current = self.list_items()?;
        current.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.id.cmp(&b.id))
        });

        for it in current.iter() {
            if let Ok(ts) = OffsetDateTime::parse(
                &it.created_at,
                &time::format_description::well_known::Rfc3339,
            ) {
                if ts < cutoff {
                    match self.delete_item(&it.id) {
                        Ok(()) => deleted.push(it.id.clone()),
                        Err(e) => warnings.push(format!("WARN_DELETE_FAILED:{}:{}", it.id, e)),
                    }
                }
            }
        }

        fn dir_size_bytes(p: &Path) -> u64 {
            let mut total = 0u64;
            let mut stack = vec![p.to_path_buf()];
            while let Some(d) = stack.pop() {
                if let Ok(rd) = fs::read_dir(&d) {
                    let mut ents = rd.filter_map(|e| e.ok()).collect::<Vec<_>>();
                    ents.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
                    for e in ents {
                        let path = e.path();
                        if let Ok(md) = e.metadata() {
                            if md.is_dir() {
                                stack.push(path);
                            } else {
                                total = total.saturating_add(md.len());
                            }
                        }
                    }
                }
            }
            total
        }

        loop {
            let mut cur = self.list_items()?;
            cur.sort_by(|a, b| {
                a.created_at
                    .cmp(&b.created_at)
                    .then_with(|| a.id.cmp(&b.id))
            });

            if cur.len() as u64 <= policy.max_items {
                let total_bytes = dir_size_bytes(&self.base_dir.join("items"));
                if total_bytes <= policy.max_total_bytes {
                    break;
                }
            }

            if let Some(oldest) = cur.first() {
                match self.delete_item(&oldest.id) {
                    Ok(()) => deleted.push(oldest.id.clone()),
                    Err(e) => warnings.push(format!("WARN_DELETE_FAILED:{}:{}", oldest.id, e)),
                }
            } else {
                break;
            }
        }

        warnings.sort();
        Ok(CleanupResult {
            deleted_ids: deleted,
            warnings,
        })
    }
}
