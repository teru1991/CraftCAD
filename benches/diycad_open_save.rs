#![forbid(unsafe_code)]

mod perf_common;
use perf_common::*;

use std::path::Path;

fn main() {
    let dataset_id = "heavy_sample_v1";
    let dataset = load_dataset_json(Path::new(
        "tests/datasets/heavy_sample_v1/heavy_sample_v1.json",
    ));

    let s = session(dataset_id);

    craftcad_perf::perf_span!("diycad.open.total", {
        craftcad_perf::perf_span!("diycad.open.preflight", {
            let _ = dataset
                .get("schema_version")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
        });
        craftcad_perf::perf_span!("diycad.open.salvage.parts", {
            let mut n: u64 = 0;
            if let Some(ents) = dataset.get("entities").and_then(|v| v.as_array()) {
                for e in ents {
                    if e.get("kind").and_then(|v| v.as_str()).is_some() {
                        n = n.wrapping_add(1);
                    }
                }
            }
            std::hint::black_box(n);
        });
    });

    craftcad_perf::perf_span!("diycad.save.total", {
        craftcad_perf::perf_span!("diycad.save.serialize", {
            let _ = serde_json::to_vec(&dataset).expect("serialize");
        });
    });

    let report = s.finish();
    write_report("bench_diycad_open_save_heavy_sample_v1", &report);
}
