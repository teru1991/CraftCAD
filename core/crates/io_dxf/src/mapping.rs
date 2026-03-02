use craftcad_io::model::{InternalModel, Metadata, Units};

pub fn empty_model(seed: u64, eps: f64) -> InternalModel {
    InternalModel {
        units: Units::Mm,
        entities: Vec::new(),
        texts: Vec::new(),
        metadata: Metadata {
            source_format: "dxf".to_string(),
            unit_guess: Some("mm".to_string()),
            determinism_tag: format!("seed={seed};eps={eps}"),
        },
    }
}
