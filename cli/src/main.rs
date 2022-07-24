use anyhow::Result;
use clap::{arg, command, Command};

fn main() -> Result<()> {
    let matches = command!()
        .author("Bee Clark")
        .version("0.1.0")
        .about("CLI for a file tagging app called Trapezoid")
        .display_name("Trapezoid")
        .subcommand_required(true)
        .subcommand(
            Command::new("tag")
                .about("Tags files")
                .arg(
                    arg!(-a --add <tag> "Tags to apply")
                        .required(true)
                        .action(clap::ArgAction::Append),
                )
                .arg(arg!(<glob> "The glob to find files from").multiple_values(true)),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("tag", sub_matches)) => {
            let tags = sub_matches
                .get_many::<String>("add")
                .unwrap()
                .map(|v| v.as_str())
                .collect::<Vec<&str>>();
            let paths = sub_matches.get_one::<String>("glob");

            println!("{:#?}\n{:#?}", tags, paths)
        }
        _ => unreachable!("A subcommand is required"),
    }

    Ok(())
}
