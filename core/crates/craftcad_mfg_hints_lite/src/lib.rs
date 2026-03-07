use craftcad_ssot::{FeatureTypeV1, SsotV1};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManufacturingHintsLiteV1 {
    pub schema_version: u32, // = 1
    pub items: Vec<MfgHintItemV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MfgHintItemV1 {
    pub part_id: Uuid,
    pub feature_id: Uuid,
    pub spec_name: String,
    pub pilot_hole_mm: f64,
    pub countersink: bool,
    pub note_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FastenerBomLiteV1 {
    pub schema_version: u32,
    pub items: Vec<FastenerBomItemV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FastenerBomItemV1 {
    pub part_id: Uuid,
    pub feature_id: Uuid,
    pub spec_name: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FastenerBomWithHintsLiteV1 {
    pub fastener_bom: FastenerBomLiteV1,
    pub mfg_hints: ManufacturingHintsLiteV1,
}

#[derive(Debug, Deserialize)]
struct ScrewParamsHintsViewV1 {
    v: u32,
    spec_name: String,
    #[serde(default)]
    pilot_hole_mm: Option<f64>,
    #[serde(default)]
    countersink: Option<bool>,
    #[serde(default)]
    countersink_depth_mm: Option<f64>,
}

fn round3(v: f64) -> f64 {
    if !v.is_finite() {
        return 0.0;
    }
    (v * 1000.0).round() / 1000.0
}

pub fn compute_mfg_hints_lite(ssot: &SsotV1) -> Result<ManufacturingHintsLiteV1, (String, String)> {
    let mut items: Vec<MfgHintItemV1> = Vec::new();

    for f in &ssot.feature_graph.features {
        if f.feature_type != FeatureTypeV1::ScrewFeature {
            continue;
        }
        if f.targets.len() != 1 {
            return Err((
                "HINTS_UNSUPPORTED_FEATURE".to_string(),
                "ScrewFeature must target exactly one part_id in Step1".to_string(),
            ));
        }
        let part_id = f.targets[0].part_id;

        let params: ScrewParamsHintsViewV1 =
            serde_json::from_value(f.params.clone()).map_err(|e| {
                (
                    "HINTS_INVALID_VALUE".to_string(),
                    format!("failed to parse hint params: {e}"),
                )
            })?;
        if params.v != 1 {
            return Err((
                "HINTS_UNSUPPORTED_FEATURE".to_string(),
                format!("unsupported params version: {}", params.v),
            ));
        }

        let pilot = params
            .pilot_hole_mm
            .filter(|v| v.is_finite() && *v >= 0.0)
            .unwrap_or(3.0);
        let pilot = round3(pilot);

        let countersink = params.countersink.unwrap_or(false);

        let mut note = format!("pilot φ{}mm", pilot);
        if countersink {
            note.push_str(", countersink");
            if let Some(d) = params
                .countersink_depth_mm
                .filter(|v| v.is_finite() && *v >= 0.0)
            {
                note.push_str(&format!(" depth {}mm", round3(d)));
            }
        }

        items.push(MfgHintItemV1 {
            part_id,
            feature_id: f.feature_id,
            spec_name: params.spec_name,
            pilot_hole_mm: pilot,
            countersink,
            note_text: note,
        });
    }

    // Determinism
    items.sort_by_key(|i| (i.part_id, i.feature_id));

    Ok(ManufacturingHintsLiteV1 {
        schema_version: 1,
        items,
    })
}

pub fn compute_fastener_bom_with_hints_lite(
    ssot: &SsotV1,
) -> Result<FastenerBomWithHintsLiteV1, (String, String)> {
    let hints = compute_mfg_hints_lite(ssot)?;
    let items = hints
        .items
        .iter()
        .map(|i| FastenerBomItemV1 {
            part_id: i.part_id,
            feature_id: i.feature_id,
            spec_name: i.spec_name.clone(),
            quantity: 1,
        })
        .collect();

    Ok(FastenerBomWithHintsLiteV1 {
        fastener_bom: FastenerBomLiteV1 {
            schema_version: 1,
            items,
        },
        mfg_hints: hints,
    })
}

pub fn hints_hash_hex(h: &ManufacturingHintsLiteV1) -> String {
    let bytes = serde_json::to_vec(h).expect("hints json serialize must not fail");
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn fastener_bom_hash_hex(b: &FastenerBomLiteV1) -> String {
    let bytes = serde_json::to_vec(b).expect("fastener bom json serialize must not fail");
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
