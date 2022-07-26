use std::path::Path;

use anyhow::{anyhow, Result};
use glob::Pattern;
use rusqlite::Connection;
use walkdir::WalkDir;

pub mod utils;

pub struct Trapezoid<'a> {
    pub data_path: &'a Path,
    db_conn: Connection,
}

#[derive(Debug)]
pub struct AddOutput {
    pub amount: i32,
    pub tags: Vec<String>,
}

impl Trapezoid<'static> {
    pub fn new(data_path: &'static Path) -> Result<Self> {
        let path = Path::new(&data_path);
        let conn: Connection;

        if path.is_dir() {
            let db_path_buf = path.join("trapezoid.sqlite");
            let db_path = db_path_buf.as_path();
            conn = Connection::open(db_path)?;
        } else {
            return Err(anyhow!(
                "Directory '{}' does not exist",
                data_path.display()
            ));
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
            data_path: &data_path,
            db_conn: conn,
        })
    }

    pub fn add_tags(
        self: &mut Self,
        tags: Vec<&str>,
        glob: Pattern,
        base: &Path,
        ignore: Option<Vec<Pattern>>,
    ) -> Result<AddOutput> {
        let entries = WalkDir::new(base)
            .into_iter()
            .filter_entry(|e| {
                let filename = e.file_name().to_str().unwrap();

                for pattern in ignore.as_ref().unwrap_or(&Vec::new()) {
                    if pattern.matches(filename) {
                        return false;
                    }
                }

                return true;
            })
            .filter_map(|e| {
                if match e {
                    Ok(_) => true,
                    Err(_) => false,
                } && glob.matches(e.as_ref().unwrap().file_name().to_str().unwrap())
                {
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
