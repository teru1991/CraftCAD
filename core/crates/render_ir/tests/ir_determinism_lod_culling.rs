#![forbid(unsafe_code)]

use craftcad_render_ir::*;
use serde_json::json;

fn prim(
    id: u64,
    layer: i32,
    style: &str,
    kind: PrimitiveKind,
    bounds: Aabb,
    complexity: u32,
) -> Primitive {
    Primitive {
        stable_id: id,
        layer_order: layer,
        style: StyleKey(style.to_string()),
        kind,
        bounds,
        complexity,
        payload: json!({"id": id}),
    }
}

#[test]
fn build_ir_is_deterministic_sorted_and_batched() {
    let cfg = PipelineConfig::default();
    let viewport = Viewport {
        world: Aabb {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 10.0,
            max_y: 10.0,
        },
    };

    let primitives = vec![
        prim(
            3,
            2,
            "b",
            PrimitiveKind::Line,
            Aabb {
                min_x: 1.0,
                min_y: 1.0,
                max_x: 2.0,
                max_y: 2.0,
            },
            10,
        ),
        prim(
            1,
            1,
            "a",
            PrimitiveKind::Line,
            Aabb {
                min_x: 1.0,
                min_y: 1.0,
                max_x: 2.0,
                max_y: 2.0,
            },
            10,
        ),
        prim(
            2,
            1,
            "a",
            PrimitiveKind::Line,
            Aabb {
                min_x: 1.0,
                min_y: 1.0,
                max_x: 2.0,
                max_y: 2.0,
            },
            10,
        ),
        prim(
            9,
            1,
            "a",
            PrimitiveKind::Circle,
            Aabb {
                min_x: 1.0,
                min_y: 1.0,
                max_x: 2.0,
                max_y: 2.0,
            },
            10,
        ),
    ];

    let ir1 = build_ir(&primitives, viewport, 1.0, (800, 600), None, &cfg);
    let j1 = serde_json::to_string(&ir1).unwrap();

    let ir2 = build_ir(&primitives, viewport, 1.0, (800, 600), None, &cfg);
    let j2 = serde_json::to_string(&ir2).unwrap();
    assert_eq!(j1, j2);

    let keys: Vec<String> = ir1
        .batches
        .iter()
        .map(|b| format!("{}:{}:{:?}", b.key.layer_order, b.key.style.0.clone(), b.key.kind))
        .collect();
    assert_eq!(keys, vec![
        "1:a:Line".to_string(),
        "1:a:Circle".to_string(),
        "2:b:Line".to_string(),
    ]);
}

#[test]
fn viewport_culling_excludes_outside_primitives() {
    let cfg = PipelineConfig::default();
    let viewport = Viewport {
        world: Aabb {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 10.0,
            max_y: 10.0,
        },
    };

    let primitives = vec![
        prim(
            1,
            1,
            "a",
            PrimitiveKind::Line,
            Aabb {
                min_x: 1.0,
                min_y: 1.0,
                max_x: 2.0,
                max_y: 2.0,
            },
            10,
        ),
        prim(
            2,
            1,
            "a",
            PrimitiveKind::Line,
            Aabb {
                min_x: 100.0,
                min_y: 100.0,
                max_x: 101.0,
                max_y: 101.0,
            },
            10,
        ),
    ];

    let ir = build_ir(&primitives, viewport, 1.0, (800, 600), None, &cfg);
    let all_ids: Vec<u64> = ir
        .batches
        .iter()
        .flat_map(|b| b.items.iter().cloned())
        .collect();
    assert_eq!(all_ids, vec![1]);
}

#[test]
fn lod_hysteresis_is_deterministic_and_filters_complexity() {
    let mut cfg = PipelineConfig::default();
    cfg.lod.max_complexity_medium = 100;
    cfg.lod.max_complexity_coarse = 10;

    let viewport = Viewport {
        world: Aabb {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 10.0,
            max_y: 10.0,
        },
    };

    let primitives = vec![
        prim(
            1,
            1,
            "a",
            PrimitiveKind::Polyline,
            Aabb {
                min_x: 1.0,
                min_y: 1.0,
                max_x: 2.0,
                max_y: 2.0,
            },
            50,
        ),
        prim(
            2,
            1,
            "a",
            PrimitiveKind::Polyline,
            Aabb {
                min_x: 1.0,
                min_y: 1.0,
                max_x: 2.0,
                max_y: 2.0,
            },
            500,
        ),
    ];

    let ir_med = build_ir(&primitives, viewport, 0.5, (800, 600), Some(LodMode::Full), &cfg);
    assert!(matches!(ir_med.lod_mode, LodMode::Medium | LodMode::Coarse));

    let ids: Vec<u64> = ir_med
        .batches
        .iter()
        .flat_map(|b| b.items.iter().cloned())
        .collect();
    assert!(ids.contains(&1));
    assert!(!ids.contains(&2));
}
