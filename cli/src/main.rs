use std::{env::current_exe, path::PathBuf};

use clap::{arg, builder::ValueParser, command, ArgMatches, Command};
use glob::Pattern;
use normpath::PathExt;
use trapezoid::Trapezoid;

fn main() -> anyhow::Result<()> {
    let matches = command!()
        .author("Bee Clark")
        .version("0.1.0")
        .about("CLI for a file tagging app called Trapezoid")
        .display_name("Trapezoid")
		.name("Trapezoid")
		.bin_name("trapezoid")
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
		.subcommand(
			Command::new("list")
				.about("Lists tagged files")
				.arg(
					arg!(-t --tag <tags> ... "The tag(s) to search by. Multiple tags under one occurrence are ANDed, multiple occurrences are ORed")	
						.value_delimiter(',')
						.action(clap::ArgAction::Append)
				)
				.arg(
					arg!(-f)
				)
		)
        .get_matches();

    match matches.subcommand() {
        Some(("tag", submatches)) => subcommand_tag(submatches)?,
        Some(("list", submatches)) => subcommand_list(submatches)?,
        _ => unreachable!("A subcommand is required"),
    }

    Ok(())
}

fn subcommand_tag(matches: &ArgMatches) -> anyhow::Result<()> {
    let tags: Vec<String> = matches
        .get_many::<String>("add")
        .unwrap()
        .map(|s| s.to_owned())
        .collect();

    let globs: Vec<Pattern> = matches
        .get_many::<String>("glob")
        .unwrap()
        .map(|s| Pattern::new(s).unwrap())
        .collect();

    let paths: anyhow::Result<Vec<PathBuf>> = matches
        .get_many::<PathBuf>("path")
        .unwrap()
        .map(|p| Ok(p.normalize()?.into_path_buf()))
        .collect();

    let mut paths = paths.unwrap();

    let mut current_exe_dir = current_exe().unwrap();

    current_exe_dir.pop();

    let mut trapezoid = Trapezoid::new(current_exe_dir, true).unwrap();

    let output = trapezoid.add_tags(tags, globs, &mut paths)?;

    println!("{} files matched", output.matched_files);
    println!("{} files tagged", output.tagged_files);

    println!("{} directories matched", output.matched_dirs);
    println!("{} directories tagged", output.tagged_dirs);

    Ok(())
}

fn subcommand_list(matches: &ArgMatches) -> anyhow::Result<()> {
    let tags: Vec<Vec<String>> = matches
        .get_many::<Vec<String>>("tag")
        .unwrap()
        .map(|e| e.iter().map(|f| f.to_string()).collect())
        .collect();
}
