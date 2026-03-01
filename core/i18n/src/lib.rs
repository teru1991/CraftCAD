use once_cell::sync::Lazy;
use serde_json::{Map, Value};
use std::collections::HashMap;

static JA: &str = include_str!("../locales/ja-JP.json");
static EN: &str = include_str!("../locales/en-US.json");

static DICTS: Lazy<HashMap<&'static str, HashMap<String, String>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("ja-JP", parse_dict(JA));
    m.insert("en-US", parse_dict(EN));
    m
});

fn parse_dict(src: &str) -> HashMap<String, String> {
    serde_json::from_str(src).unwrap_or_default()
}

pub fn resolve_user_message(
    user_msg_key: &str,
    params: &Map<String, Value>,
    locale: &str,
) -> String {
    let dict = DICTS
        .get(locale)
        .or_else(|| DICTS.get("en-US"))
        .or_else(|| DICTS.get("ja-JP"));

    let mut msg = dict
        .and_then(|d| d.get(user_msg_key).cloned())
        .unwrap_or_else(|| user_msg_key.to_string());

    for (k, v) in params {
        let ph = format!("{{{}}}", k);
        let rv = if let Some(s) = v.as_str() {
            s.to_string()
        } else {
            v.to_string()
        };
        msg = msg.replace(&ph, &rv);
    }
    msg
}

pub fn locale_dict(locale: &str) -> Option<&'static HashMap<String, String>> {
    DICTS.get(locale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_with_param() {
        let mut p = Map::new();
        p.insert("id".into(), Value::String("abc".into()));
        let msg = resolve_user_message("model_reference_not_found", &p, "ja-JP");
        assert!(msg.contains("abc"));
    }
}
