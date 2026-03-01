use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    dataset: Option<String>,
    #[arg(long)]
    write: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if !args.write {
        println!("dry-run only. pass --write to update golden artifacts.");
        return Ok(());
    }

    println!("TODO: regenerate golden outputs for {:?}", args.dataset);
    Ok(())
}
