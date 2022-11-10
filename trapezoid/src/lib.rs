use std::{
    fs::{create_dir_all, File},
    io::{BufRead, BufReader},
    ops::{Add, AddAssign},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use glob::{Pattern, PatternError};
use pathdiff::diff_paths;
use rusqlite::{params, Connection};
use walkdir::WalkDir;

pub struct Trapezoid {
    pub data_path: PathBuf,
    db_conn: Connection,
    pub ignore_path: PathBuf,
}

#[derive(Debug, PartialEq)]
pub struct AddOutput {
    pub matched_files: i32,
    pub tagged_files: i32,
    pub matched_dirs: i32,
    pub tagged_dirs: i32,
    pub tags: Vec<String>,
}

fn vec_union(first: &Vec<String>, second: Vec<String>) -> Vec<String> {
    let mut output = first.clone();

    for item in second {
        if !output.contains(&item) {
            output.push(item);
        }
    }

    return output;
}

impl AddOutput {
    pub fn new() -> Self {
        AddOutput {
            matched_files: 0,
            tagged_files: 0,
            matched_dirs: 0,
            tagged_dirs: 0,
            tags: Vec::new(),
        }
    }
}

/// Add `AddOutput`s together with +
impl Add for AddOutput {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            matched_files: self.matched_files + other.matched_files,
            tagged_files: self.tagged_files + other.tagged_files,
            matched_dirs: self.matched_dirs + other.matched_dirs,
            tagged_dirs: self.tagged_dirs + other.tagged_dirs,
            tags: vec_union(&self.tags, other.tags),
        }
    }
}

/// Add `AddOutput`s together with +=
impl AddAssign for AddOutput {
    fn add_assign(&mut self, other: Self) {
        self.matched_files += other.matched_files;
        self.tagged_files += other.tagged_files;
        self.matched_dirs += other.matched_dirs;
        self.tagged_dirs += other.tagged_dirs;
        self.tags = vec_union(&self.tags, other.tags);
    }
}

#[derive(Debug)]
enum ReadIgnoreErr {
    Io(std::io::Error),
    Pattern(PatternError),
}

impl From<std::io::Error> for ReadIgnoreErr {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<PatternError> for ReadIgnoreErr {
    fn from(e: PatternError) -> Self {
        Self::Pattern(e)
    }
}

impl std::fmt::Display for ReadIgnoreErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadIgnoreErr::Io(e) => write!(f, "{}", e),
            ReadIgnoreErr::Pattern(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ReadIgnoreErr {}

fn read_ignore(ignore_path: PathBuf) -> Result<Vec<Pattern>> {
    let file = File::open(ignore_path)?;

    let lines = BufReader::new(file).lines();

    // turn lines into a Vec<String>, or, if any lines are invalid utf-8, return an std::io::Error
    let lines: Result<Vec<String>, _> = lines.into_iter().collect();
    let lines = lines?;

    let patterns: Result<Vec<Pattern>, _> = lines
        .into_iter()
        .map(|s| {
            let s = if !s.starts_with("/") {
                "**/".to_owned() + &s
            } else {
                s
            };

            let s = if s.ends_with("/") { s + "**" } else { s };

            Pattern::new(s.as_str())
        })
        .collect();
    let patterns = patterns?;

    Ok(patterns)
}

impl Trapezoid {
    pub fn new<T: AsRef<Path>>(data_path: T, create_parents: bool) -> Result<Self> {
        let path = data_path.as_ref();
        let conn: Connection;
        let ignore_path: PathBuf;

        if !path.is_dir() && !path.is_file() && create_parents {
            create_dir_all(&path)?;
        }

        if path.is_dir() {
            let db_path = path.clone().join("trapezoid.sqlite");
            conn = Connection::open(db_path)?;

            ignore_path = path.join(".tzignore");

            if !ignore_path.is_file() {
                File::create(&ignore_path)?;
            }
        } else {
            return Err(anyhow!("Directory '{}' does not exist", path.display()));
        };

        conn.execute(
            r#"CREATE TABLE IF NOT EXISTS "tags" (
			"id"	INTEGER NOT NULL UNIQUE,
			"tag"	TEXT UNIQUE,
			PRIMARY KEY("id" AUTOINCREMENT)
		)"#,
            [],
        )?;

        conn.execute(
            r#"CREATE TABLE IF NOT EXISTS "items" (
				"id"	 INTEGER NOT NULL UNIQUE,
				"path"	 TEXT NOT NULL,
				"tag"	 INTEGER,
				PRIMARY KEY("id" AUTOINCREMENT),
				FOREIGN KEY("tag") REFERENCES "tags"("id") ON DELETE CASCADE
			)"#,
            [],
        )?;

        Ok(Self {
            data_path: path.to_path_buf(),
            db_conn: conn,
            ignore_path,
        })
    }

