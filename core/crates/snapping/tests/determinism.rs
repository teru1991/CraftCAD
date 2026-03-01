use craftcad_geom2d::Pt;
use craftcad_snapping::resolve::{resolve, SnapPolicy};
use craftcad_snapping::types::{SnapKind, SnapPoint};

#[test]
fn stable_ordering_is_deterministic() {
    let cands = vec![
        SnapPoint {
            kind: SnapKind::Grid,
            pt: Pt { x: 0.0, y: 0.0 },
            score: 1,
        },
        SnapPoint {
            kind: SnapKind::Endpoint,
            pt: Pt { x: 0.0, y: 0.0 },
            score: 2,
        },
        SnapPoint {
            kind: SnapKind::Center,
            pt: Pt { x: 0.2, y: 0.0 },
            score: 2,
        },
    ];
    let policy = SnapPolicy {
        max_dist: 10.0,
        angle_step_deg: 15.0,
        prefer: vec![],
    };
    let r1 = resolve(cands.clone(), Pt { x: 0.0, y: 0.0 }, &policy);
    let r2 = resolve(cands, Pt { x: 0.0, y: 0.0 }, &policy);
    assert_eq!(r1.candidates.len(), r2.candidates.len());
    assert_eq!(r1.snapped.unwrap().kind, SnapKind::Endpoint);
    for (a, b) in r1.candidates.iter().zip(r2.candidates.iter()) {
        assert_eq!(a.kind, b.kind);
        assert_eq!(a.pt.x, b.pt.x);
        assert_eq!(a.pt.y, b.pt.y);
    }
}
