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

fn upper(s: &str) -> String {
    s.trim().to_ascii_uppercase()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    None,
    Header,
    Entities,
    Other,
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

    let max_lines = opts
        .limits
        .max_entities
        .saturating_mul(60)
        .saturating_add(512);
    let max_groups = opts
        .limits
        .max_entities
        .saturating_mul(120)
        .saturating_add(1024);
    let max_string_len = 4096usize;

    let mut lines = s.lines();
    let mut groups = Vec::new();
    let mut line_count = 0usize;

    loop {
        let Some(code_line) = lines.next() else { break };
        let Some(val_line) = lines.next() else {
            return Err(
                AppError::new(ReasonCode::IO_PARSE_DXF_MALFORMED, "odd number of lines").fatal(),
            );
        };

        line_count = line_count.saturating_add(2);
        if line_count > max_lines {
            return Err(AppError::new(
                ReasonCode::IO_DXF_LIMIT_LINES_EXCEEDED,
                "dxf line limit exceeded",
            )
            .with_context("max_lines", max_lines.to_string())
            .with_context("lines", line_count.to_string())
            .fatal());
        }
        if groups.len().saturating_add(1) > max_groups {
            return Err(AppError::new(
                ReasonCode::IO_DXF_LIMIT_GROUPS_EXCEEDED,
                "dxf group limit exceeded",
            )
            .with_context("max_groups", max_groups.to_string())
            .with_context("groups", groups.len().to_string())
            .fatal());
        }
        if val_line.len() > max_string_len {
            return Err(AppError::new(
                ReasonCode::IO_DXF_LIMIT_STRING_EXCEEDED,
                "dxf string too long",
            )
            .with_context("max_string_len", max_string_len.to_string())
            .with_context("len", val_line.len().to_string())
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

pub fn parse_header_insunits(groups: &[DxfGroup]) -> Option<i32> {
    let mut sec = Section::None;
    let mut i = 0usize;

    while i < groups.len() {
        let g = &groups[i];
        if g.code == 0 && upper(&g.value) == "SECTION" {
            if i + 1 < groups.len() && groups[i + 1].code == 2 {
                sec = match upper(&groups[i + 1].value).as_str() {
                    "HEADER" => Section::Header,
                    "ENTITIES" => Section::Entities,
                    _ => Section::Other,
                };
                i += 2;
                continue;
            }
        }
        if g.code == 0 && upper(&g.value) == "ENDSEC" {
            sec = Section::None;
            i += 1;
            continue;
        }

        if sec == Section::Header && g.code == 9 && g.value.trim() == "$INSUNITS" {
            for j in (i + 1)..(i + 12).min(groups.len()) {
                if groups[j].code == 70 {
                    if let Ok(v) = groups[j].value.trim().parse::<i32>() {
                        return Some(v);
                    }
                }
                if groups[j].code == 9 || groups[j].code == 0 {
                    break;
                }
            }
        }

        i += 1;
    }

    None
}

pub fn split_entities(groups: &[DxfGroup]) -> Vec<DxfEntity> {
    let mut ents: Vec<DxfEntity> = Vec::new();
    let mut sec = Section::None;

    let mut i = 0usize;
    while i < groups.len() {
        let g = &groups[i];

        if g.code == 0 && upper(&g.value) == "SECTION" {
            if i + 1 < groups.len() && groups[i + 1].code == 2 {
                sec = match upper(&groups[i + 1].value).as_str() {
                    "HEADER" => Section::Header,
                    "ENTITIES" => Section::Entities,
                    _ => Section::Other,
                };
                i += 2;
                continue;
            }
        }
        if g.code == 0 && upper(&g.value) == "ENDSEC" {
            sec = Section::None;
            i += 1;
            continue;
        }

        if sec != Section::Entities {
            i += 1;
            continue;
        }

        if g.code != 0 {
            i += 1;
            continue;
        }

        let kind = upper(&g.value);
        if kind == "POLYLINE" {
            let mut e = DxfEntity {
                kind: kind.clone(),
                layer: "0".into(),
                linetype: "CONTINUOUS".into(),
                groups: Vec::new(),
            };

            i += 1;
            while i < groups.len() {
                let gg = &groups[i];
                if gg.code == 0 {
                    break;
                }
                if gg.code == 8 {
                    e.layer = gg.value.trim().to_string();
                } else if gg.code == 6 {
                    e.linetype = gg.value.trim().to_string();
                }
                e.groups.push(gg.clone());
                i += 1;
            }

            while i < groups.len() {
                if groups[i].code != 0 {
                    i += 1;
                    continue;
                }
                let k = upper(&groups[i].value);
                if k == "VERTEX" {
                    i += 1;
                    while i < groups.len() {
                        let vg = &groups[i];
                        if vg.code == 0 {
                            break;
                        }
                        if vg.code == 10 || vg.code == 20 || vg.code == 42 {
                            e.groups.push(vg.clone());
                        }
                        i += 1;
                    }
                    continue;
                }
                if k == "SEQEND" {
                    i += 1;
                    while i < groups.len() {
                        if groups[i].code == 0 {
                            break;
                        }
                        i += 1;
                    }
                    break;
                }
                break;
            }

            ents.push(e);
            continue;
        }

        let mut e = DxfEntity {
            kind: kind.clone(),
            layer: "0".into(),
            linetype: "CONTINUOUS".into(),
            groups: Vec::new(),
        };

        i += 1;
        while i < groups.len() {
            let gg = &groups[i];
            if gg.code == 0 {
                break;
            }
            if gg.code == 8 {
                e.layer = gg.value.trim().to_string();
            } else if gg.code == 6 {
                e.linetype = gg.value.trim().to_string();
            }
            e.groups.push(gg.clone());
            i += 1;
        }

        ents.push(e);
    }

    ents
}
