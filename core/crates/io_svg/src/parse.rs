use craftcad_io::options::ImportOptions;
use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
use quick_xml::events::Event;
use quick_xml::Reader;

#[derive(Debug, Clone)]
pub struct SvgNode {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<SvgNode>,
    pub text: Option<String>,
}

#[derive(Debug)]
pub struct SvgDom {
    pub root: SvgNode,
    pub warnings: Vec<AppError>,
}

fn truncate_for_context(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

fn is_external_ref_attr(k: &str) -> bool {
    k == "href" || k == "xlink:href"
}

pub fn parse_svg_dom(bytes: &[u8], opts: &ImportOptions) -> AppResult<SvgDom> {
    if bytes.len() > opts.limits.max_bytes {
        return Err(AppError::new(ReasonCode::IO_LIMIT_BYTES_EXCEEDED, "input too large").fatal());
    }

    let mut reader = Reader::from_reader(bytes);
    reader.trim_text(true);

    let max_nodes = opts.limits.max_entities.max(1);
    let max_depth = opts.limits.max_depth.max(1);

    let mut buf = Vec::new();
    let mut stack: Vec<SvgNode> = Vec::new();
    let mut nodes_count = 0usize;
    let mut warnings: Vec<AppError> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                nodes_count += 1;
                if nodes_count > max_nodes {
                    return Err(AppError::new(
                        ReasonCode::IO_SVG_LIMIT_NODES_EXCEEDED,
                        "svg nodes limit exceeded",
                    )
                    .with_context("max_nodes", max_nodes.to_string())
                    .with_context("nodes", nodes_count.to_string())
                    .fatal());
                }
                if stack.len() + 1 > max_depth {
                    return Err(AppError::new(
                        ReasonCode::IO_SVG_LIMIT_DEPTH_EXCEEDED,
                        "svg depth limit exceeded",
                    )
                    .with_context("max_depth", max_depth.to_string())
                    .with_context("depth", (stack.len() + 1).to_string())
                    .fatal());
                }

                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut attrs = Vec::new();

                for a in e.attributes().flatten() {
                    let k = String::from_utf8_lossy(a.key.as_ref()).to_string();
                    let v = a
                        .unescape_value()
                        .map_err(|err| {
                            AppError::new(
                                ReasonCode::IO_PARSE_SVG_MALFORMED,
                                "svg attribute decode failed",
                            )
                            .with_context("error", err.to_string())
                            .fatal()
                        })?
                        .to_string();

                    if is_external_ref_attr(&k) && !v.is_empty() {
                        warnings.push(
                            AppError::new(
                                ReasonCode::IO_IMAGE_REFERENCE_DROPPED,
                                "external reference blocked (href/xlink:href)",
                            )
                            .with_context("attr", k.clone())
                            .with_context("value", truncate_for_context(&v, 128)),
                        );
                        attrs.push((k, String::new()));
                    } else {
                        attrs.push((k, v));
                    }
                }

                stack.push(SvgNode {
                    name,
                    attrs,
                    children: vec![],
                    text: None,
                });
            }

            Ok(Event::Empty(e)) => {
                nodes_count += 1;
                if nodes_count > max_nodes {
                    return Err(AppError::new(
                        ReasonCode::IO_SVG_LIMIT_NODES_EXCEEDED,
                        "svg nodes limit exceeded",
                    )
                    .with_context("max_nodes", max_nodes.to_string())
                    .with_context("nodes", nodes_count.to_string())
                    .fatal());
                }

                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut attrs = Vec::new();

                for a in e.attributes().flatten() {
                    let k = String::from_utf8_lossy(a.key.as_ref()).to_string();
                    let v = a
                        .unescape_value()
                        .map_err(|err| {
                            AppError::new(
                                ReasonCode::IO_PARSE_SVG_MALFORMED,
                                "svg attribute decode failed",
                            )
                            .with_context("error", err.to_string())
                            .fatal()
                        })?
                        .to_string();

                    if is_external_ref_attr(&k) && !v.is_empty() {
                        warnings.push(
                            AppError::new(
                                ReasonCode::IO_IMAGE_REFERENCE_DROPPED,
                                "external reference blocked (href/xlink:href)",
                            )
                            .with_context("attr", k.clone())
                            .with_context("value", truncate_for_context(&v, 128)),
                        );
                        attrs.push((k, String::new()));
                    } else {
                        attrs.push((k, v));
                    }
                }

                let node = SvgNode {
                    name,
                    attrs,
                    children: vec![],
                    text: None,
                };

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else {
                    return Ok(SvgDom {
                        root: node,
                        warnings,
                    });
                }
            }

            Ok(Event::End(_)) => {
                let node = stack.pop().ok_or_else(|| {
                    AppError::new(ReasonCode::IO_PARSE_SVG_MALFORMED, "unbalanced svg tags").fatal()
                })?;
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else {
                    return Ok(SvgDom {
                        root: node,
                        warnings,
                    });
                }
            }

            Ok(Event::Text(t)) => {
                if let Some(cur) = stack.last_mut() {
                    let s = t
                        .unescape()
                        .map_err(|err| {
                            AppError::new(
                                ReasonCode::IO_PARSE_SVG_MALFORMED,
                                "svg text decode failed",
                            )
                            .with_context("error", err.to_string())
                            .fatal()
                        })?
                        .to_string();
                    if !s.is_empty() {
                        cur.text = Some(s);
                    }
                }
            }

            Ok(Event::Eof) => break,

            Err(e) => {
                return Err(
                    AppError::new(ReasonCode::IO_PARSE_SVG_MALFORMED, "malformed svg")
                        .with_context("error", e.to_string())
                        .fatal(),
                );
            }

            _ => {}
        }
        buf.clear();
    }

    Err(AppError::new(ReasonCode::IO_PARSE_SVG_MALFORMED, "unexpected eof").fatal())
}
