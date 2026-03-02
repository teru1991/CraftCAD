use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use thiserror::Error;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReasonCode {
    IO_FORMAT_NOT_REGISTERED,
    IO_LIMIT_BYTES_EXCEEDED,

    IO_SANITIZE_NONFINITE,
    IO_PATH_CLOSED_BY_EPS,
    IO_NORMALIZE_ROUNDED,

    // support_matrix.json parity (SSOT)
    IO_TEXT_FALLBACK_FONT,
    IO_FALLBACK_024,
    IO_UNSUPPORTED_ENTITY_DXF_SPLINE,
    IO_HATCH_SIMPLIFIED,
    IO_IMAGE_REFERENCE_DROPPED,

    // PR3: approx/postprocess
    IO_CURVE_APPROX_APPLIED,
    IO_ORIGIN_SHIFTED,
    IO_PATH_JOIN_APPLIED,
    IO_DEDUP_REMOVED,
    IO_TINY_SEGMENT_REMOVED,
    IO_PATH_ORDER_OPTIMIZED,

    // PR4: io_json
    IO_PARSE_JSON_MALFORMED,
    IO_JSON_SCHEMA_INVALID,
    IO_JSON_SCHEMA_UNSUPPORTED_VERSION,

    IO_PARSE_SVG_MALFORMED,
    IO_SVG_LIMIT_NODES_EXCEEDED,
    IO_SVG_LIMIT_DEPTH_EXCEEDED,
    IO_SVG_EXTERNAL_REFERENCE_BLOCKED,
    IO_SVG_PATH_COMMAND_UNKNOWN,
    IO_UNIT_GUESSED,

    IO_PARSE_DXF_MALFORMED,
    IO_DXF_LIMIT_LINES_EXCEEDED,
    IO_DXF_LIMIT_GROUPS_EXCEEDED,
    IO_DXF_LIMIT_STRING_EXCEEDED,
    IO_DXF_ENTITY_UNKNOWN_DROPPED,

    IO_SUPPORT_MATRIX_FEATURE_MISSING,
    IO_SVG_ARC_CONVERTED,
    IO_DXF_SPLINE_CONVERTED,
    PERF_BUDGET_EXCEEDED,

    // Bridge / Save / Load
    SAVE_DIYCAD_WRITE_FAILED,
    SAVE_PART_IMPORT_FAILED,
    SAVE_GEOM_DROPPED,
    SAVE_TEXT_BEST_EFFORT,

    LOAD_DIYCAD_READ_FAILED,
    LOAD_PART_EXPORT_FAILED,
    LOAD_DIYCAD_EMPTY,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub reason: ReasonCode,
    pub message: String,
    pub hint: Option<String>,
    pub context: BTreeMap<String, String>,
    pub is_fatal: bool,
}

impl AppError {
    pub fn new(reason: ReasonCode, message: impl Into<String>) -> Self {
        Self {
            reason,
            message: message.into(),
            hint: None,
            context: BTreeMap::new(),
            is_fatal: false,
        }
    }

    pub fn fatal(mut self) -> Self {
        self.is_fatal = true;
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn with_context(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.context.insert(k.into(), v.into());
        self
    }
}

#[derive(Debug, Error)]
#[error("{reason:?}: {message}")]
pub struct AppErrorWrapper {
    pub reason: ReasonCode,
    pub message: String,
    pub hint: Option<String>,
    pub context: BTreeMap<String, String>,
}

impl From<AppError> for AppErrorWrapper {
    fn from(e: AppError) -> Self {
        Self {
            reason: e.reason,
            message: e.message,
            hint: e.hint,
            context: e.context,
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
