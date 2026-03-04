use crate::options::ImportOptions;
use crate::reasons::{AppError, AppResult, ReasonCode};

pub fn preflight_bytes(format_id: &str, bytes: &[u8], opts: &ImportOptions) -> AppResult<()> {
    let len = bytes.len();

    if let Ok((sec_limits, _)) = crate::security_defaults() {
        if let Err(e) = sec_limits.check_bytes(security::LimitKind::ImportBytes, len as u64) {
            return Err(AppError::new(
                ReasonCode::IO_LIMIT_BYTES_EXCEEDED,
                format!("SEC_LIMIT_EXCEEDED: {}", e.message),
            )
            .with_context("format_id", format_id)
            .with_context("bytes", len.to_string())
            .with_context("max_bytes", sec_limits.max_import_bytes.to_string())
            .with_hint("入力を分割するか、要素数を減らして再試行してください。"));
        }
    }

    if len > opts.limits.max_bytes {
        return Err(AppError::new(
            ReasonCode::IO_LIMIT_BYTES_EXCEEDED,
            format!(
                "input too large: {} bytes (max {})",
                len, opts.limits.max_bytes
            ),
        )
        .with_context("format_id", format_id)
        .with_context("bytes", len.to_string())
        .with_context("max_bytes", opts.limits.max_bytes.to_string())
        .with_hint("入力を分割するか、要素数を減らして再試行してください。"));
    }
    Ok(())
}
