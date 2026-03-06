use craftcad_estimate_lite::{compute_estimate_lite, estimate_hash_hex};
use craftcad_ssot::{
    FeatureGraphV1, GrainPolicyV1, ManufacturingOutline2dV1, MaterialCategoryV1, MaterialV1,
    PartV1, SsotV1,
};
use uuid::Uuid;

fn sample_ssot(parts: Vec<PartV1>) -> SsotV1 {
    let material_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    SsotV1::new(
        vec![MaterialV1 {
            material_id,
            category: MaterialCategoryV1::Unspecified,
            name: "plywood18".to_string(),
            thickness_mm: Some(18.0),
            grain_policy: GrainPolicyV1::None,
            kerf_mm: 2.0,
            margin_mm: 5.0,
            estimate_loss_factor: None,
        }],
        parts,
        FeatureGraphV1::empty(),
    )
}

fn make_part(id: &str, quantity: u32, outline: Option<ManufacturingOutline2dV1>) -> PartV1 {
    let material_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    PartV1 {
        part_id: Uuid::parse_str(id).unwrap(),
        name: "part".to_string(),
        material_id,
        quantity,
        manufacturing_outline_2d: outline,
        thickness_mm: Some(18.0),
        grain_direction: None,
        labels: vec![],
        feature_ids: vec![],
    }
}

#[test]
fn deterministic_hash_ignores_part_order() {
    let a = make_part(
        "00000000-0000-0000-0000-0000000000a1",
        1,
        Some(ManufacturingOutline2dV1 {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 100.0,
            max_y: 50.0,
        }),
    );
    let b = make_part(
        "00000000-0000-0000-0000-0000000000b2",
        2,
        Some(ManufacturingOutline2dV1 {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 10.0,
            max_y: 10.0,
        }),
    );

    let h1 = estimate_hash_hex(&compute_estimate_lite(&sample_ssot(vec![
        a.clone(),
        b.clone(),
    ])));
    let h2 = estimate_hash_hex(&compute_estimate_lite(&sample_ssot(vec![b, a])));
    assert_eq!(h1, h2);
}

#[test]
fn quantity_affects_area_and_parts_count() {
    let p = make_part(
        "00000000-0000-0000-0000-0000000000a1",
        3,
        Some(ManufacturingOutline2dV1 {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 10.0,
            max_y: 20.0,
        }),
    );
    let est = compute_estimate_lite(&sample_ssot(vec![p]));
    assert_eq!(est.items[0].parts_count, 3);
    assert_eq!(est.items[0].total_area_mm2, 600.0);
}

#[test]
fn missing_outline_counts_parts_but_zero_area() {
    let p = make_part("00000000-0000-0000-0000-0000000000a1", 4, None);
    let est = compute_estimate_lite(&sample_ssot(vec![p]));
    assert_eq!(est.items[0].parts_count, 4);
    assert_eq!(est.items[0].total_area_mm2, 0.0);
}

#[test]
fn nan_inf_outline_treated_as_zero() {
    let p = make_part(
        "00000000-0000-0000-0000-0000000000a1",
        1,
        Some(ManufacturingOutline2dV1 {
            min_x: f64::NAN,
            min_y: 0.0,
            max_x: f64::INFINITY,
            max_y: f64::NEG_INFINITY,
        }),
    );
    let est = compute_estimate_lite(&sample_ssot(vec![p]));
    assert_eq!(est.items[0].total_area_mm2, 0.0);
}
