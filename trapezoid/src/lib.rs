use std::path::Path;

use anyhow::{anyhow, Result};
use glob::Pattern;
use rusqlite::Connection;

pub struct Trapezoid<'a> {
    pub data_path: &'a Path,
    conn: Connection,
}

impl Trapezoid<'static> {
    pub fn new(data_path: &'static Path) -> Result<Self> {
        let conn: Connection = if Path::new(&data_path).is_dir() {
            let db_path = Path::new(&data_path).join("trapezoid.sqlite");

            Connection::open(db_path)?;
            conn.execute(
                "CREATE TABLE 'tags' (
				'id'	INTEGER NOT NULL UNIQUE,
				'path'	TEXT NOT NULL,
				'tag'	TEXT NOT NULL,
				PRIMARY KEY('id' AUTOINCREMENT)
			)",
            );
        } else {
            return Err(anyhow!(
                "Directory '{}' does not exist",
                data_path.display()
            ));
        };

        Ok(Self {
            data_path: &data_path,
            conn,
        })
    }

    pub fn add_tags(self: &Self, tags: Vec<String>, glob: &Pattern, base: &Path) {}
}
