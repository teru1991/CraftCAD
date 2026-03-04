use regex::Regex;

use crate::limits::Limits;
use crate::reasons::{SecCode, SecError, SecResult, SecWarning};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalRefPolicy {
    Reject,
    Strip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvgExternalRefAction {
    Rejected,
    Stripped(u32),
    Noop,
}

#[derive(Debug, Clone)]
pub struct PathValidationContext {
    pub max_depth: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedRelPath(String);

impl NormalizedRelPath {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Sandbox {
    pub policy_external_ref: ExternalRefPolicy,
}

impl Sandbox {
    pub fn new(policy_external_ref: ExternalRefPolicy) -> Self {
        Self {
            policy_external_ref,
        }
    }

    pub fn default_from_ssot() -> Self {
        Self {
            policy_external_ref: ExternalRefPolicy::Reject,
        }
    }

    pub fn normalize_rel_path(
        &self,
        ctx: PathValidationContext,
        raw: &str,
    ) -> SecResult<NormalizedRelPath> {
        if raw.is_empty() {
            return Err(SecError::new(SecCode::SecInvalidPathChars, "empty path"));
        }
        if raw.as_bytes().contains(&0) {
            return Err(SecError::new(SecCode::SecInvalidPathChars, "NUL in path"));
        }

        let s = raw.replace('\\', "/");
        if s.starts_with('/') {
            return Err(SecError::new(
                SecCode::SecAbsolutePathBlocked,
                "absolute unix path blocked",
            ));
        }
        if s.starts_with("//") {
            return Err(SecError::new(
                SecCode::SecDevicePathBlocked,
                "UNC/device path blocked",
            ));
        }
        if is_windows_drive_abs(&s) {
            return Err(SecError::new(
                SecCode::SecAbsolutePathBlocked,
                "windows drive absolute path blocked",
            ));
        }

        let mut out: Vec<&str> = Vec::new();
        for seg in s.split('/') {
            if seg.is_empty() || seg == "." {
                continue;
            }
            if seg == ".." {
                return Err(SecError::new(
                    SecCode::SecPathTraversalBlocked,
                    "path traversal '..' blocked",
                ));
            }
            if seg.chars().any(|c| c.is_control()) {
                return Err(SecError::new(
                    SecCode::SecInvalidPathChars,
                    "control char in path segment",
                ));
            }
            out.push(seg);
            if (out.len() as u64) > ctx.max_depth {
                return Err(SecError::new(
                    SecCode::SecPathTooDeep,
                    "path depth exceeded",
                ));
            }
        }
        if out.is_empty() {
            return Err(SecError::new(
                SecCode::SecInvalidPathChars,
                "path normalized to empty",
            ));
        }
        Ok(NormalizedRelPath(out.join("/")))
    }

    pub fn reject_external_ref(&self, href: &str) -> SecResult<()> {
        let h = href.trim();
        if h.is_empty() {
            return Ok(());
        }
        let hl = h.to_ascii_lowercase();
        if hl.starts_with("http://")
            || hl.starts_with("https://")
            || hl.starts_with("file:")
            || hl.starts_with("//")
        {
            return Err(SecError::new(
                SecCode::SecExternalRefRejected,
                "external ref blocked",
            ));
        }
        if h.starts_with('/') || is_windows_drive_abs(h) || h.starts_with("\\\\") {
            return Err(SecError::new(
                SecCode::SecExternalRefRejected,
                "path-like external ref blocked",
            ));
        }
        Ok(())
    }

    pub fn handle_svg_external_refs(
        &self,
        limits: &Limits,
        svg: &str,
    ) -> SecResult<(String, SvgExternalRefAction, Option<SecWarning>)> {
        let max = limits.max_import_bytes.min(16 * 1024 * 1024);
        let mut s = svg;
        if (svg.len() as u64) > max {
            s = &svg[..(max as usize)];
        }

        let re_href = href_re()?;
        let re_import = import_re()?;

        let mut found: Vec<String> = Vec::new();
        for cap in re_href.captures_iter(s) {
            let v = cap
                .get(2)
                .or_else(|| cap.get(3))
                .map(|m| m.as_str())
                .unwrap_or("");
            if looks_external(v) {
                found.push(v.to_string());
            }
        }
        for cap in re_import.captures_iter(s) {
            let v = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if looks_external(v) {
                found.push(v.to_string());
            }
        }

        if found.is_empty() {
            return Ok((svg.to_string(), SvgExternalRefAction::Noop, None));
        }

        match self.policy_external_ref {
            ExternalRefPolicy::Reject => Err(SecError::new(
                SecCode::SecExternalRefRejected,
                "svg contains external references",
            )),
            ExternalRefPolicy::Strip => {
                let mut sanitized = svg.to_string();
                let mut stripped: u32 = 0;
                sanitized = re_href
                    .replace_all(&sanitized, |caps: &regex::Captures| {
                        let full = caps.get(0).map(|m| m.as_str()).unwrap_or("");
                        let v = caps
                            .get(2)
                            .or_else(|| caps.get(3))
                            .map(|m| m.as_str())
                            .unwrap_or("");
                        if looks_external(v) {
                            stripped += 1;
                            if full.to_ascii_lowercase().contains("xlink:href") {
                                "xlink:href=\"\"".to_string()
                            } else {
                                "href=\"\"".to_string()
                            }
                        } else {
                            full.to_string()
                        }
                    })
                    .to_string();

                sanitized = re_import
                    .replace_all(&sanitized, |caps: &regex::Captures| {
                        let full = caps.get(0).map(|m| m.as_str()).unwrap_or("");
                        let v = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                        if looks_external(v) {
                            stripped += 1;
                            String::new()
                        } else {
                            full.to_string()
                        }
                    })
                    .to_string();

                let warn = Some(SecWarning {
                    code: SecCode::SecExternalRefStripped,
                    message: "svg external references were stripped".into(),
                });
                Ok((sanitized, SvgExternalRefAction::Stripped(stripped), warn))
            }
        }
    }
}

fn href_re() -> SecResult<&'static Regex> {
    static RE_HREF: once_cell::sync::Lazy<Result<Regex, String>> =
        once_cell::sync::Lazy::new(|| {
            Regex::new(r#"(?i)\b(?:xlink:href|href)\s*=\s*("([^"]*)"|'([^']*)')"#)
                .map_err(|e| e.to_string())
        });
    RE_HREF
        .as_ref()
        .map_err(|e| SecError::new(SecCode::SecRegexInvalid, format!("href regex invalid: {e}")))
}

fn import_re() -> SecResult<&'static Regex> {
    static RE_IMPORT: once_cell::sync::Lazy<Result<Regex, String>> =
        once_cell::sync::Lazy::new(|| {
            Regex::new(r#"(?i)@import\s+url\(\s*['\"]?([^'\")\s]+)['\"]?\s*\)"#)
                .map_err(|e| e.to_string())
        });
    RE_IMPORT.as_ref().map_err(|e| {
        SecError::new(
            SecCode::SecRegexInvalid,
            format!("import regex invalid: {e}"),
        )
    })
}

fn is_windows_drive_abs(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() >= 3
        && b[1] == b':'
        && (b[2] == b'/' || b[2] == b'\\')
        && ((b[0] as char).is_ascii_alphabetic())
}

fn looks_external(v: &str) -> bool {
    let t = v.trim();
    if t.is_empty() {
        return false;
    }
    let tl = t.to_ascii_lowercase();
    tl.starts_with("http://")
        || tl.starts_with("https://")
        || tl.starts_with("file:")
        || tl.starts_with("//")
}
