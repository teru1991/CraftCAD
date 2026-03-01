use craftcad_io::model::InternalModel;

pub fn parse_model(bytes: &[u8]) -> serde_json::Result<InternalModel> {
    serde_json::from_slice(bytes)
}

pub fn encode_model(model: &InternalModel) -> serde_json::Result<Vec<u8>> {
    serde_json::to_vec_pretty(model)
}
