use crate::reasons::{WizardReason, WizardReasonCode};
use sha2::{Digest, Sha256};

pub const SSOT_DEFAULT_SEED: u64 = 20260101;

fn sha256_u64(bytes: &[u8]) -> u64 {
    let mut h = Sha256::new();
    h.update(bytes);
    let out = h.finalize();
    let mut b = [0u8; 8];
    b.copy_from_slice(&out[..8]);
    u64::from_be_bytes(b)
}

pub fn derive_seed_from_template_and_inputs(
    template_id: &str,
    template_version: &str,
    inputs_json_canon: &str,
) -> u64 {
    let s = format!("tpl:{template_id}@{template_version}\ninputs:{inputs_json_canon}");
    SSOT_DEFAULT_SEED ^ sha256_u64(s.as_bytes())
}

pub fn choose_seed(explicit: Option<u64>, derived: u64) -> Result<u64, WizardReason> {
    if let Some(s) = explicit {
        if s == 0 {
            return Err(WizardReason::new(
                WizardReasonCode::WizardDeterminismError,
                "seed must be non-zero",
            ));
        }
        return Ok(s);
    }
    Ok(derived)
}
