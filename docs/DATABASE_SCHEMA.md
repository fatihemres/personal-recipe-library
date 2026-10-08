# Database schema and initialization

M1 implements schema version **1**, not the future recipe schema. SQLite is compiled into the Rust application through rusqlite 0.40.2 (`bundled`, `backup`). The verified local runtime is SQLite 3.53.2. Core runtime access never requires a server, account, API key, or network.

## Location, access and ownership

Tauri resolves `app_data_dir()` for identifier `com.recipeatlas.desktop`; the worker opens `library.sqlite3` there. On macOS this is normally `~/Library/Application Support/com.recipeatlas.desktop/library.sqlite3`; the exact root is resolved by Tauri on each platform. Paths are not returned through IPC. On Windows the corresponding per-user application-data root is used; Windows runtime/ACL behavior remains unverified.

A dedicated Rust thread owns the connection. Async commands submit typed work and await oneshot replies; SQL/filesystem work never runs on the UI thread. Repositories are under `persistence`, application orchestration under `services`, commands are thin adapters. The worker opens lazily on bootstrap and retries opening after an initialization failure. No delete/reset/recreate command exists.

New Unix directories are mode 0700 and newly created DB/snapshot files 0600; existing data is not chmodded. Direct app-root or DB symlinks are rejected. SQLite uses a 5-second busy timeout, foreign keys on each connection, and WAL after migration validation. There is no unrestricted renderer SQL/filesystem API. SQLite data is private local storage, not encrypted storage; full portable backup/restore is M9.

## Implemented tables

```sql
CREATE TABLE schema_migrations (
  version INTEGER PRIMARY KEY CHECK (version > 0),
  name TEXT NOT NULL,
  checksum TEXT NOT NULL CHECK (length(checksum) = 64),
  applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
CREATE TABLE preferences (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  theme TEXT NOT NULL CHECK (theme IN ('light', 'dark', 'system')),
  locale TEXT NOT NULL CHECK (locale = 'tr'),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
```

The single preference row initially uses system theme and Turkish locale. Updates are validated, parameterized, and transactional. Theme saves return the committed row and the renderer applies it after success. Invalid locale or theme cannot enter supported storage through IPC. No recipe/ingredient/FTS index tables are created yet; FTS5 readiness is tested with a temporary virtual table and an actual MATCH query that is discarded afterward.

## Migration protocol

1. Inspect existing DB read-only. Run `quick_check`, validate `user_version`, migration ledger prefix, exact names and SHA-256 of embedded SQL bytes, and foreign-key integrity. Reject unknown/newer schemas, missing/tampered ledger, corruption or version disagreement without resetting data.
2. Only an empty DB with version 0 and no user schema is initialized. Create private app-data directory/file if needed; open read/write, enable foreign keys, verify temporary FTS5 query.
3. Begin an IMMEDIATE transaction and repeat validation to exclude writer races. If no migration is pending, exit without applying SQL or changing the ledger.
4. Before migrating an existing nonempty DB, take a SQLite online snapshot through a separate read-only connection while the write transaction excludes competing writers. Snapshot filename is `pre-migration-<old-version>-<timestamp>.sqlite3`, with exclusive creation. Failed snapshot aborts migration. This also captures committed WAL data. Internal snapshots are not the complete portable backups planned for M9.
5. Execute pending embedded SQL, insert ledger version/name/checksum, and set `PRAGMA user_version` in the same transaction. Validate integrity before commit. Failure rolls back the schema and tracking writes; keep the snapshot.
6. Select WAL, verify preferences and report readiness. Errors are a safe `{code,messageKey,recoverable}` envelope; no raw DB paths or SQL errors reach the UI.

Applied SQL is immutable: never edit `001_foundation.sql` after a DB has used it. Add increasing migrations to the embedded registry and new SQL files. No automatic down-migration. A newer DB must be opened with its compatible app version; future upgrade/restore procedures must document supported schema ranges. Snapshot retention/restore UI is not yet implemented.

## Proposed full schema, not implemented

The entity/relationship table in [ARCHITECTURE.md](ARCHITECTURE.md#proposed-logical-schema) is the full logical model: recipes and immutable versions/relations; ingredients/products/localized aliases/categories; dimensions/units/presets/conversions/nutrients/allergens; recipe ingredients/steps/category/tag/facet joins; beverage extensions; media/reference joins; collections/ratings/history/filters/private notes; lot inventory/movements; shopping provenance; meal plans; drafts; source/seed/import/conflict/override metadata. Those entities are introduced by their owning milestones, with detailed columns and tests before migrations ship. UUID personal identities, deterministic seed identities, explicit unknown metadata, decimal strings, FK/join indexes, RESTRICT on referenced definitions and deliberate purge-only cascades remain design requirements.
