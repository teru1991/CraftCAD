use crate::reasons::{LibraryReason, LibraryReasonCode};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, Deserialize)]
pub struct TagsPolicy {
    pub schema_version: i32,
    pub max_len: i32,
    pub forbidden_patterns: Vec<String>,
    pub normalize: NormalizePolicy,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NormalizePolicy {
    pub lowercase: bool,
    pub trim: bool,
    pub collapse_spaces: bool,
    pub remove_zenkaku_spaces: bool,
}

fn repo_root_from_manifest() -> PathBuf {
    let start = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for up in 0..=10usize {
        let mut p = start.clone();
        for _ in 0..up {
            p = p.parent().unwrap_or(&p).to_path_buf();
        }
        if p.join("docs").join("specs").exists() {
            return p;
        }
    }
    panic!("repo root not found from {}", start.display());
}

pub fn load_tags_policy_from_repo_root(
    repo_root: Option<PathBuf>,
) -> Result<TagsPolicy, LibraryReason> {
    let root = repo_root.unwrap_or_else(repo_root_from_manifest);
    let p = root
        .join("docs")
        .join("specs")
        .join("library")
        .join("tags.schema.json");
    let s = fs::read_to_string(&p).map_err(|e| {
        LibraryReason::new(LibraryReasonCode::LibIoError, format!("read failed: {e}"))
            .with_path(p.display().to_string())
    })?;
    serde_json::from_str::<TagsPolicy>(&s).map_err(|e| {
        LibraryReason::new(
            LibraryReasonCode::LibTemplateInvalid,
            format!("tags policy parse failed: {e}"),
        )
        .with_path(p.display().to_string())
    })
}

fn collapse_spaces(s: &str) -> String {
    let mut out = String::new();
    let mut prev_space = false;
    for ch in s.chars() {
        let is_space = ch.is_whitespace();
        if is_space {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out
}

fn remove_zenkaku_spaces(s: &str) -> String {
    s.replace('\u{3000}', " ")
}

fn normalize_nfkc(s: &str) -> String {
    s.nfkc().collect::<String>()
}

pub fn normalize_tag(tag: &str, policy: &TagsPolicy) -> (String, Vec<LibraryReason>) {
    let mut w = vec![];
    let mut s = normalize_nfkc(tag);

    if policy.normalize.remove_zenkaku_spaces {
        let before = s.clone();
        s = remove_zenkaku_spaces(&s);
        if s != before {
            w.push(LibraryReason::new(
                LibraryReasonCode::LibTagNormalized,
                "zenkaku space removed",
            ));
        }
    }
    if policy.normalize.trim {
        let before = s.clone();
        s = s.trim().to_string();
        if s != before {
            w.push(LibraryReason::new(
                LibraryReasonCode::LibTagNormalized,
                "trimmed",
            ));
        }
    }
    if policy.normalize.collapse_spaces {
        let before = s.clone();
        s = collapse_spaces(&s);
        if s != before {
            w.push(LibraryReason::new(
                LibraryReasonCode::LibTagNormalized,
                "spaces collapsed",
            ));
        }
    }
    if policy.normalize.lowercase {
        let before = s.clone();
        s = s.to_lowercase();
        if s != before {
            w.push(LibraryReason::new(
                LibraryReasonCode::LibTagNormalized,
                "lowercased",
            ));
        }
    }
    (s, w)
}

pub fn validate_tag(tag: &str, policy: &TagsPolicy) -> Result<(), LibraryReason> {
    if tag.is_empty() {
        return Err(LibraryReason::new(
            LibraryReasonCode::LibTagInvalid,
            "tag must not be empty",
        ));
    }
    if (tag.chars().count() as i32) > policy.max_len {
        return Err(LibraryReason::new(
            LibraryReasonCode::LibTagInvalid,
            format!("tag too long (max {}): {}", policy.max_len, tag),
        ));
    }
    for pat in &policy.forbidden_patterns {
        if tag.contains(pat) {
            return Err(LibraryReason::new(
                LibraryReasonCode::LibTagInvalid,
                format!("tag contains forbidden pattern '{pat}': {tag}"),
            ));
        }
    }
    Ok(())
}

pub fn normalize_and_validate_tags(
    tags: &[String],
    policy: &TagsPolicy,
) -> Result<(Vec<String>, Vec<LibraryReason>), LibraryReason> {
    if tags.len() > 32 {
        return Err(LibraryReason::new(
            LibraryReasonCode::LibTagInvalid,
            "too many tags (max 32)",
        ));
    }
    let mut out = vec![];
    let mut warnings = vec![];

    for t in tags {
        let (n, mut w) = normalize_tag(t, policy);
        warnings.append(&mut w);
        validate_tag(&n, policy)?;
        out.push(n);
    }

    out.sort();
    out.dedup();
    Ok((out, warnings))
}
