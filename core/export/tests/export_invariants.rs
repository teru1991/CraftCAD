use craftcad_export::{
    compute_tiled_layout, export_svg, gauge_length_in_doc_units, Orientation, PageSize,
    SvgExportOptions, TiledPdfOptions,
};
use craftcad_serialize::{
    Document, Entity, Geom2D, GrainPolicy, Layer, Material, MaterialCategory, NestConstraints,
    NestJob, NestObjective, Part, PartRef, Polygon2D, ProjectSettings, SheetDef,
};
use uuid::Uuid;

fn doc(units: &str) -> Document {
    let layer_id = Uuid::new_v4();
    let mat = Uuid::new_v4();
    let part = Uuid::new_v4();
    Document {
        schema_version: 1,
        id: Uuid::nil(),
        units: units.into(),
        layers: vec![Layer {
            id: layer_id,
            name: "L".into(),
            visible: true,
            locked: false,
            editable: true,
        }],
        entities: vec![
            Entity {
                id: Uuid::from_u128(2),
                layer_id,
                geom: Geom2D::Line {
                    a: craftcad_serialize::Vec2 { x: 0.0, y: 0.0 },
                    b: craftcad_serialize::Vec2 { x: 300.0, y: 0.0 },
                },
                style: serde_json::json!({}),
                tags: vec![],
                meta: Default::default(),
            },
            Entity {
                id: Uuid::from_u128(1),
                layer_id,
                geom: Geom2D::Polyline {
                    pts: vec![
                        craftcad_serialize::Vec2 { x: 0.0, y: 0.0 },
                        craftcad_serialize::Vec2 { x: 0.0, y: 200.0 },
                        craftcad_serialize::Vec2 { x: 50.0, y: 200.0 },
                    ],
                    closed: false,
                },
                style: serde_json::json!({}),
                tags: vec![],
                meta: Default::default(),
            },
        ],
        parts: vec![Part {
            id: part,
            name: "P".into(),
            outline: Polygon2D {
                outer: vec![
                    craftcad_serialize::Vec2 { x: 0.0, y: 0.0 },
                    craftcad_serialize::Vec2 { x: 10.0, y: 0.0 },
                    craftcad_serialize::Vec2 { x: 10.0, y: 10.0 },
                ],
                holes: vec![],
            },
            thickness: 1.0,
            quantity: 1,
            material_id: mat,
            grain_dir: None,
            allow_rotate: true,
            margin: 0.0,
            kerf: 0.0,
        }],
        jobs: vec![NestJob {
            id: Uuid::new_v4(),
            sheet_defs: vec![SheetDef {
                id: Uuid::new_v4(),
                material_id: mat,
                width: 1000.0,
                height: 1000.0,
                quantity: 1,
            }],
            parts_ref: vec![PartRef {
                part_id: part,
                quantity_override: None,
            }],
            constraints: NestConstraints {
                global_margin: 0.0,
                global_kerf: 0.0,
                allow_rotate_default: true,
                no_go_zones: vec![],
                grain_policy: GrainPolicy::Ignore,
            },
            objective: NestObjective {
                w_utilization: 1.0,
                w_sheet_count: 1.0,
                w_cut_count: 1.0,
            },
            seed: 1,
            result: None,
            trace: None,
        }],
        materials: vec![Material {
            id: mat,
            name: "M".into(),
            category: MaterialCategory::Other,
            thickness_mm: None,
            sheet_default: None,
            notes: String::new(),
        }],
        settings: ProjectSettings::default(),
    }
}

#[test]
fn gauge_units_conversion() {
    assert_eq!(gauge_length_in_doc_units("mm").unwrap(), 100.0);
    assert!((gauge_length_in_doc_units("inch").unwrap() - (100.0 / 25.4)).abs() < 1e-9);
}

#[test]
fn deterministic_layout_invariants() {
    let d = doc("mm");
    let opt = TiledPdfOptions {
        page_size: PageSize::A4,
        orientation: Orientation::Portrait,
        ..Default::default()
    };
    let a = compute_tiled_layout(&d, &opt).unwrap();
    let b = compute_tiled_layout(&d, &opt).unwrap();
    assert_eq!(a, b);
    assert!(!a.page_labels.is_empty());
}

#[test]
fn svg_stable_order_and_precision() {
    let d = doc("mm");
    let s = export_svg(
        &d,
        &SvgExportOptions {
            precision: 2,
            include_parts: true,
            include_entities: true,
        },
    )
    .unwrap();
    let idx_poly = s
        .find("data-id=\"00000000-0000-0000-0000-000000000001\"")
        .unwrap();
    let idx_line = s
        .find("data-id=\"00000000-0000-0000-0000-000000000002\"")
        .unwrap();
    assert!(idx_poly < idx_line);
    assert!(s.contains("10.00"));
}
