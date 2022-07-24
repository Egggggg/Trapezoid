# Trapezoid

Trapezoid is a program designed for file tagging automation
There will be rules users can set, such as tagging files that match a glob pattern
The first thing I'm going to work on is a CLI, but eventually I plan to add a GUI app too

## Usage

Since this program is in very very early development, there are no releases yet
If you want to test it at any time, you will need to have Rust installed, and run `cargo run --bin cli` in the base directory

## Contributions

Contributions are greatly appreciated.
If you want to contribute, first either comment on an unassigned issue in this repo, or make a new one for what you want to work on
Once you have an issue to work on, fork this repository and start working on it
You will need Rust installed for this project, since it is written in Rust, and when I add the GUI, you will need Node.js to work on that
After you make a pull request, I will review it, and either merge it or comment what I don't like about it

## Common Features

The desktop app will just wrap the CLI, so they will share most/all major features
From here on, `app` will be used to refer to both the desktop app and the CLI

- Tagging
	The app will be able to tag files and folders, either manually item by item, or by globs that can match any number of items
- .tzignore
	Files can be excluded from being tagged using the .tzignore file in the data directory
	Globs can be added to this file manually, but they can also be added through the app

## CLI

Arguments in \<> are required
Arguments in \[] are optional

### Implemented Commands

nah

### Planned Commands

- **`trapezoid tag <TAG> <GLOB> [BASE_PATH]`** - Tags all files matching the given glob, starting at either `BASE_PATH` or the current working directory
	- `<TAG>`: *`string`* - The tag to add to matching files
  	- `<GLOB>`: *`glob`* - The glob files must match to be tagged
  	- `[BASE_PATH]`: *`path`* - The path the search will be started at, defaults to the current directory

- **`trapezoid ignore <GLOB>`** - Adds a glob to the .tzignore file in the data directory
  	- `<GLOB>`: *`glob`* - The glob to add