    pub fn add_tags<T: AsRef<Path>>(
        self: &mut Self,
        tags: &Vec<String>,
        globs: &Vec<Pattern>,
        base: T,
    ) -> Result<AddOutput> {
        let ignore = read_ignore(self.ignore_path.clone())?;

        let entries = WalkDir::new(base.as_ref())
            .into_iter()
            .filter_entry(|e| {
                for pattern in ignore.iter() {
                    let path = e.path().to_str();

                    if match path {
                        Some(path) => pattern.matches(path),
                        None => true,
                    } {
                        return false;
                    }
                }

                true
            })
            .filter_map(|e| {
                if let Ok(current) = e {
                    let filename = current.file_name().to_str()?;

                    let abs_path = current.path();
                    let rel_path = diff_paths(abs_path, base.as_ref())?;
                    let rel_path_str = rel_path.to_str()?;

                    for pattern in ignore.iter() {
                        let path = current.path().to_str();

                        if match path {
                            Some(path) => pattern.matches(path),
                            None => true,
                        } {
                            return None;
                        }
                    }

                    for glob in globs {
                        if glob.matches(filename) || glob.matches(rel_path_str) {
                            println!("{:#?}", rel_path_str);
                            return Some(current);
                        }
                    }
                }

                return None;
            });

        let tx = self.db_conn.transaction()?;
        let mut matched_files = 0;
        let mut tagged_files = 0;
        let mut matched_dirs = 0;
        let mut tagged_dirs = 0;

        {
            let mut select_tag = tx.prepare("SELECT id FROM tags WHERE tag = ?")?;
            let mut insert_tag = tx.prepare("INSERT INTO tags (tag) VALUES (?)")?;
            let mut select_item = tx.prepare("SELECT id FROM items WHERE path = ? AND tag = ?")?;
            let mut insert_item = tx.prepare("INSERT INTO items (path, tag) VALUES (?, ?)")?;

            let mut tag_ids: Vec<i64> = Vec::new();

            for tag in tags {
                if !select_tag.exists([tag])? {
                    let tag_id = insert_tag.insert([tag])?;

                    tag_ids.push(tag_id);
                } else {
                    let tag_id = select_tag.query_row([tag], |row| {
                        let id = row.get(0)?;
                        return Ok(id);
                    })?;

                    tag_ids.push(tag_id);
                }
            }

            for item in entries {
                let dir = item.path().is_dir();

                if dir {
                    matched_dirs += 1;
                } else {
                    matched_files += 1;
                }

                let mut added = false;

                let path_opt = item.path().to_str();

                if path_opt.is_none() {
                    continue;
                }

                let path = path_opt.unwrap();

                for tag in &tag_ids {
                    if !select_item.exists(params![path, tag])? {
                        if !added {
                            if dir {
                                tagged_dirs += 1;
                            } else {
                                tagged_files += 1;
                            }

                            added = true;
                        }

                        insert_item.insert(params![path, tag])?;
                    }
                }
            }
        }

        tx.commit()?;

        let mut tags_out: Vec<String> = Vec::new();

        for tag in tags {
            tags_out.push(tag.to_string());
        }

        return Ok(AddOutput {
            matched_files,
            tagged_files,
            matched_dirs,
            tagged_dirs,
            tags: tags_out,
        });
    }
}
