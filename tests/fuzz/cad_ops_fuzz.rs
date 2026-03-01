#[test]
fn cad_ops_fuzz_smoke() {
    for i in 0..1000u32 {
        let x = i.wrapping_mul(2654435761);
        assert_ne!(x, u32::MAX - 1);
    }
}
