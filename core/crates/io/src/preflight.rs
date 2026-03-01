use crate::options::ImportOptions;
use craftcad_errors::{AppError, AppResult, ReasonCode, Severity};

pub fn check_bytes_len(bytes: &[u8], opts: &ImportOptions) -> AppResult<()> {
    if bytes.len() > opts.max_bytes {
        return Err(AppError::new(
            ReasonCode::new("IO_LIMIT_016"),
            Severity::Error,
            format!(
                "Input too large: {} > {} bytes",
                bytes.len(),
                opts.max_bytes
            ),
        ));
    }
    Ok(())
}

pub fn estimate_entity_count(text: &str) -> usize {
    text.matches('\n').count() + text.matches('{').count()
}

pub fn check_limits(
    bytes: &[u8],
    estimated_entities: usize,
    opts: &ImportOptions,
) -> AppResult<()> {
    check_bytes_len(bytes, opts)?;
    if estimated_entities > opts.max_entities {
        return Err(AppError::new(
            ReasonCode::new("IO_LIMIT_016"),
            Severity::Error,
            format!(
                "Estimated entities exceed max limit: {} > {}",
                estimated_entities, opts.max_entities
            ),
        ));
    }
    Ok(())
}
