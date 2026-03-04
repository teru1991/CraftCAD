use std::collections::BTreeSet;
use std::path::Path;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::limits::Limits;
use crate::reasons::{SecCode, SecError, SecResult};
use crate::ssot::{RepoRoot, SsotPaths};
use crate::util::{clamp_str_to_chars, sha256_hex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionRules {
    pub version: u64,
    pub path_rules: PathRules,
    pub pii_patterns: PiiPatterns,
    pub text_policy: TextPolicy,
    pub json_policy: JsonPolicy,
    pub zip_policy: ZipPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRules {
    pub replace_home: String,
    pub replace_drive_letters: String,
    pub replace_unc: String,
    pub replace_absolute: String,
    pub replace_parent_segments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiPatterns {
    pub email: String,
    pub phone: String,
    pub url: String,
    pub token_like: String,
    pub bearer: String,
    pub query_secret_kv: String,
    pub long_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPolicy {
    pub free_text_mode: String, // "hash_len" | "hash_len_prefix"
    pub free_text_prefix_chars: u64,
    pub hash_algo: String, // "sha256"
    pub max_len_after_redaction: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonPolicy {
    pub always_mask_keys: Vec<String>,
    pub mask_value_token: String,
    pub free_text_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipPolicy {
    pub redact_filenames: bool,
    pub filename_replacement: String,
}

#[derive(Debug, Clone)]
pub struct RedactorConfig {
    pub limits_profile: crate::limits::LimitsProfile,
}

#[derive(Debug, Clone)]
pub struct Redactor {
    rules: RedactionRules,
    limits: Limits,
    // compiled regex
    re_email: Regex,
    re_phone: Regex,
    re_url: Regex,
    re_token_like: Regex,
    re_bearer: Regex,
    re_query_secret_kv: Regex,
    re_long_hex: Regex,
    always_mask_keys: BTreeSet<String>,
    free_text_keys: BTreeSet<String>,
}

impl RedactionRules {
    pub fn load_from_ssot() -> SecResult<Self> {
        let root = RepoRoot::discover()?;
        let paths = SsotPaths::from_repo_root(&root);
        let s = std::fs::read_to_string(&paths.redaction_rules_json).map_err(|e| {
            SecError::new(
                SecCode::SecSsotNotFound,
                format!("read redaction_rules.json failed: {e}"),
            )
        })?;
        let r: RedactionRules = serde_json::from_str(&s).map_err(|e| {
            SecError::new(
                SecCode::SecSsotInvalid,
                format!("parse redaction_rules.json failed: {e}"),
            )
        })?;
        if r.version < 1 {
            return Err(SecError::new(
                SecCode::SecSsotInvalid,
                "redaction_rules.json version must be >= 1",
            ));
        }
        if r.text_policy.hash_algo != "sha256" {
            return Err(SecError::new(
                SecCode::SecSsotInvalid,
                "text_policy.hash_algo must be sha256",
            ));
        }
        Ok(r)
    }
}

impl Redactor {
    pub fn new(rules: RedactionRules, limits: Limits) -> SecResult<Self> {
        // compile regex patterns (fail fast, no panic)
        let re_email = Regex::new(&rules.pii_patterns.email).map_err(|e| {
            SecError::new(
                SecCode::SecRegexInvalid,
                format!("email regex invalid: {e}"),
            )
        })?;
        let re_phone = Regex::new(&rules.pii_patterns.phone).map_err(|e| {
            SecError::new(
                SecCode::SecRegexInvalid,
                format!("phone regex invalid: {e}"),
            )
        })?;
        let re_url = Regex::new(&rules.pii_patterns.url).map_err(|e| {
            SecError::new(SecCode::SecRegexInvalid, format!("url regex invalid: {e}"))
        })?;
        let re_token_like = Regex::new(&rules.pii_patterns.token_like).map_err(|e| {
            SecError::new(
                SecCode::SecRegexInvalid,
                format!("token_like regex invalid: {e}"),
            )
        })?;
        let re_bearer = Regex::new(&rules.pii_patterns.bearer).map_err(|e| {
            SecError::new(
                SecCode::SecRegexInvalid,
                format!("bearer regex invalid: {e}"),
            )
        })?;
        let re_query_secret_kv = Regex::new(&rules.pii_patterns.query_secret_kv).map_err(|e| {
            SecError::new(
                SecCode::SecRegexInvalid,
                format!("query_secret_kv regex invalid: {e}"),
            )
        })?;
        let re_long_hex = Regex::new(&rules.pii_patterns.long_hex).map_err(|e| {
            SecError::new(
                SecCode::SecRegexInvalid,
                format!("long_hex regex invalid: {e}"),
            )
        })?;

        let always_mask_keys = rules
            .json_policy
            .always_mask_keys
            .iter()
            .map(|s| s.to_ascii_lowercase())
            .collect();
        let free_text_keys = rules
            .json_policy
            .free_text_keys
            .iter()
            .map(|s| s.to_ascii_lowercase())
            .collect();

        Ok(Self {
            rules,
            limits,
            re_email,
            re_phone,
            re_url,
            re_token_like,
            re_bearer,
            re_query_secret_kv,
            re_long_hex,
            always_mask_keys,
            free_text_keys,
        })
    }

    pub fn from_ssot(cfg: RedactorConfig) -> SecResult<Self> {
        let limits = Limits::load_from_ssot(cfg.limits_profile)?;
        let rules = RedactionRules::load_from_ssot()?;
        Self::new(rules, limits)
    }

    pub fn rules(&self) -> &RedactionRules {
        &self.rules
    }

    pub fn redact_str(&self, input: &str) -> String {
        // Bound input length to avoid pathological regex runtime.
        let max_in = self
            .rules
            .text_policy
            .max_len_after_redaction
            .min(2_000_000) as usize;
        let mut s = if input.len() > max_in {
            input[..max_in].to_string()
        } else {
            input.to_string()
        };

        // Apply patterns in deterministic order.
        s = self
            .re_bearer
            .replace_all(&s, "Bearer <REDACTED>")
            .to_string();
        s = self
            .re_query_secret_kv
            .replace_all(&s, "$1<REDACTED>")
            .to_string();
        s = self
            .re_token_like
            .replace_all(&s, "$1=<REDACTED>")
            .to_string();
        s = self.re_email.replace_all(&s, "<EMAIL>").to_string();
        s = self.re_phone.replace_all(&s, "<PHONE>").to_string();
        s = self.re_url.replace_all(&s, "<URL>").to_string();
        s = self.re_long_hex.replace_all(&s, "<HEX>").to_string();

        // Path-like sequences: do a best-effort pass (platform-agnostic).
        // NOTE: This is heuristic; caller should pass actual paths to redact_path.
        self.clamp_final(s)
    }

    pub fn redact_path(&self, p: impl AsRef<Path>) -> String {
        let raw = p.as_ref().to_string_lossy().replace('\\', "/");
        self.redact_path_str(&raw)
    }

    pub fn redact_path_str(&self, raw: &str) -> String {
        let mut s = raw.replace('\\', "/");

        // Replace home dirs if we can detect them from env.
        if let Ok(home) = std::env::var("HOME") {
            let h = home.replace('\\', "/");
            if !h.is_empty() && s.starts_with(&h) {
                s = format!("{}{}", self.rules.path_rules.replace_home, &s[h.len()..]);
            }
        }
        if let Ok(up) = std::env::var("USERPROFILE") {
            let u = up.replace('\\', "/");
            if !u.is_empty() && s.starts_with(&u) {
                s = format!("{}{}", self.rules.path_rules.replace_home, &s[u.len()..]);
            }
        }

        // UNC path
        if s.starts_with("//") {
            s = format!("{}{}", self.rules.path_rules.replace_unc, &s[1..]); // keep one slash shape
        }

        // Drive letters (C:/...)
        if is_windows_drive_abs(&s) {
            s = format!("{}{}", self.rules.path_rules.replace_drive_letters, &s[2..]);
            // drop "C:"
        }

        // Absolute unix path
        if s.starts_with('/') {
            s = format!("{}{}", self.rules.path_rules.replace_absolute, s);
        }

        // Parent traversal segments
        if s.contains("/../") || s.contains("../") {
            s = s.replace("..", &self.rules.path_rules.replace_parent_segments);
        }

        self.clamp_final(s)
    }

    pub fn redact_json(&self, v: &Value) -> Value {
        self.redact_json_impl(v, 0)
    }

    fn redact_json_impl(&self, v: &Value, depth: u64) -> Value {
        // Depth cap: this is redaction-time guard (import-time guard is Limits).
        if depth > self.limits.max_json_depth {
            return Value::String("<DEPTH_LIMIT>".to_string());
        }

        match v {
            Value::Null => Value::Null,
            Value::Bool(b) => Value::Bool(*b),
            Value::Number(n) => Value::Number(n.clone()),
            Value::String(s) => Value::String(self.redact_str(s)),
            Value::Array(a) => {
                let mut out = Vec::with_capacity(a.len().min(1024));
                for it in a.iter().take(1024 * 1024) {
                    // very high cap; relies on upstream limits
                    out.push(self.redact_json_impl(it, depth + 1));
                }
                Value::Array(out)
            }
            Value::Object(o) => {
                let mut out = serde_json::Map::new();
                for (k, val) in o {
                    let kl = k.to_ascii_lowercase();
                    if self.always_mask_keys.contains(&kl) {
                        out.insert(
                            k.clone(),
                            Value::String(self.rules.json_policy.mask_value_token.clone()),
                        );
                        continue;
                    }
                    if self.free_text_keys.contains(&kl) {
                        out.insert(k.clone(), Value::String(self.redact_free_text(val)));
                        continue;
                    }
                    out.insert(k.clone(), self.redact_json_impl(val, depth + 1));
                }
                Value::Object(out)
            }
        }
    }

    fn redact_free_text(&self, v: &Value) -> String {
        let s = match v {
            Value::String(x) => x.as_str(),
            _ => {
                // Non-string free text -> stringify deterministically
                let rendered = v.to_string();
                return format!(
                    "free_text:hash=sha256:{};len={}",
                    sha256_hex(rendered.as_bytes()),
                    rendered.len()
                );
            }
        };
        let hash = sha256_hex(s.as_bytes());
        let len = s.chars().count();
        let prefix_chars = (self.rules.text_policy.free_text_prefix_chars as usize).min(64);
        match self.rules.text_policy.free_text_mode.as_str() {
            "hash_len" => format!("free_text:hash=sha256:{hash};len={len}"),
            "hash_len_prefix" => {
                let pref = clamp_str_to_chars(s, prefix_chars);
                format!(
                    "free_text:hash=sha256:{hash};len={len};prefix={}",
                    self.redact_str(&pref)
                )
            }
            _ => format!("free_text:hash=sha256:{hash};len={len}"),
        }
    }

    fn clamp_final(&self, mut s: String) -> String {
        let max = self.rules.text_policy.max_len_after_redaction as usize;
        if s.len() > max {
            s.truncate(max);
        }
        // Also enforce string_len limit (SSOT) to keep consistent with Limits usage.
        let lim = self.limits.max_string_len as usize;
        if s.len() > lim {
            s.truncate(lim);
        }
        s
    }
}

fn is_windows_drive_abs(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() >= 3
        && b[1] == b':'
        && (b[2] == b'/' || b[2] == b'\\')
        && ((b[0] as char).is_ascii_alphabetic())
}
