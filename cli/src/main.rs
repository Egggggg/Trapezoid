use std::{env, path::PathBuf};

use anyhow::Result;
use clap::{arg, builder::ValueParser, command, ArgMatches, Command};
use glob::Pattern;
use normpath::PathExt;
use trapezoid::{AddOutput, Trapezoid};

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
                .arg(arg!(-a --add <tag> "Tags to apply").action(clap::ArgAction::Append))
                .arg(
                    arg!(-g --glob <glob> "The glob to find files from")
                        .action(clap::ArgAction::Append),
                )
                .arg(
                    arg!(-p --path [path] "Base path to start the search from")
                        .value_parser(ValueParser::path_buf())
                        .action(clap::ArgAction::Append)
                        .default_values(&["./"]),
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
    let tags: Vec<String> = matches
        .get_many::<String>("add")
        .unwrap()
        .map(|s| s.to_string())
        .collect();

    let globs: Vec<Pattern> = matches
        .get_many::<String>("glob")
        .unwrap()
        .map(|s| Pattern::new(s.as_str()).unwrap())
        .collect();

    let paths: Vec<PathBuf> = matches
        .get_many::<PathBuf>("path")
        .unwrap()
        .map(|p| p.to_path_buf())
        .collect();

    println!("{:#?}", paths);

    let mut trapezoid = Trapezoid::new("C://Users/mrhum/Projects/TrapezoidData", true).unwrap();

    let mut output = AddOutput {
        amount: 0,
        tags: tags.clone(),
    };

    for path in paths {
        let add_output = trapezoid.add_tags(&tags, &globs, path.normalize()?.into_path_buf())?;

        output.amount += add_output.amount;
    }

    println!("{} files tagged", output.amount);

    Ok(())
}
