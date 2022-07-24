use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(
    author = "Bee Clark",
    version = "0.1.0",
    about = "CLI for a file tagging app called Trapezoid"
)]
struct Cli {
    #[clap(subcommand)]
    subcommand: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Tag {},
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let content = std::fs::read_to_string(&args.path)
        .with_context(|| format!("could not read file {:#?}", args.path.as_path()))?;

    for line in content.lines() {
        if line.contains(&args.pattern) {
            println!("{}", line);
        }
    }

    Ok(())
}
