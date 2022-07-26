use std::path::PathBuf;

use anyhow::Result;
use clap::{arg, builder::ValueParser, command, Command};
use normpath::PathExt;
use trapezoid::{
    utils::{to_path, to_pattern},
    Trapezoid,
};

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
                .arg(arg!(<glob> "The glob to find files from").multiple_values(true))
                .arg(
                    arg!(<base> "Base path to start the search from")
                        .value_parser(ValueParser::path_buf()),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("tag", sub_matches)) => {
            let tags = sub_matches
                .get_many::<String>("add")
                .unwrap()
                .map(|v| v.as_str())
                .collect::<Vec<&str>>();
            let glob = to_pattern(sub_matches.get_one::<String>("glob").unwrap()).unwrap();
            let base = sub_matches.get_one::<PathBuf>("base").unwrap();
            let mut trapezoid =
                Trapezoid::new(to_path("C://Users/mrhum/Projects/TrapezoidData")).unwrap();

            let add_output = trapezoid.add_tags(
                tags,
                glob,
                base.as_path().normalize().unwrap().as_path(),
                None,
            )?;

            println!("{} files tagged", add_output.amount);
        }
        _ => unreachable!("A subcommand is required"),
    }

    Ok(())
}
