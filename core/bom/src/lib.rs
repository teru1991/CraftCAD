use craftcad_serialize::{Document, Reason, ReasonCode, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitPolicy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundingPolicy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BomRow {
    pub part_id: String,
    pub part_name: String,
    pub qty: u32,
    pub material_name: String,
    pub thickness: f64,
    pub bbox_w: f64,
    pub bbox_h: f64,
    pub area: f64,
    pub perimeter: f64,
    pub grain_dir: Option<f64>,
    pub allow_rotate: bool,
    pub margin: f64,
    pub kerf: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BomTable {
    pub rows: Vec<BomRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvOptions {
    #[serde(default = "default_delim")]
    pub delimiter: char,
}
fn default_delim() -> char {
    ','
}

fn round(v: f64, d: i32) -> f64 {
    let s = 10_f64.powi(d);
    (v * s).round() / s
}

fn metrics(outer: &[craftcad_serialize::Vec2]) -> (f64, f64, f64, f64) {
    let mut minx = f64::INFINITY;
    let mut maxx = f64::NEG_INFINITY;
    let mut miny = f64::INFINITY;
    let mut maxy = f64::NEG_INFINITY;
    let mut area2 = 0.0;
    let mut peri = 0.0;
    for i in 0..outer.len() {
        let p = &outer[i];
        let q = &outer[(i + 1) % outer.len()];
        minx = minx.min(p.x);
        maxx = maxx.max(p.x);
        miny = miny.min(p.y);
        maxy = maxy.max(p.y);
        area2 += p.x * q.y - q.x * p.y;
        peri += ((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt();
    }
    (maxx - minx, maxy - miny, area2.abs() * 0.5, peri)
}

pub fn generate_bom(
    doc: &Document,
    _unit_policy: UnitPolicy,
    _rounding: RoundingPolicy,
) -> Result<BomTable> {
    let mats: HashMap<_, _> = doc.materials.iter().map(|m| (m.id, m)).collect();
    let mut rows = Vec::new();
    for p in &doc.parts {
        let m = mats.get(&p.material_id).ok_or_else(|| {
            let mut r = Reason::from_code(ReasonCode::MaterialNotFound);
            r.debug.insert("part_id".into(), serde_json::json!(p.id));
            r.debug
                .insert("material_id".into(), serde_json::json!(p.material_id));
            r
        })?;
        let (bw, bh, area, peri) = metrics(&p.outline.outer);
        let grain_deg = p.grain_dir.map(|r| round(r.to_degrees(), 1));
        rows.push(BomRow {
            part_id: p.id.to_string(),
            part_name: p.name.clone(),
            qty: p.quantity,
            material_name: m.name.clone(),
            thickness: round(p.thickness, 2),
            bbox_w: round(bw, 2),
            bbox_h: round(bh, 2),
            area: round(area, 2),
            perimeter: round(peri, 2),
            grain_dir: grain_deg,
            allow_rotate: p.allow_rotate,
            margin: round(p.margin, 2),
            kerf: round(p.kerf, 2),
        });
    }
    rows.sort_by(|a, b| {
        a.material_name
            .cmp(&b.material_name)
            .then_with(|| {
                a.thickness
                    .partial_cmp(&b.thickness)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.part_name.cmp(&b.part_name))
            .then_with(|| a.part_id.cmp(&b.part_id))
    });
    Ok(BomTable { rows })
}

fn csv_escape(s: &str, d: char) -> String {
    let needs = s.contains(d) || s.contains('"') || s.contains('\n') || s.contains('\r');
    if !needs {
        return s.to_string();
    }
    format!("\"{}\"", s.replace('"', "\"\""))
}

pub fn write_bom_csv(bom: &BomTable, options: CsvOptions) -> Result<Vec<u8>> {
    let d = options.delimiter;
    let mut out = Vec::new();
    out.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    let header = [
        "part_id",
        "part_name",
        "qty",
        "material_name",
        "thickness",
        "bbox_w",
        "bbox_h",
        "area",
        "perimeter",
        "grain_dir",
        "allow_rotate",
        "margin",
        "kerf",
    ]
    .join(&d.to_string());
    out.extend_from_slice(header.as_bytes());
    out.extend_from_slice(b"\r\n");
    for r in &bom.rows {
        let cols = vec![
            csv_escape(&r.part_id, d),
            csv_escape(&r.part_name, d),
            r.qty.to_string(),
            csv_escape(&r.material_name, d),
            format!("{:.2}", r.thickness),
            format!("{:.2}", r.bbox_w),
            format!("{:.2}", r.bbox_h),
            format!("{:.2}", r.area),
            format!("{:.2}", r.perimeter),
            r.grain_dir.map(|v| format!("{:.1}", v)).unwrap_or_default(),
            r.allow_rotate.to_string(),
            format!("{:.2}", r.margin),
            format!("{:.2}", r.kerf),
        ];
        out.extend_from_slice(cols.join(&d.to_string()).as_bytes());
        out.extend_from_slice(b"\r\n");
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use craftcad_serialize::{
        Document, Layer, Material, MaterialCategory, Part, Polygon2D, ProjectSettings, Vec2,
    };
    use uuid::Uuid;

    fn doc() -> Document {
        let mid1 = Uuid::new_v4();
        let mid2 = Uuid::new_v4();
        let mk = |id: Uuid, name: &str| Material {
            id,
            name: name.to_string(),
            category: MaterialCategory::Wood,
            thickness_mm: Some(18.0),
            sheet_default: None,
            notes: String::new(),
        };
        Document {
            schema_version: 1,
            id: Uuid::new_v4(),
            units: "mm".into(),
            layers: vec![Layer {
                id: Uuid::new_v4(),
                name: "d".into(),
                visible: true,
                locked: false,
                editable: true,
            }],
            entities: vec![],
            jobs: vec![],
            materials: vec![mk(mid1, "A"), mk(mid2, "B")],
            settings: ProjectSettings::default(),
            parts: vec![
                Part {
                    id: Uuid::new_v4(),
                    name: "z".into(),
                    outline: Polygon2D {
                        outer: vec![
                            Vec2 { x: 0.0, y: 0.0 },
                            Vec2 { x: 2.0, y: 0.0 },
                            Vec2 { x: 2.0, y: 1.0 },
                        ],
                        holes: vec![],
                    },
                    thickness: 2.0,
                    quantity: 1,
                    material_id: mid2,
                    grain_dir: Some(0.0),
                    allow_rotate: true,
                    margin: 0.0,
                    kerf: 0.0,
                },
                Part {
                    id: Uuid::new_v4(),
                    name: "a".into(),
                    outline: Polygon2D {
                        outer: vec![
                            Vec2 { x: 0.0, y: 0.0 },
                            Vec2 { x: 1.0, y: 0.0 },
                            Vec2 { x: 1.0, y: 1.0 },
                        ],
                        holes: vec![],
                    },
                    thickness: 1.0,
                    quantity: 1,
                    material_id: mid1,
                    grain_dir: None,
                    allow_rotate: true,
                    margin: 0.0,
                    kerf: 0.0,
                },
            ],
        }
    }

    #[test]
    fn ordering_rounding_and_csv_policy() {
        let bom = generate_bom(&doc(), UnitPolicy, RoundingPolicy).unwrap();
        assert_eq!(bom.rows[0].material_name, "A");
        let bytes = write_bom_csv(&bom, CsvOptions { delimiter: ',' }).unwrap();
        assert!(bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.contains("\r\n"));
        assert!(s.lines().count() >= 2);
    }
}
