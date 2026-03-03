#![forbid(unsafe_code)]

mod perf_common;
use perf_common::*;

use std::path::Path;

fn main() {
    let dataset_id = "heavy_sample_v1";
    let dataset = load_dataset_json(Path::new("tests/datasets/heavy_sample_v1/heavy_sample_v1.json"));

    let s = session(dataset_id);

    craftcad_perf::perf_span!("io.import.total", {
        craftcad_perf::perf_span!("io.import.parse", {
            let _ = dataset
                .get("entities")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
        });
        craftcad_perf::perf_span!("io.import.map", {
            let mut acc: u64 = 0;
            if let Some(ents) = dataset.get("entities").and_then(|v| v.as_array()) {
                for e in ents {
                    acc = acc.wrapping_add(e.get("id").and_then(|v| v.as_u64()).unwrap_or(0));
                }
            }
            std::hint::black_box(acc);
        });
    });

    craftcad_perf::perf_span!("io.export.total", {
        craftcad_perf::perf_span!("io.export.serialize", {
            let _ = serde_json::to_vec(&dataset).expect("serialize");
        });
    });

    let report = s.finish();
    write_report("bench_io_import_export_heavy_sample_v1", &report);
}
