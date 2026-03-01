use craftcad_io::model::InternalModel;
use craftcad_io::options::ExportOptions;

pub fn optimize_for_machine(
    model: &InternalModel,
    _opts: &ExportOptions,
) -> (InternalModel, usize, bool) {
    (model.clone(), 0, false)
}
