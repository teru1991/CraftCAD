use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReasonCode(pub Cow<'static, str>);

impl ReasonCode {
    pub const fn new(code: &'static str) -> Self {
        Self(Cow::Borrowed(code))
    }

    pub fn owned(code: impl Into<String>) -> Self {
        Self(Cow::Owned(code.into()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppError {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub hint: Option<String>,
    pub context: Vec<(String, String)>,
}

impl AppError {
    pub fn new(code: ReasonCode, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            code: code.as_str().to_owned(),
            severity,
            message: message.into(),
            hint: None,
            context: vec![],
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.push((key.into(), value.into()));
        self
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub trait ResultExt<T> {
    fn with_reason(
        self,
        code: ReasonCode,
        severity: Severity,
        message: impl Into<String>,
    ) -> AppResult<T>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn with_reason(
        self,
        code: ReasonCode,
        severity: Severity,
        message: impl Into<String>,
    ) -> AppResult<T> {
        self.map_err(|e| AppError::new(code, severity, format!("{}: {}", message.into(), e)))
    }
}
