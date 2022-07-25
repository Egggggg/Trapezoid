use std::{path::Path};
use std::fs::File;

use anyhow::{anyhow, Result};
use glob::Pattern;
use rusqlite::Connection;
use walkdir::WalkDir;

pub struct Trapezoid<'a> {
	pub data_path: &'a Path,
    db_conn: Connection,
	ignore_path: &'a Path
}

impl Trapezoid<'static> {
    pub fn new(data_path: &'static Path) -> Result<Self> {
        let path = Path::new(&data_path);
		let mut conn: Connection;
		let mut ignore_path: &'static Path;

        if path.is_dir() {
			let db_path = &path.join("trapezoid.sqlite").as_path();
            conn = Connection::open(db_path)?;
			ignore_path = &path.join(".tzignore").as_path();

			if !ignore_path.is_file() {
				File::create(ignore_path);
			}
        } else {
            return Err(anyhow!(
                "Directory '{}' does not exist",
                data_path.display()
            ));
        };

        conn.execute(
            "CREATE TABLE 'tags' (
			'id'	INTEGER NOT NULL UNIQUE,
			'path'	TEXT NOT NULL,
			'tag'	TEXT NOT NULL,
			PRIMARY KEY('id' AUTOINCREMENT)
		)",
            [],
        );

        Ok(Self {
            data_path: &data_path,
            db_conn: conn,
			ignore_path
        })
    }

    pub fn add_tags(self: &Self, tags: Vec<String>, glob: &Pattern, base: &Path) {
		let ignore = Read(self)
	}
}
