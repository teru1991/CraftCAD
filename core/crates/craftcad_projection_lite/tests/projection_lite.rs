use craftcad_projection_lite::{project_to_sheet_lite, sheet_hash_hex, Aabb, PartBox, ViewLite};
use uuid::Uuid;

fn part(id: &str, aabb: Aabb) -> PartBox {
    PartBox {
        part_id: Uuid::parse_str(id).expect("uuid"),
        aabb,
    }
}

#[test]
fn determinism_same_hash_with_shuffled_input() {
    let a = part(
        "00000000-0000-0000-0000-0000000000a1",
        Aabb {
            min_x: 0.0,
            min_y: 0.0,
            min_z: 0.0,
            max_x: 10.0,
            max_y: 20.0,
            max_z: 3.0,
        },
    );
    let b = part(
        "00000000-0000-0000-0000-0000000000b2",
        Aabb {
            min_x: 5.0,
            min_y: 5.0,
            min_z: 0.0,
            max_x: 15.0,
            max_y: 10.0,
            max_z: 6.0,
        },
    );

    for view in [ViewLite::Front, ViewLite::Top, ViewLite::Side] {
        let s1 = project_to_sheet_lite(view, vec![a.clone(), b.clone()]);
        let s2 = project_to_sheet_lite(view, vec![b.clone(), a.clone()]);
        assert_eq!(sheet_hash_hex(&s1), sheet_hash_hex(&s2));
    }
}

#[test]
fn outline_is_closed_rectangle_with_five_points() {
    let s = project_to_sheet_lite(
        ViewLite::Top,
        vec![part(
            "00000000-0000-0000-0000-0000000000a1",
            Aabb {
                min_x: 1.0,
                min_y: 2.0,
                min_z: 3.0,
                max_x: 11.0,
                max_y: 22.0,
                max_z: 33.0,
            },
        )],
    );
    let outline = &s.items[0].outline;
    assert_eq!(outline.len(), 5);
    assert_eq!(outline.first(), outline.last());
}

#[test]
fn nan_inf_handling_is_deterministic() {
    let p = part(
        "00000000-0000-0000-0000-0000000000a1",
        Aabb {
            min_x: f64::NAN,
            min_y: f64::INFINITY,
            min_z: f64::NEG_INFINITY,
            max_x: 1.0,
            max_y: 2.0,
            max_z: 3.0,
        },
    );

    let s1 = project_to_sheet_lite(ViewLite::Front, vec![p.clone()]);
    let s2 = project_to_sheet_lite(ViewLite::Front, vec![p]);
    assert_eq!(sheet_hash_hex(&s1), sheet_hash_hex(&s2));
    assert_eq!(s1.items[0].outline[0].x, 0.0);
}
