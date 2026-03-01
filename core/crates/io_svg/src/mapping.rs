use craftcad_io::model::{InternalModel, Metadata, Units};

pub fn empty_model(seed: u64, eps: f64) -> InternalModel {
    InternalModel {
        units: Units::Mm,
        entities: Vec::new(),
        metadata: Metadata {
            source_format: "svg".to_string(),
            source_units: None,
            determinism_tag: format!("seed={seed};eps={eps}"),
        },
    }
}
