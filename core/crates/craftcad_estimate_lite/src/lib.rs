use craftcad_ssot::{ManufacturingOutline2dV1, SsotV1};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EstimateLiteV1 {
    pub schema_version: u32, // = 1
    pub units: String,       // "mm2"
    pub items: Vec<EstimateItemV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EstimateItemV1 {
    pub material_id: Uuid,
    pub material_name: String,
    pub thickness_mm: Option<f64>,
    pub parts_count: u32,
    pub total_area_mm2: f64,
    pub total_area_m2: f64,
}

fn round6(v: f64) -> f64 {
    if !v.is_finite() {
        return 0.0;
    }
    (v * 1_000_000.0).round() / 1_000_000.0
}

fn outline_area_mm2(o: &ManufacturingOutline2dV1) -> f64 {
    let w = round6(o.max_x - o.min_x).max(0.0);
    let h = round6(o.max_y - o.min_y).max(0.0);
    round6(w * h)
}

pub fn compute_estimate_lite(ssot: &SsotV1) -> EstimateLiteV1 {
    let mut items = Vec::<EstimateItemV1>::new();

    // Deterministic: materials sorted by UUID, then aggregate.
    let mut mats = ssot.materials.clone();
    mats.sort_by_key(|m| m.material_id);

    for m in &mats {
        let mut parts_count: u32 = 0;
        let mut total_area_mm2: f64 = 0.0;

        for p in ssot.parts.iter().filter(|p| p.material_id == m.material_id) {
            // parts_count includes quantity; guard overflow
            parts_count = parts_count.saturating_add(p.quantity);

            let base_area = p
                .manufacturing_outline_2d
                .as_ref()
                .map(outline_area_mm2)
                .unwrap_or(0.0);

            // multiply by quantity deterministically
            let q = p.quantity as f64;
            total_area_mm2 = round6(total_area_mm2 + round6(base_area * q));
        }

        let total_area_m2 = round6(total_area_mm2 / 1_000_000.0); // (1000mm)^2
        items.push(EstimateItemV1 {
            material_id: m.material_id,
            material_name: m.name.clone(),
            thickness_mm: m.thickness_mm,
            parts_count,
            total_area_mm2,
            total_area_m2,
        });
    }

    EstimateLiteV1 {
        schema_version: 1,
        units: "mm2".to_string(),
        items,
    }
}

pub fn estimate_hash_hex(est: &EstimateLiteV1) -> String {
    let bytes = serde_json::to_vec(est).expect("estimate json serialize must not fail");
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}
