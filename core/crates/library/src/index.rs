use crate::reasons::{LibraryReason, LibraryReasonCode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    PresetMaterial,
    PresetProcess,
    PresetOutput,
    PresetHardware,
    Template,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AssetId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMeta {
    pub kind: AssetKind,
    pub id: String,
    pub version: String,
    pub display_name_key: Option<String>,
    pub tags: Vec<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryIndex {
    pub schema_version: i32,
    pub built_at_unix_ms: i64,
    pub assets: BTreeMap<String, AssetMeta>,
    pub token_to_assets: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub asset_key: String,
    pub score: i64,
}

fn sha256_hex(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn make_asset_key(kind: AssetKind, id: &str, version: &str, source: &str) -> String {
    format!(
        "{:?}:{}:{}",
        kind,
        id,
        sha256_hex(&format!("{version}:{source}"))
    )
}

fn tokenize_query(q: &str) -> Vec<String> {
    let s = q.trim().to_lowercase();
    let mut tokens = vec![];
    let mut cur = String::new();
    for ch in s.chars() {
        let sep = ch.is_whitespace()
            || matches!(
                ch,
                '/' | '\\'
                    | ':'
                    | ';'
                    | ','
                    | '.'
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '"'
                    | '\''
                    | '|'
                    | '!'
                    | '?'
                    | '+'
                    | '*'
                    | '='
                    | '<'
                    | '>'
            );
        let sep2 = matches!(ch, '_' | '-');
        if sep || sep2 {
            if !cur.is_empty() {
                tokens.push(cur.clone());
                cur.clear();
            }
            continue;
        }
        cur.push(ch);
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }

    tokens.retain(|t| !t.is_empty());
    tokens.sort();
    tokens.dedup();
    tokens
}

fn score_asset(meta: &AssetMeta, tokens: &[String], raw_q: &str) -> i64 {
    let q = raw_q.trim().to_lowercase();
    let mut score = 0i64;

    let id = meta.id.to_lowercase();
    if id == q {
        score += 10_000_000;
    }
    if id.starts_with(&q) && !q.is_empty() {
        score += 5_000_000;
    }

    if let Some(name) = &meta.display_name_key {
        let n = name.to_lowercase();
        if n == q {
            score += 2_000_000;
        }
        if n.contains(&q) && !q.is_empty() {
            score += 1_000_000;
        }
    }

    let tag_set: BTreeSet<String> = meta.tags.iter().cloned().collect();
    for t in tokens {
        if tag_set.contains(t) {
            score += 100_000;
        }
        if id.contains(t) {
            score += 50_000;
        }
    }

    score
}

fn source_priority(src: &str) -> i64 {
    match src {
        "builtin" => 0,
        "user" => 1,
        "project" => 2,
        _ => 9,
    }
}

pub fn search(index: &LibraryIndex, query: &str, limit: usize) -> Vec<SearchHit> {
    let tokens = tokenize_query(query);
    let mut hits = vec![];

    for (k, meta) in &index.assets {
        let s = score_asset(meta, &tokens, query);
        if s > 0 {
            hits.push(SearchHit {
                asset_key: k.clone(),
                score: s,
            });
        }
    }

    hits.sort_by(|a, b| {
        let ma = &index.assets[&a.asset_key];
        let mb = &index.assets[&b.asset_key];
        match b.score.cmp(&a.score) {
            std::cmp::Ordering::Equal => {
                let va = semver::Version::parse(&ma.version).ok();
                let vb = semver::Version::parse(&mb.version).ok();
                match (va, vb) {
                    (Some(va), Some(vb)) => match vb.cmp(&va) {
                        std::cmp::Ordering::Equal => {
                            match source_priority(&ma.source).cmp(&source_priority(&mb.source)) {
                                std::cmp::Ordering::Equal => a.asset_key.cmp(&b.asset_key),
                                o => o,
                            }
                        }
                        o => o,
                    },
                    _ => a.asset_key.cmp(&b.asset_key),
                }
            }
            o => o,
        }
    });

    hits.truncate(limit);
    hits
}

pub fn rebuild_index(
    built_at_unix_ms: i64,
    assets: Vec<AssetMeta>,
) -> Result<LibraryIndex, LibraryReason> {
    let mut map = BTreeMap::<String, AssetMeta>::new();
    let mut token_to_assets = BTreeMap::<String, Vec<String>>::new();

    for a in assets {
        let key = make_asset_key(a.kind, &a.id, &a.version, &a.source);
        if map.contains_key(&key) {
            return Err(LibraryReason::new(
                LibraryReasonCode::LibIndexCorrupt,
                format!("duplicate asset_key: {key}"),
            ));
        }

        let mut toks = BTreeSet::<String>::new();
        toks.insert(a.id.to_lowercase());
        for t in &a.tags {
            toks.insert(t.to_lowercase());
        }
        if let Some(n) = &a.display_name_key {
            toks.insert(n.to_lowercase());
        }

        for tok in toks {
            token_to_assets.entry(tok).or_default().push(key.clone());
        }

        map.insert(key.clone(), a);
    }

    for vecs in token_to_assets.values_mut() {
        vecs.sort();
        vecs.dedup();
    }

    Ok(LibraryIndex {
        schema_version: 1,
        built_at_unix_ms,
        assets: map,
        token_to_assets,
    })
}
