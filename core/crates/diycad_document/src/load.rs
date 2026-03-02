use crate::errors::{DocumentError, DocumentWarning};
use drawing_model::{
    migrate_json_to_latest, validate_drawing_doc, DrawingDoc, DrawingMigrateError,
};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct LoadedDocument {
    pub document: Value,
    pub drawing: Option<DrawingDoc>,
    pub warnings: Vec<DocumentWarning>,
}

fn sort_warnings_stable(mut warnings: Vec<DocumentWarning>) -> Vec<DocumentWarning> {
    warnings.sort_by(|a, b| a.reason_code().cmp(b.reason_code()));
    warnings
}

pub fn load_document_json(json_text: &str) -> Result<LoadedDocument, DocumentError> {
    let mut root: Value = serde_json::from_str(json_text)
        .map_err(|e| DocumentError::JsonParseFailed(e.to_string()))?;
    let obj = root.as_object_mut().ok_or(DocumentError::NotObject)?;

    let mut warnings = vec![];
    let mut drawing: Option<DrawingDoc> = None;

    if let Some(raw_drawing) = obj.get("drawing").cloned() {
        match raw_drawing {
            Value::Null => {}
            Value::Object(_) => {
                let migrated = match migrate_json_to_latest(raw_drawing) {
                    Ok(v) => v,
                    Err(e) => {
                        match e {
                            DrawingMigrateError::UnsupportedSchemaVersion(_) => {
                                warnings.push(DocumentWarning::DrawingUnsupportedSchemaVersion)
                            }
                            _ => warnings.push(DocumentWarning::DrawingParseFailed),
                        }
                        warnings.push(DocumentWarning::DrawingSalvagedReadOnly);
                        obj.remove("drawing");
                        return Ok(LoadedDocument {
                            document: root,
                            drawing: None,
                            warnings: sort_warnings_stable(warnings),
                        });
                    }
                };

                let parsed = match serde_json::from_value::<DrawingDoc>(migrated) {
                    Ok(d) => d,
                    Err(_) => {
                        warnings.push(DocumentWarning::DrawingParseFailed);
                        warnings.push(DocumentWarning::DrawingSalvagedReadOnly);
                        obj.remove("drawing");
                        return Ok(LoadedDocument {
                            document: root,
                            drawing: None,
                            warnings: sort_warnings_stable(warnings),
                        });
                    }
                };

                if validate_drawing_doc(&parsed).is_err() {
                    warnings.push(DocumentWarning::DrawingValidateFailed);
                    warnings.push(DocumentWarning::DrawingSalvagedReadOnly);
                    obj.remove("drawing");
                    return Ok(LoadedDocument {
                        document: root,
                        drawing: None,
                        warnings: sort_warnings_stable(warnings),
                    });
                }

                drawing = Some(parsed);
            }
            _ => {
                warnings.push(DocumentWarning::DrawingParseFailed);
                warnings.push(DocumentWarning::DrawingSalvagedReadOnly);
                obj.remove("drawing");
            }
        }
    }

    Ok(LoadedDocument {
        document: root,
        drawing,
        warnings: sort_warnings_stable(warnings),
    })
}
