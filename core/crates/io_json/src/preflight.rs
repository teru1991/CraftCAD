use craftcad_errors::AppResult;
use craftcad_io::options::ImportOptions;

pub fn run(bytes: &[u8], opts: &ImportOptions) -> AppResult<()> {
    craftcad_io::preflight::check_bytes_len(bytes, opts)
}
