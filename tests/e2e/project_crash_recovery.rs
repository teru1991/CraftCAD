use diycad_format::{Document, Entrypoints, Manifest, NestJob, OpenOptions, Unit};
use recovery::{
    autosave_if_dirty, list_generations, restore_latest_best_effort, AutosaveContext,
    RecoveryPolicy,
};
use std::fs;
use tempfile::tempdir;

fn manifest() -> Manifest {
    Manifest {
        schema_version: 1,
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

fn doc(name: &str) -> Document {
    Document {
        id: "d".to_string(),
        name: name.to_string(),
        unit: Unit::Mm,
        entities: vec![],
        parts_index: vec![],
        nest_jobs_index: vec![],
        created_at: None,
        updated_at: None,
    }
}

#[test]
fn crash_during_autosave_keeps_last_good_and_restores() {
    let td = tempdir().unwrap();
    let autosave_dir = td.path().join("autosave");
    let mut ctx = AutosaveContext::new(
        autosave_dir.clone(),
        RecoveryPolicy {
            autosave_interval_sec: 1,
            max_generations: 5,
            max_total_bytes: 1024 * 1024 * 128,
            keep_last_good: true,
        },
    );

    let r1 = autosave_if_dirty(
        &mut ctx,
        true,
        manifest(),
        doc("A"),
        vec![],
        Vec::<NestJob>::new(),
    )
    .unwrap();
    assert!(r1.saved);
    let g1 = r1.generation_path.clone().unwrap();
    ctx.last_good = Some(g1);

    let tmp = autosave_dir.join("crash_partial.tmp");
    fs::create_dir_all(&autosave_dir).unwrap();
    fs::write(&tmp, b"partial").unwrap();

    let _r2 = autosave_if_dirty(
        &mut ctx,
        true,
        manifest(),
        doc("B"),
        vec![],
        Vec::<NestJob>::new(),
    )
    .unwrap();

    let gens = list_generations(&autosave_dir).unwrap();
    assert!(!gens.is_empty());

    let open = restore_latest_best_effort(&autosave_dir, OpenOptions::default()).unwrap();
    assert!(open.document.name == "A" || open.document.name == "B");

    assert!(!tmp.exists());
}
