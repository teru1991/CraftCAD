use craftcad_perf::{perf_span, PerfSession};

fn main() {
    let session = PerfSession::start("heavy_sample_v1");
    perf_span!("render.frame", {
        let mut x = 0u64;
        for i in 0..50000 {
            x = x.wrapping_add(i);
        }
        std::hint::black_box(x);
    });
    let report = session.finish();
    println!("{}", report.to_json_pretty().unwrap());
}
