#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct IoReport {
    pub format: String,
    pub entities_in: usize,
    pub entities_out: usize,
    pub approx_applied: usize,
    pub unit_guessed: bool,
    pub postprocess_applied: bool,
    pub origin_shifted: bool,
}
