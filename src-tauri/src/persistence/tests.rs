use super::*;
use crate::domain::{Preferences, Theme};
use tempfile::tempdir;

#[test]
fn fresh_database_has_real_fts_foreign_keys_and_defaults() {
    let dir = tempdir().unwrap();
    let db = Database::open(dir.path()).unwrap();
    let bootstrap = db.bootstrap().unwrap();
    assert_eq!(bootstrap.storage.schema_version, 3);
    assert!(bootstrap.storage.foreign_keys && bootstrap.storage.fts5);
    assert_eq!(bootstrap.preferences.theme, Theme::System);
    assert_eq!(bootstrap.preferences.locale, "tr");
    assert!(db
        .conn
        .execute(
            "INSERT INTO preferences(id,theme,locale) VALUES(2,'light','tr')",
            []
        )
        .is_err());
    db.conn.execute_batch("CREATE TABLE parent(id INTEGER PRIMARY KEY); CREATE TABLE child(parent_id INTEGER REFERENCES parent(id));").unwrap();
    assert!(db
        .conn
        .execute("INSERT INTO child VALUES (99)", [])
        .is_err());
}
#[test]
fn repeated_initialization_preserves_preferences_and_migration_ledger() {
    let dir = tempdir().unwrap();
    {
        let mut db = Database::open(dir.path()).unwrap();
        db.save_preferences(Preferences {
            theme: Theme::Dark,
            locale: "tr".into(),
        })
        .unwrap();
    }
    for _ in 0..3 {
        let db = Database::open(dir.path()).unwrap();
        assert_eq!(db.preferences().unwrap().theme, Theme::Dark);
        assert_eq!(
            db.conn
                .query_row("SELECT count(*) FROM schema_migrations", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            3
        );
    }
}
#[test]
fn invalid_preference_does_not_modify_saved_data() {
    let dir = tempdir().unwrap();
    let mut db = Database::open(dir.path()).unwrap();
    let err = db
        .save_preferences(Preferences {
            theme: Theme::Dark,
            locale: "en".into(),
        })
        .unwrap_err();
    assert_eq!(err.code, "INVALID_INPUT");
    assert_eq!(db.preferences().unwrap().theme, Theme::System);
    assert!(serde_json::from_str::<Preferences>(r#"{"theme":"purple","locale":"tr"}"#).is_err());
    assert!(serde_json::from_str::<Preferences>(
        r#"{"theme":"dark","locale":"tr","sql":"DROP TABLE preferences"}"#
    )
    .is_err());
}
#[test]
fn modified_checksum_is_rejected_without_mutating_file() {
    let dir = tempdir().unwrap();
    let db = Database::open(dir.path()).unwrap();
    db.conn
        .execute("UPDATE schema_migrations SET checksum=?1", ["0".repeat(64)])
        .unwrap();
    drop(db);
    let path = dir.path().join("library.sqlite3");
    let before = std::fs::read(&path).unwrap();
    assert_eq!(
        Database::open(dir.path()).err().unwrap().code,
        "SCHEMA_INTEGRITY"
    );
    assert_eq!(before, std::fs::read(path).unwrap());
}
#[test]
fn newer_schema_is_rejected_without_mutation() {
    let dir = tempdir().unwrap();
    let db = Database::open(dir.path()).unwrap();
    db.conn.pragma_update(None, "user_version", 4).unwrap();
    drop(db);
    let path = dir.path().join("library.sqlite3");
    let before = std::fs::read(&path).unwrap();
    assert_eq!(
        Database::open(dir.path()).err().unwrap().code,
        "SCHEMA_TOO_NEW"
    );
    assert_eq!(before, std::fs::read(path).unwrap());
}
#[test]
fn missing_ledger_and_corrupt_files_are_not_replaced() {
    for corrupt in [false, true] {
        let dir = tempdir().unwrap();
        let path = dir.path().join("library.sqlite3");
        if corrupt {
            std::fs::write(&path, b"not a database").unwrap();
        } else {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch("CREATE TABLE personal_notes(note TEXT); INSERT INTO personal_notes VALUES('kişisel');").unwrap();
        }
        let before = std::fs::read(&path).unwrap();
        assert!(Database::open(dir.path()).is_err());
        assert_eq!(before, std::fs::read(path).unwrap());
    }
}
#[test]
fn migration_failure_rolls_back_schema_and_ledger() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    let mut initial = Connection::open(&path).unwrap();
    migrations::apply(&mut initial, &path, &migrations::MIGRATIONS[..1]).unwrap();
    drop(initial);
    let path = dir.path().join("library.sqlite3");
    let mut conn = Connection::open(&path).unwrap();
    let changes = [
        migrations::Migration {
            version: 1,
            name: "foundation",
            sql: migrations::MIGRATIONS[0].sql,
        },
        migrations::Migration {
            version: 2,
            name: "failure",
            sql: "CREATE TABLE must_rollback(id INTEGER); INVALID SQL;",
        },
    ];
    assert!(migrations::apply(&mut conn, &path, &changes).is_err());
    assert_eq!(
        migrations::validate(&conn, migrations::MIGRATIONS).unwrap(),
        1
    );
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='must_rollback'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    let backups: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("pre-migration-")
        })
        .collect();
    assert_eq!(backups.len(), 1);
    let backup = Connection::open(backups[0].path()).unwrap();
    assert_eq!(
        migrations::validate(&backup, migrations::MIGRATIONS).unwrap(),
        1
    );
}
#[test]
fn read_only_database_returns_error_without_reset() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    let mut initial = Connection::open(&path).unwrap();
    migrations::apply(&mut initial, &path, &migrations::MIGRATIONS[..1]).unwrap();
    drop(initial);
    let path = dir.path().join("library.sqlite3");
    let mut conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let changes = [
        migrations::Migration {
            version: 1,
            name: "foundation",
            sql: migrations::MIGRATIONS[0].sql,
        },
        migrations::Migration {
            version: 2,
            name: "next",
            sql: "CREATE TABLE next_table(id INTEGER);",
        },
    ];
    assert!(migrations::apply(&mut conn, &path, &changes).is_err());
    assert_eq!(
        migrations::validate(&conn, migrations::MIGRATIONS).unwrap(),
        1
    );
}
#[test]
fn invalid_directory_is_reported_and_error_does_not_expose_paths() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("file");
    std::fs::write(&file, "original").unwrap();
    let error = Database::open(&file).err().unwrap();
    assert_eq!(error.code, "STORAGE_UNAVAILABLE");
    assert!(!serde_json::to_string(&error)
        .unwrap()
        .contains(&file.to_string_lossy().to_string()));
    assert_eq!(std::fs::read_to_string(file).unwrap(), "original");
}
#[test]
fn rust_bootstrap_matches_frontend_contract_shape() {
    let dir = tempdir().unwrap();
    let db = Database::open(dir.path()).unwrap();
    let value = serde_json::to_value(db.bootstrap().unwrap()).unwrap();
    assert_eq!(
        value["preferences"],
        serde_json::json!({"theme":"system","locale":"tr"})
    );
    assert_eq!(value["storage"]["schemaVersion"], 3);
    assert_eq!(value["storage"]["foreignKeys"], true);
    assert_eq!(value["storage"]["fts5"], true);
}
#[cfg(unix)]
#[test]
fn new_database_and_directory_are_private_and_symlinks_rejected() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let dir = tempdir().unwrap();
    let app = dir.path().join("app");
    let db = Database::open(&app).unwrap();
    drop(db);
    assert_eq!(app.metadata().unwrap().permissions().mode() & 0o777, 0o700);
    assert_eq!(
        app.join("library.sqlite3")
            .metadata()
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let link = dir.path().join("link");
    symlink(&app, &link).unwrap();
    assert!(Database::open(&link).is_err());
}

