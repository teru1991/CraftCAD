use crate::args::Args;
use anyhow::{anyhow, Context, Result};
use diycad_format::{open_package, OpenOptions};

pub fn run_verify(args: &Args) -> Result<()> {
    let input = args
        .input
        .as_ref()
        .ok_or_else(|| anyhow!("input is required"))?;
    let opt = OpenOptions {
        verify_integrity: true,
        strict_schema: true,
        ..OpenOptions::default()
    };
    let open = open_package(input, opt).with_context(|| "open failed")?;

    if open.read_only {
        return Err(anyhow!(
            "verify failed: open resulted in read_only; see warnings"
        ));
    }
    Ok(())
}
