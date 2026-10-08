pub mod migrations;
mod preferences;
mod recipes;
use crate::domain::{AppError, Bootstrap, StorageStatus};
use rusqlite::{Connection, OpenFlags};
use std::{path::Path, time::Duration};

pub struct Database {
    pub(super) conn: Connection,
}
impl Database {
    pub fn open(directory: &Path) -> Result<Self, AppError> {
        let path = directory.join("library.sqlite3");
        // Validate existing data read-only before any journal, schema, or preference mutation.
        if path.exists() {
            if std::fs::symlink_metadata(&path)?.file_type().is_symlink() {
                return Err(AppError::storage());
            }
            let reader = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            migrations::validate(&reader, migrations::MIGRATIONS)?;
        }
        if !directory.exists() {
            std::fs::create_dir_all(directory)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700))?;
            }
        }
        if std::fs::symlink_metadata(directory)?
            .file_type()
            .is_symlink()
        {
            return Err(AppError::storage());
        }
        let fresh = !path.exists();
        if fresh {
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            options.open(&path)?;
        }
        let mut conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.pragma_update(None, "foreign_keys", true)?;
        Self::verify_fts(&conn)?;
        migrations::apply(&mut conn, &path, migrations::MIGRATIONS)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        let db = Self { conn };
        db.bootstrap()?;
        Ok(db)
    }
    fn verify_fts(conn: &Connection) -> Result<(), AppError> {
        let probe = (|| -> rusqlite::Result<i64> {
            conn.execute_batch(
                "CREATE VIRTUAL TABLE temp.foundation_fts_probe USING fts5(content);
                INSERT INTO temp.foundation_fts_probe(content) VALUES ('tarif');",
            )?;
            let count = conn.query_row("SELECT count(*) FROM temp.foundation_fts_probe WHERE foundation_fts_probe MATCH 'tarif'", [], |r| r.get(0))?;
            conn.execute_batch("DROP TABLE temp.foundation_fts_probe;")?;
            Ok(count)
        })();
        match probe {
            Ok(1) => Ok(()),
            _ => Err(AppError::new("FTS_UNAVAILABLE", "errors.fts", false)),
        }
    }
    pub fn bootstrap(&self) -> Result<Bootstrap, AppError> {
        Ok(Bootstrap {
            preferences: self.preferences()?,
            storage: StorageStatus {
                schema_version: migrations::validate(&self.conn, migrations::MIGRATIONS)?,
                sqlite_version: rusqlite::version().into(),
                foreign_keys: self
                    .conn
                    .query_row("PRAGMA foreign_keys", [], |r| r.get::<_, i64>(0))?
                    == 1,
                fts5: {
                    Self::verify_fts(&self.conn)?;
                    true
                },
            },
        })
    }
}
#[cfg(test)]
mod tests;

#[cfg(test)]
mod recipe_tests;
