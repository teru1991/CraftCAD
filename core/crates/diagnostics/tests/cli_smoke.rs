use clap::Parser;
use craftcad_diagnostics::cli::{Args, Command};

#[test]
fn cli_parses_zip_preview() {
    let args = Args::parse_from(["craftcad-diagnostics", "zip", "--preview"]);
    match args.cmd {
        Command::Zip { preview, .. } => assert!(preview),
        _ => panic!("expected zip command"),
    }
}
