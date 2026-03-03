use diycad_format::{
    open_package, save_package, Document, Entrypoints, Manifest, NestJob, OpenOptions, SaveOptions,
    Unit,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use sha2::{Digest, Sha256};
use std::fs;
use tempfile::tempdir;

fn minimal_manifest(schema_version: i64) -> Manifest {
    Manifest {
        schema_version,
        app_version: "0.0.0-test".to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        unit: Unit::Mm,
        entrypoints: Entrypoints {
            document: "document.json".to_string(),
        },
        features: None,
        determinism_tag: None,
        content_manifest: None,
    }
}

fn minimal_doc() -> Document {
    Document {
        id: "doc1".to_string(),
        name: "Minimal".to_string(),
        unit: Unit::Mm,
        entities: vec![],
        parts_index: vec![],
        nest_jobs_index: vec![],
        created_at: None,
        updated_at: None,
    }
}

#[test]
fn short_seeded_fuzz_no_panic() {
    let td = tempdir().expect("tmp");
    let base = td.path().join("base.diycad");
    save_package(
        &base,
        SaveOptions::default(),
        minimal_manifest(1),
        minimal_doc(),
        vec![],
        Vec::<NestJob>::new(),
        vec![],
    )
    .expect("save");

    let base_bytes = fs::read(&base).expect("read base");
    let mut rng = StdRng::seed_from_u64(0x5eed_1407);

    for case_idx in 0..50 {
        let mut mutated = base_bytes.clone();
        let edit_count = rng.random_range(1..=8).min(mutated.len().max(1));
        for _ in 0..edit_count {
            if mutated.is_empty() {
                mutated.push(rng.random());
                continue;
            }
            let at = rng.random_range(0..mutated.len());
            mutated[at] = rng.random();
        }

        let mut hasher = Sha256::new();
        hasher.update(&mutated);
        let hash = hex::encode(hasher.finalize());
        let name = format!("fuzz_{case_idx:03}_{}.diycad", &hash[..8]);
        let p = td.path().join(name);
        fs::write(&p, &mutated).expect("write case");

        let _ = open_package(&p, OpenOptions::default());

        eprintln!(
            "[fuzz] case={case_idx} hash8={} path={}",
            &hash[..8],
            p.display()
        );
    }
}
