use std::borrow::Cow;

use thiserror::Error;

pub type SecResult<T> = Result<T, SecError>;

/// Stable reason code strings (SSOT contract).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecCode {
    // Limits
    SecLimitExceeded,
    SecZipLimitExceeded,
    SecJsonDepthExceeded,
    SecStringTooLong,

    // Sandbox / paths / external refs
    SecPathTraversalBlocked,
    SecAbsolutePathBlocked,
    SecDevicePathBlocked,
    SecInvalidPathChars,
    SecPathTooDeep,
    SecExternalRefRejected,
    SecExternalRefStripped,

    // Consent
    SecConsentReset,

    // SSOT / rules
    SecSsotNotFound,
    SecSsotInvalid,
    SecRegexInvalid,
}

impl SecCode {
    pub fn as_str(self) -> &'static str {
        match self {
            SecCode::SecLimitExceeded => "SEC_LIMIT_EXCEEDED",
            SecCode::SecZipLimitExceeded => "SEC_ZIP_LIMIT_EXCEEDED",
            SecCode::SecJsonDepthExceeded => "SEC_JSON_DEPTH_EXCEEDED",
            SecCode::SecStringTooLong => "SEC_STRING_TOO_LONG",

            SecCode::SecPathTraversalBlocked => "SEC_PATH_TRAVERSAL_BLOCKED",
            SecCode::SecAbsolutePathBlocked => "SEC_ABSOLUTE_PATH_BLOCKED",
            SecCode::SecDevicePathBlocked => "SEC_DEVICE_PATH_BLOCKED",
            SecCode::SecInvalidPathChars => "SEC_INVALID_PATH_CHARS",
            SecCode::SecPathTooDeep => "SEC_PATH_TOO_DEEP",
            SecCode::SecExternalRefRejected => "SEC_EXTERNAL_REF_REJECTED",
            SecCode::SecExternalRefStripped => "SEC_EXTERNAL_REF_STRIPPED",

            SecCode::SecConsentReset => "SEC_CONSENT_RESET",

            SecCode::SecSsotNotFound => "SEC_SSOT_NOT_FOUND",
            SecCode::SecSsotInvalid => "SEC_SSOT_INVALID",
            SecCode::SecRegexInvalid => "SEC_REGEX_INVALID",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecWarning {
    pub code: SecCode,
    pub message: Cow<'static, str>,
}

#[derive(Debug, Error)]
#[error("{code}: {message}")]
pub struct SecError {
    pub code: SecCode,
    pub message: Cow<'static, str>,
}

impl std::fmt::Display for SecCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl SecError {
    pub fn new(code: SecCode, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
