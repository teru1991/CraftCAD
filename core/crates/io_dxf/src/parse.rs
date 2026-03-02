use craftcad_io::options::ImportOptions;
use craftcad_io::reasons::{AppError, AppResult, ReasonCode};

#[derive(Debug, Clone)]
pub struct DxfGroup {
    pub code: i32,
    pub value: String,
}
#[derive(Debug, Clone)]
pub struct DxfEntity {
    pub kind: String,
    pub layer: String,
    pub linetype: String,
    pub groups: Vec<DxfGroup>,
}

pub fn parse_dxf_groups(bytes: &[u8], opts: &ImportOptions) -> AppResult<Vec<DxfGroup>> {
    let s = std::str::from_utf8(bytes).map_err(|e| {
        AppError::new(
            ReasonCode::IO_PARSE_DXF_MALFORMED,
            "dxf must be utf-8 (for now)",
        )
        .with_context("error", e.to_string())
        .fatal()
    })?;
    let max_lines = (opts.limits.max_entities * 4).max(50_000);
    let max_groups = (opts.limits.max_entities * 2).max(100_000);
    let max_string_len = 4096usize;
    let mut lines = s.lines();
    let mut groups = Vec::new();
    let mut line_count = 0usize;

    loop {
        let Some(code_line) = lines.next() else {
            break;
        };
        let Some(val_line) = lines.next() else {
            return Err(
                AppError::new(ReasonCode::IO_PARSE_DXF_MALFORMED, "odd number of lines").fatal(),
            );
        };
        line_count += 2;
        if line_count > max_lines {
            return Err(AppError::new(
                ReasonCode::IO_DXF_LIMIT_LINES_EXCEEDED,
                "dxf line limit exceeded",
            )
            .fatal());
        }
        if groups.len() + 1 > max_groups {
            return Err(AppError::new(
                ReasonCode::IO_DXF_LIMIT_GROUPS_EXCEEDED,
                "dxf group limit exceeded",
            )
            .fatal());
        }
        if val_line.len() > max_string_len {
            return Err(AppError::new(
                ReasonCode::IO_DXF_LIMIT_STRING_EXCEEDED,
                "dxf string too long",
            )
            .fatal());
        }
        let code = code_line.trim().parse::<i32>().map_err(|_| {
            AppError::new(ReasonCode::IO_PARSE_DXF_MALFORMED, "invalid group code")
                .with_context("code_line", code_line)
                .fatal()
        })?;
        groups.push(DxfGroup {
            code,
            value: val_line.to_string(),
        });
    }

    Ok(groups)
}

pub fn split_entities(groups: &[DxfGroup]) -> Vec<DxfEntity> {
    let mut ents = Vec::new();
    let mut cur: Option<DxfEntity> = None;
    for g in groups {
        if g.code == 0 {
            if let Some(e) = cur.take() {
                ents.push(e);
            }
            cur = Some(DxfEntity {
                kind: g.value.trim().to_string(),
                layer: "0".into(),
                linetype: "CONTINUOUS".into(),
                groups: Vec::new(),
            });
            continue;
        }
        if let Some(e) = cur.as_mut() {
            if g.code == 8 {
                e.layer = g.value.trim().to_string();
            } else if g.code == 6 {
                e.linetype = g.value.trim().to_string();
            }
            e.groups.push(g.clone());
        }
    }
    if let Some(e) = cur.take() {
        ents.push(e);
    }
    ents
}
