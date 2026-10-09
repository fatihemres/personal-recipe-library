use crate::domain::AppError;
use rusqlite::{Connection, TransactionBehavior};
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "foundation",
        sql: include_str!("../../migrations/001_foundation.sql"),
    },
    Migration {
        version: 2,
        name: "recipes",
        sql: include_str!("../../migrations/002_recipes.sql"),
    },
    Migration {
        version: 3,
        name: "reliability",
        sql: include_str!("../../migrations/003_reliability.sql"),
    },
    Migration {
        version: 4,
        name: "catalog",
        sql: include_str!("../../migrations/004_catalog.sql"),
    },
];
pub fn checksum(sql: &str) -> String {
    format!("{:x}", Sha256::digest(sql.as_bytes()))
}

pub fn validate(conn: &Connection, migrations: &[Migration]) -> Result<i64, AppError> {
    let check: String = conn.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
    if check != "ok" {
        return Err(AppError::integrity());
    }
    let user_version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let max = migrations.last().map_or(0, |m| m.version);
    if user_version > max {
        return Err(AppError::new("SCHEMA_TOO_NEW", "errors.newerSchema", false));
    }
    let tables: i64 = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE name NOT LIKE 'sqlite_%'",
        [],
        |r| r.get(0),
    )?;
    let ledger: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_migrations')", [], |r| r.get(0))?;
    if !ledger {
        return if tables == 0 && user_version == 0 {
            Ok(0)
        } else {
            Err(AppError::integrity())
        };
    }
    let mut stmt =
        conn.prepare("SELECT version, name, checksum FROM schema_migrations ORDER BY version")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })?;
    let mut current = 0;
    for (index, row) in rows.enumerate() {
        let (version, name, hash) = row?;
        if version > max {
            return Err(AppError::new("SCHEMA_TOO_NEW", "errors.newerSchema", false));
        }
        let expected = migrations.get(index).ok_or_else(AppError::integrity)?;
        if version != expected.version || name != expected.name || hash != checksum(expected.sql) {
            return Err(AppError::integrity());
        }
        current = version;
    }
    if current == 0 || current != user_version {
        return Err(AppError::integrity());
    }
    let mut fk = conn.prepare("PRAGMA foreign_key_check")?;
    if fk.query([])?.next()?.is_some() {
        return Err(AppError::integrity());
    }
    Ok(current)
}

pub fn apply(conn: &mut Connection, path: &Path, migrations: &[Migration]) -> Result<(), AppError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let current = validate(&tx, migrations)?;
    if current == migrations.last().map_or(0, |m| m.version) {
        return Ok(());
    }
    // A separate reader snapshots committed data while this transaction excludes other writers.
    if path.exists() && path.metadata()?.len() > 0 {
        let reader = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AppError::storage())?
            .as_nanos();
        let snapshot = path.with_file_name(format!("pre-migration-{current}-{stamp}.sqlite3"));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options.open(&snapshot)?;
        reader.backup(rusqlite::MAIN_DB, &snapshot, None)?;
    }
    for migration in migrations.iter().filter(|m| m.version > current) {
        tx.execute_batch(migration.sql)?;
        tx.execute(
            "INSERT INTO schema_migrations(version, name, checksum) VALUES (?1, ?2, ?3)",
            rusqlite::params![migration.version, migration.name, checksum(migration.sql)],
        )?;
        tx.pragma_update(None, "user_version", migration.version)?;
    }
    validate(&tx, migrations)?;
    tx.commit()?;
    Ok(())
}
