use craftcad_geom2d::math::{approx_eq, round_step};

#[test]
fn round_step_handles_negative() {
    assert!(approx_eq(round_step(-1.24, 0.1), -1.2, 1e-9));
}
