use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "diycad-migrate")]
struct Args {
    input: String,
    #[arg(long)]
    output: Option<String>,
    #[arg(long, default_value = "latest")]
    to: String,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    in_place: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.in_place && args.output.is_none() {
        anyhow::bail!("--in-place は破壊的です。明示の --output 併用を推奨します");
    }

    println!("migrate request: {:?}", args);
    println!("TODO: open zip -> validate schemas -> apply step migration -> verify");
    Ok(())
}
