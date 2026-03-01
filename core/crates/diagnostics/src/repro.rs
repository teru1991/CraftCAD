use crate::oplog::OpAction;

pub fn build_repro_template(
    dataset_id: Option<&str>,
    seed: Option<u64>,
    actions: &[OpAction],
) -> String {
    let mut out = String::new();
    out.push_str("# Repro template\n");
    out.push_str(&format!("dataset_id: {}\n", dataset_id.unwrap_or("none")));
    out.push_str(&format!(
        "seed: {}\n",
        seed.map(|s| s.to_string()).unwrap_or_else(|| "none".into())
    ));
    out.push_str("actions:\n");
    for a in actions {
        out.push_str(&format!("- {:?}\n", a));
    }
    out
}
