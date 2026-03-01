use craftcad_geom2d::aabb::Aabb;
use craftcad_sketch::query::SpatialIndex;

#[test]
fn query_returns_sorted_unique_hits() {
    let aabbs = vec![
        Aabb {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
        },
        Aabb {
            min_x: 0.5,
            min_y: 0.5,
            max_x: 2.0,
            max_y: 2.0,
        },
    ];
    let idx = SpatialIndex::build(1.0, aabbs);
    let hits = idx.query_aabb(Aabb {
        min_x: 0.75,
        min_y: 0.75,
        max_x: 0.9,
        max_y: 0.9,
    });
    assert_eq!(hits, vec![0, 1]);
}
