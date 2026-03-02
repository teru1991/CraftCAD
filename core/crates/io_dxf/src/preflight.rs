use craftcad_io::options::ImportOptions;
use craftcad_io::reasons::AppResult;

pub fn run(bytes: &[u8], opts: &ImportOptions) -> AppResult<()> {
    craftcad_io::preflight::preflight_bytes("dxf", bytes, opts)
}
