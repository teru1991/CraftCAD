use craftcad_io::model::{InternalModel, Metadata, Units};

pub fn empty_model(seed: u64, eps: f64, allow_unit_guess: bool) -> InternalModel {
    InternalModel {
        units: Units::Mm,
        entities: Vec::new(),
        texts: Vec::new(),
        metadata: Metadata {
            source_format: "svg".to_string(),
            unit_guess: allow_unit_guess.then_some("mm".to_string()),
            determinism_tag: format!("seed={seed};eps={eps}"),
        },
    }
}
