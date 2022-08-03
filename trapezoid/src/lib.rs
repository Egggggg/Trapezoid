use std::{
    fs::{create_dir_all, File},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use glob::Pattern;
use ignore::gitignore::Gitignore;
use pathdiff::diff_paths;
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
            "CREATE TABLE IF NOT EXISTS 'tags' (
			'id'	INTEGER NOT NULL UNIQUE,
			'path'	TEXT NOT NULL,
			'tag'	TEXT NOT NULL,
			PRIMARY KEY('id' AUTOINCREMENT)
		)",
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
        let (ignore, err) = Gitignore::new(&self.ignore_path);

        if let Some(e) = err {
            return Err(anyhow!(e));
        }

        let entries = WalkDir::new(base.as_ref())
            .into_iter()
            .filter_entry(|e| {
                let matched = ignore.matched(e.path(), e.path().is_dir());

                return !matched.is_ignore();
            })
            .filter_map(|e| {
                if let Ok(current) = e {
                    let matched = ignore.matched(current.path(), current.path().is_dir());

                    if matched.is_ignore() {
                        return None;
                    }

                    let filename = current.file_name().to_str()?;

                    let abs_path = current.path();
                    let rel_path = diff_paths(abs_path, base.as_ref())?;
                    let rel_path_str = rel_path.to_str()?;

                    for glob in globs {
                        if glob.matches(filename) || glob.matches(rel_path_str) {
                            return Some(current);
                        }
                    }
                }

                return None;
            });

        let tx = self.db_conn.transaction()?;
        let mut amount = 0;

        {
            let mut insert = tx.prepare("INSERT INTO tags (path, tag) VALUES (?, ?)")?;

            for item in entries {
                amount += 1;

                for tag in tags {
                    insert.execute([item.path().to_str().unwrap(), tag])?;
                }
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
