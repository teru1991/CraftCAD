use crate::errors::DocumentError;
use drawing_model::{validate_drawing_doc, DrawingDoc};
use serde_json::Value;

pub fn save_document_json(
    mut document: Value,
    drawing: Option<&DrawingDoc>,
) -> Result<String, DocumentError> {
    let obj = document.as_object_mut().ok_or(DocumentError::NotObject)?;

    if let Some(d) = drawing {
        validate_drawing_doc(d).map_err(|e| {
            DocumentError::JsonSerializeFailed(format!("drawing validate failed: {e}"))
        })?;
        let drawing_value = serde_json::to_value(d)
            .map_err(|e| DocumentError::JsonSerializeFailed(e.to_string()))?;
        obj.insert("drawing".to_string(), drawing_value);
    } else {
        obj.remove("drawing");
    }

    serde_json::to_string_pretty(&document)
        .map_err(|e| DocumentError::JsonSerializeFailed(e.to_string()))
}
