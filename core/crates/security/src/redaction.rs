use regex::Regex;
use serde_json::Value;
use sha2::{Digest, Sha256};

pub fn redact_str(s: &str) -> String {
    let email = Regex::new(r"(?i)\b[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}\b").expect("valid regex");
    let phone = Regex::new(r"\+?[0-9][0-9\- ]{7,}[0-9]").expect("valid regex");
    let url = Regex::new(r"https?://[^\s]+\b").expect("valid regex");

    let mut out = s
        .replace("/home/", "<HOME>/")
        .replace("\\Users\\", "<HOME>\\");
    out = email.replace_all(&out, "<EMAIL>").to_string();
    out = phone.replace_all(&out, "<PHONE>").to_string();
    out = url.replace_all(&out, "<URL>").to_string();
    out
}

fn hash_text(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn redact_json(v: Value) -> Value {
    match v {
        Value::String(s) => {
            if s.len() > 64 {
                serde_json::json!({"hashed": hash_text(&s), "len": s.len()})
            } else {
                Value::String(redact_str(&s))
            }
        }
        Value::Array(items) => Value::Array(items.into_iter().map(redact_json).collect()),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, redact_json(v)))
                .collect::<serde_json::Map<_, _>>(),
        ),
        other => other,
    }
}
