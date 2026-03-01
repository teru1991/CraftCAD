pub fn export_stub(doc: &craftcad_sketch::model::SketchDoc) -> String {
    format!("{}:{}", doc.id, doc.entities.len())
}
