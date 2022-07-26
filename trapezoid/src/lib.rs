use std::{
    fs::{create_dir_all, File},
    io::{self, BufRead},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use glob::Pattern;
use rusqlite::Connection;
use walkdir::WalkDir;

pub struct Trapezoid {
    pub data_path: PathBuf,
    db_conn: Connection,
    pub ignore_path: PathBuf,
}

#[derive(Debug)]
pub struct AddOutput {
    pub amount: i32,
    pub tags: Vec<String>,
}

impl Trapezoid {
    pub fn new(data_path: &str, create_parents: bool) -> Result<Self> {
        let path = PathBuf::from(&data_path);
        let conn: Connection;
        let ignore_path: PathBuf;

        if path.is_dir() || create_parents {
            if !path.is_dir() {
                create_dir_all(path.clone())?;
            }

            let db_path = path.clone().join("trapezoid.sqlite");
            conn = Connection::open(db_path)?;

            ignore_path = path.join(".tzignore");
        } else {
            return Err(anyhow!("Directory '{}' does not exist", data_path));
        };

        conn.execute(
            "CREATE TABLE IF NOT EXISTS 'tags' (
			'id'	INTEGER NOT NULL UNIQUE,
			'path'	TEXT NOT NULL,
			'tag'	TEXT NOT NULL,
			PRIMARY KEY('id' AUTOINCREMENT)
		)",
            [],
        )?;

        Ok(Self {
            data_path: path,
            db_conn: conn,
            ignore_path,
        })
    }

    pub fn add_tags(
        self: &mut Self,
        tags: Vec<&str>,
        glob: Pattern,
        base: PathBuf,
    ) -> Result<AddOutput> {
        let ignore_file = File::open(&self.ignore_path)?;
        let ignore_lines = io::BufReader::new(ignore_file).lines();
        let mut ignore: Vec<Pattern> = Vec::new();

        for line in ignore_lines {
            ignore.push(Pattern::new(line?.as_str())?);
        }

        let entries = WalkDir::new(base)
            .into_iter()
            .filter_entry(|e| {
                let filename = e.file_name().to_str().unwrap();

                for pattern in &ignore {
                    if pattern.matches(filename) {
                        return false;
                    }
                }

                return true;
            })
            .filter_map(|e| {
                if e.is_ok() && glob.matches(e.as_ref().unwrap().file_name().to_str().unwrap()) {
                    return Some(e.unwrap());
                }

                return None;
            });

        let tx = self.db_conn.transaction()?;
        let mut amount = 0;

        for item in entries {
            amount += 1;

            for tag in &tags {
                tx.execute(
                    "INSERT INTO tags (path, tag) VALUES (?, ?)",
                    [item.path().to_str().unwrap(), tag],
                )?;
            }
        }

        tx.commit()?;

        let mut tags_out: Vec<String> = Vec::new();

        for tag in tags {
            tags_out.push(tag.to_string());
        }

        return Ok(AddOutput {
            amount,
            tags: tags_out,
        });
    }
}
