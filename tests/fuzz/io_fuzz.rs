use craftcad_io::options::ImportOptions;
use craftcad_io::IoEngine;
use craftcad_io_dxf::DxfIo;
use craftcad_io_json::JsonIo;
use craftcad_io_svg::SvgIo;

fn rnd(seed: &mut u64) -> u8 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    (*seed & 0xFF) as u8
}

fn gen_bytes(seed: u64, max_len: usize) -> Vec<u8> {
    let mut s = seed;
    let len = (seed as usize % max_len).max(1);
    let mut v = Vec::with_capacity(len);
    for _ in 0..len {
        v.push(rnd(&mut s));
    }
    v
}

#[test]
fn fuzz_import_no_panic() {
    let eng = IoEngine::new()
        .register_importer(Box::new(JsonIo::new()))
        .register_importer(Box::new(SvgIo::new()))
        .register_importer(Box::new(DxfIo::new()));

    let mut opts = ImportOptions::default_for_tests();
    opts.limits.max_bytes = 16 * 1024;

    for (fmt, base_seed) in [("json", 1u64), ("svg", 10u64), ("dxf", 100u64)] {
        for i in 0..100u64 {
            let seed = base_seed + i;
            let bytes = gen_bytes(seed, opts.limits.max_bytes);
            let _ = eng.import(fmt, &bytes, &opts);
        }
    }
}
