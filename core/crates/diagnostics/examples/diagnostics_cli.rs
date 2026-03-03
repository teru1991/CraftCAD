use clap::Parser;
use craftcad_diagnostics::cli::{run, Args};

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("ERROR: {e:?}");
        std::process::exit(1);
    }
}
