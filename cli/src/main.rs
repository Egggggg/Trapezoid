use std::{env, path::PathBuf};

use anyhow::Result;
use clap::{arg, builder::ValueParser, command, ArgMatches, Command};
use glob::Pattern;
use normpath::PathExt;
use trapezoid::Trapezoid;

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
                .arg(
                    arg!(-g --glob <glob> "The glob to find files from")
                        .required(true)
                        .action(clap::ArgAction::Append),
                )
                .arg(
                    arg!(<base> "Base path to start the search from")
                        .value_parser(ValueParser::path_buf()),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("tag", sub_matches)) => subcommand_tag(sub_matches)?,
        _ => unreachable!("A subcommand is required"),
    }

    Ok(())
}

fn subcommand_tag(matches: &ArgMatches) -> Result<()> {
    let tags = matches
        .get_many::<String>("add")
        .unwrap()
        .map(|s| s.to_string())
        .collect();

    let globs = matches
        .get_many::<String>("glob")
        .unwrap()
        .map(|s| Pattern::new(s.as_str()).unwrap())
        .collect();

    let base = matches.get_one::<PathBuf>("base").unwrap();
    let mut trapezoid = Trapezoid::new("C://Users/mrhum/Projects/TrapezoidData", true).unwrap();

    let add_output = trapezoid.add_tags(tags, globs, base.normalize()?.into_path_buf());

    println!("{} files tagged", add_output?.amount);

    Ok(())
}
