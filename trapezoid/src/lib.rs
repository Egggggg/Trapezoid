use std::{
    fs::{create_dir_all, File},
    ops::{Add, AddAssign},
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use glob::Pattern;
use ignore::WalkBuilder;
use rusqlite::{params, Connection};

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

impl Trapezoid {
    pub fn new<T: AsRef<Path>>(data_path: T, create_parents: bool) -> anyhow::Result<Self> {
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
        tags: Vec<String>,
        globs: Vec<Pattern>,
        paths: &mut Vec<T>,
    ) -> anyhow::Result<AddOutput> {
        if tags.len() == 0 {
            return Err(anyhow!("You need to supply at least one tag"));
        }
        if globs.len() == 0 {
            return Err(anyhow!("You need to supply at least one glob"));
        }
        if paths.len() == 0 {
            return Err(anyhow!("You need to supply at least one path"));
        }

        // can unwrap cause we made sure it had at least 1 up there
        let mut walker = WalkBuilder::new(paths.pop().unwrap());

        for path in paths {
            walker.add(path);
        }

        walker.add_ignore(self.ignore_path.clone());
        walker.add_custom_ignore_filename(".tzignore");
        walker.git_ignore(false);

        let walking = walker.build();

        let entries = walking.filter_map(|item| {
            if let Ok(entry) = item {
                for glob in globs.clone() {
                    let path = entry.path().to_str()?;

                    if glob.matches(path) {
                        return Some(entry);
                    }
                }
            }

            None
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

            for tag in tags.iter() {
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
