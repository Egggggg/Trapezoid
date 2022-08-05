use std::{env::current_exe, path::PathBuf};

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
                .arg(
                    arg!(-a --add <tag> ... "Tag to apply, can have multiple occurrences")
                        .action(clap::ArgAction::Append),
                )
                .arg(
                    arg!(-g --glob <glob> ... "The glob to find files from, can have multiple occurrences")
                        .action(clap::ArgAction::Append),
                )
                .arg(
                    arg!(-p --path [path] ... "Base path to start the search from")
                        .action(clap::ArgAction::Append)
                        .value_parser(ValueParser::path_buf())
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

    let mut current_exe_dir = current_exe().unwrap();

    current_exe_dir.pop();

    let mut trapezoid = Trapezoid::new(current_exe_dir, true).unwrap();
    let mut output = AddOutput::new();

    for path in paths {
        let add_output = trapezoid.add_tags(&tags, &globs, path.normalize()?.into_path_buf())?;

        output += add_output;
    }

    println!("{} files matched", output.matched_files);
    println!("{} files tagged", output.tagged_files);

    println!("{} directories matched", output.matched_dirs);
    println!("{} directories tagged", output.tagged_dirs);

    Ok(())
}
