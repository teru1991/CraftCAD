use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentWarning {
    DrawingSalvagedReadOnly,
    DrawingUnsupportedSchemaVersion,
    DrawingParseFailed,
    DrawingValidateFailed,
}

impl DocumentWarning {
    pub fn reason_code(&self) -> &'static str {
        match self {
            DocumentWarning::DrawingSalvagedReadOnly => "CAD_DRAWING_SALVAGED_READONLY",
            DocumentWarning::DrawingUnsupportedSchemaVersion => "CAD_DRAWING_UNSUPPORTED_SCHEMA",
            DocumentWarning::DrawingParseFailed => "CAD_DRAWING_PARSE_FAILED",
            DocumentWarning::DrawingValidateFailed => "CAD_DRAWING_VALIDATE_FAILED",
        }
    }
}

#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("document.json must be a JSON object")]
    NotObject,

    #[error("failed to parse document.json: {0}")]
    JsonParseFailed(String),

    #[error("failed to serialize document.json: {0}")]
    JsonSerializeFailed(String),
}
