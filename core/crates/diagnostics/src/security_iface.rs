use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Limits {
    pub max_string_len: usize,
    pub max_inputs: usize,
    pub max_steps: usize,
    pub max_reasons_per_step: usize,
    pub max_affected_ids: usize,
}

impl Limits {
    pub fn conservative_default() -> Self {
        Self {
            max_string_len: 2048,
            max_inputs: 4096,
            max_steps: 100_000,
            max_reasons_per_step: 256,
            max_affected_ids: 4096,
        }
    }
}

pub trait Redactor: Send + Sync {
    fn redact_str(&self, s: &str) -> String;
    fn redact_json(&self, v: &Value) -> Value;
}

pub trait ConsentProvider: Send + Sync {
    fn include_project_snapshot(&self) -> bool;
    fn include_inputs_copy(&self) -> bool;
    fn telemetry_opt_in(&self) -> bool;
}

#[derive(Default)]
pub struct DefaultDenyConsent;
impl ConsentProvider for DefaultDenyConsent {
    fn include_project_snapshot(&self) -> bool {
        false
    }
    fn include_inputs_copy(&self) -> bool {
        false
    }
    fn telemetry_opt_in(&self) -> bool {
        false
    }
}

pub struct StubRedactor;

impl StubRedactor {
    fn strip_paths(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut i = 0;
        let bytes = s.as_bytes();
        while i < bytes.len() {
            let c = bytes[i] as char;
            if i + 2 < bytes.len() {
                let c0 = bytes[i] as char;
                let c1 = bytes[i + 1] as char;
                let c2 = bytes[i + 2] as char;
                if c0.is_ascii_alphabetic() && c1 == ':' && (c2 == '\\' || c2 == '/') {
                    out.push_str("<redacted_path>");
                    i += 3;
                    while i < bytes.len() {
                        let cc = bytes[i] as char;
                        if cc.is_whitespace() {
                            break;
                        }
                        i += 1;
                    }
                    continue;
                }
            }
            if c == '/' {
                out.push_str("<redacted_path>");
                i += 1;
                while i < bytes.len() {
                    let cc = bytes[i] as char;
                    if cc.is_whitespace() {
                        break;
                    }
                    i += 1;
                }
                continue;
            }
            out.push(c);
            i += 1;
        }
        out
    }
}

impl Redactor for StubRedactor {
    fn redact_str(&self, s: &str) -> String {
        Self::strip_paths(s)
    }

    fn redact_json(&self, v: &Value) -> Value {
        fn walk(red: &StubRedactor, v: &Value) -> Value {
            match v {
                Value::Null => Value::Null,
                Value::Bool(b) => Value::Bool(*b),
                Value::Number(n) => Value::Number(n.clone()),
                Value::String(s) => Value::String(red.redact_str(s)),
                Value::Array(a) => Value::Array(a.iter().map(|x| walk(red, x)).collect()),
                Value::Object(o) => {
                    let mut out = serde_json::Map::new();
                    for (k, vv) in o {
                        out.insert(k.clone(), walk(red, vv));
                    }
                    Value::Object(out)
                }
            }
        }
        walk(self, v)
    }
}
