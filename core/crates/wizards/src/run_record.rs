use sha2::{Digest, Sha256};

pub fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}

pub fn make_run_id(template_id: &str, inputs_hash: &str, seed: u64) -> String {
    let s = format!("run:{template_id}:{inputs_hash}:{seed}");
    sha256_hex(&s)[..16].to_string()
}
