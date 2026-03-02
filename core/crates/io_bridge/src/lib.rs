#![forbid(unsafe_code)]

mod from_diycad;
mod to_diycad;

use craftcad_io::IoEngine;
use craftcad_io_dxf::DxfIo;
use craftcad_io_json::JsonIo;
use craftcad_io_svg::SvgIo;

pub use from_diycad::{load_diycad_to_internal_model, LoadDiycadOptions};
pub use to_diycad::{save_internal_model_to_diycad, SaveDiycadOptions};

pub fn default_engine() -> IoEngine {
    IoEngine::new()
        .register_importer(Box::new(DxfIo::new()))
        .register_exporter(Box::new(DxfIo::new()))
        .register_importer(Box::new(SvgIo::new()))
        .register_exporter(Box::new(SvgIo::new()))
        .register_importer(Box::new(JsonIo::new()))
        .register_exporter(Box::new(JsonIo::new()))
}