#[test]
fn successful_upgrade_preserves_preferences_and_snapshot_then_is_idempotent() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("library.sqlite3");
    let mut conn = Connection::open(&path).unwrap();
    migrations::apply(&mut conn, &path, &migrations::MIGRATIONS[..1]).unwrap();
    let mut db = Database { conn };
    db.save_preferences(Preferences {
        theme: Theme::Dark,
        locale: "tr".into(),
    })
    .unwrap();
    let changes = [
        migrations::Migration {
            version: 1,
            name: "foundation",
            sql: migrations::MIGRATIONS[0].sql,
        },
        migrations::Migration {
            version: 2,
            name: "test_upgrade",
            sql: "CREATE TABLE migration_test(id INTEGER PRIMARY KEY) STRICT;",
        },
    ];
    let path = dir.path().join("library.sqlite3");
    migrations::apply(&mut db.conn, &path, &changes).unwrap();
    migrations::apply(&mut db.conn, &path, &changes).unwrap();
    assert_eq!(migrations::validate(&db.conn, &changes).unwrap(), 2);
    assert_eq!(db.preferences().unwrap().theme, Theme::Dark);
    let snapshots: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("pre-migration-")
        })
        .collect();
    assert_eq!(snapshots.len(), 1);
    let snapshot = Connection::open(snapshots[0].path()).unwrap();
    assert_eq!(
        migrations::validate(&snapshot, migrations::MIGRATIONS).unwrap(),
        1
    );
    assert_eq!(
        snapshot
            .query_row("SELECT theme FROM preferences", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        "dark"
    );
}

#[test]
fn migration_ledger_gap_and_version_disagreement_are_rejected() {
    for sql in [
        "UPDATE schema_migrations SET version=0",
        "PRAGMA user_version=0",
    ] {
        let dir = tempdir().unwrap();
        let db = Database::open(dir.path()).unwrap();
        if sql.contains("version=0") && !sql.starts_with("PRAGMA") {
            db.conn
                .execute_batch(
                    "PRAGMA ignore_check_constraints=ON; UPDATE schema_migrations SET version=0 WHERE version=1;",
                )
                .unwrap();
        } else {
            db.conn.execute_batch(sql).unwrap();
        }
        drop(db);
        assert_eq!(
            Database::open(dir.path()).err().unwrap().code,
            "SCHEMA_INTEGRITY"
        );
    }
}
