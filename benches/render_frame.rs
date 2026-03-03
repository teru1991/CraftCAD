#![forbid(unsafe_code)]

mod perf_common;
use perf_common::*;

use std::path::Path;

fn main() {
    let dataset_id = "heavy_sample_v1";
    let dataset = load_dataset_json(Path::new("tests/datasets/heavy_sample_v1/heavy_sample_v1.json"));

    let s = session(dataset_id);

    craftcad_perf::perf_span!("render.frame.total", {
        craftcad_perf::perf_span!("render.frame.build_primitives", {
            let mut sum: u64 = 0;
            if let Some(ents) = dataset.get("entities").and_then(|v| v.as_array()) {
                for e in ents {
                    let pts = e.get("points").and_then(|v| v.as_u64()).unwrap_or(1);
                    sum = sum.wrapping_add(pts);
                }
            }
            std::hint::black_box(sum);
        });

        craftcad_perf::perf_span!("render.frame.draw", {});
    });

    let report = s.finish();
    write_report("bench_render_frame_heavy_sample_v1", &report);
}
