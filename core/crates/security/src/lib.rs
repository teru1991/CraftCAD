//! CraftCAD Security Core (Sprint18)
//!
//! Goals:
//! - Treat ALL inputs as untrusted.
//! - No panics; return structured errors with reason codes.
//! - Deterministic behavior (same inputs => same outputs).
//! - SSOT-driven (docs/specs/security/*).

mod consent;
mod limits;
mod reasons;
mod redaction;
mod sandbox;
mod ssot;
mod util;

pub use consent::{ConsentDecision, ConsentLoadOutcome, ConsentState, ConsentStore};
pub use limits::{LimitKind, Limits, LimitsProfile, ZipStats};
pub use reasons::{SecCode, SecError, SecResult, SecWarning};
pub use redaction::{
    JsonPolicy, PathRules, PiiPatterns, RedactionRules, Redactor, RedactorConfig, TextPolicy,
    ZipPolicy,
};
pub use sandbox::{
    ExternalRefPolicy, NormalizedRelPath, PathValidationContext, Sandbox, SvgExternalRefAction,
};
pub use ssot::{RepoRoot, SsotPaths};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn consent_corruption_resets_to_default() {
        let dir = tempdir().expect("tempdir should be creatable");
        let p = dir.path().join("consent.json");
        std::fs::write(&p, b"{ not json").expect("fixture should be writable");
        let store = ConsentStore::with_path(&p);
        let out = store.load();
        assert_eq!(out.state, ConsentState::default());
        assert!(out
            .warnings
            .iter()
            .any(|w| w.code == SecCode::SecConsentReset));
    }

    #[test]
    fn sandbox_blocks_traversal_and_absolute() {
        let sb = Sandbox::new(ExternalRefPolicy::Reject);
        let ctx = PathValidationContext { max_depth: 20 };
        assert!(sb.normalize_rel_path(ctx.clone(), "../a").is_err());
        assert!(sb.normalize_rel_path(ctx.clone(), "/etc/passwd").is_err());
        assert!(sb
            .normalize_rel_path(ctx.clone(), "C:\\\\Windows\\\\x")
            .is_err());
        assert!(sb
            .normalize_rel_path(ctx.clone(), "ok/dir/file.txt")
            .is_ok());
    }

    #[test]
    fn redaction_is_deterministic_and_free_text_is_hashed() {
        let limits = Limits {
            max_import_bytes: 1024 * 1024,
            max_entities: 1,
            max_zip_entries: 1,
            max_zip_total_uncompressed_bytes: 1,
            max_single_entry_bytes: 1,
            max_json_depth: 50,
            max_string_len: 200000,
            max_paths_per_entity: 1,
            max_points_per_path: 1,
            max_support_zip_bytes: 1,
            max_path_depth: 20,
        };
        let rules = RedactionRules {
            version: 1,
            path_rules: PathRules {
                replace_home: "<HOME>".into(),
                replace_drive_letters: "<DRIVE>".into(),
                replace_unc: "<UNC>".into(),
                replace_absolute: "<ABS_PATH>".into(),
                replace_parent_segments: "<PARENT>".into(),
            },
            pii_patterns: PiiPatterns {
                email: r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b".into(),
                phone: r"\b\d{2,}\b".into(),
                url: r"(?i)\bhttps?://[^\s]+\b".into(),
                token_like: r"(?i)\b(token|api_key)\s*[:=]\s*[^\s,;]+".into(),
                bearer: r"(?i)\bBearer\s+[A-Za-z0-9._\-]+".into(),
                query_secret_kv: r"(?i)([?&](?:token)=)[^&\s]+".into(),
                long_hex: r"\b[a-fA-F0-9]{32,}\b".into(),
            },
            text_policy: TextPolicy {
                free_text_mode: "hash_len_prefix".into(),
                free_text_prefix_chars: 8,
                hash_algo: "sha256".into(),
                max_len_after_redaction: 200000,
            },
            json_policy: JsonPolicy {
                always_mask_keys: vec!["email".into(), "token".into()],
                mask_value_token: "<REDACTED>".into(),
                free_text_keys: vec!["note".into()],
            },
            zip_policy: ZipPolicy {
                redact_filenames: true,
                filename_replacement: "<FILE>".into(),
            },
        };
        let red = Redactor::new(rules, limits).expect("rules should be valid");
        let v = json!({"email":"a@b.com","note":"hello world","x":"Bearer SECRET"});
        let r1 = red.redact_json(&v);
        let r2 = red.redact_json(&v);
        assert_eq!(r1, r2);
        assert_eq!(r1.get("email").and_then(|x| x.as_str()), Some("<REDACTED>"));
        assert!(r1
            .get("note")
            .and_then(|x| x.as_str())
            .is_some_and(|x| x.contains("free_text:hash=sha256:")));
    }
}
