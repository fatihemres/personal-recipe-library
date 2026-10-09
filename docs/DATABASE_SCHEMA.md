# Database schema and initialization

Current schema version is **4**. Applied migrations 001–003 remain unchanged; migration 004 adds the M3A catalog foundation. SQLite is compiled into the Rust application through rusqlite 0.40.2 (`bundled`, `backup`). The verified local runtime is SQLite 3.53.2. Core runtime access never requires a server, account, API key, or network.

## Location, access and ownership

Tauri resolves `app_data_dir()` for identifier `com.recipeatlas.desktop`; the worker opens `library.sqlite3` there. On macOS this is normally `~/Library/Application Support/com.recipeatlas.desktop/library.sqlite3`; the exact root is resolved by Tauri on each platform. Paths are not returned through IPC. On Windows the corresponding per-user application-data root is used; Windows runtime/ACL behavior remains unverified.

A dedicated Rust thread owns the connection. Async commands submit typed work and await oneshot replies; SQL/filesystem work never runs on the UI thread. Repositories are under `persistence`, application orchestration under `services`, commands are thin adapters. The worker opens lazily on bootstrap and retries opening after an initialization failure. No delete/reset/recreate command exists.

New Unix directories are mode 0700 and newly created DB/snapshot files 0600; existing data is not chmodded. Direct app-root or DB symlinks are rejected. SQLite uses a 5-second busy timeout, foreign keys on each connection, and WAL after migration validation. There is no unrestricted renderer SQL/filesystem API. SQLite data is private local storage, not encrypted storage; full portable backup/restore is M9.

## M1 tables (retained unchanged; historical feature boundary)

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

Applied SQL is immutable: never edit `001_foundation.sql`, `002_recipes.sql` or `003_reliability.sql` after a DB has used them. Add increasing migrations to the embedded registry and new SQL files. No automatic down-migration. A newer DB must be opened with its compatible app version; future upgrade/restore procedures must document supported schema ranges. Snapshot retention/restore UI is not yet implemented.

## Proposed full schema, not implemented

The entity/relationship table in [ARCHITECTURE.md](ARCHITECTURE.md#proposed-logical-schema) is the full logical model: recipes and immutable versions/relations; ingredients/products/localized aliases/categories; dimensions/units/presets/conversions/nutrients/allergens; recipe ingredients/steps/category/tag/facet joins; beverage extensions; media/reference joins; collections/ratings/history/filters/private notes; lot inventory/movements; shopping provenance; meal plans; drafts; source/seed/import/conflict/override metadata. Those entities are introduced by their owning milestones, with detailed columns and tests before migrations ship. UUID personal identities, deterministic seed identities, explicit unknown metadata, decimal strings, FK/join indexes, RESTRICT on referenced definitions and deliberate purge-only cascades remain design requirements.

## M2A tables — migration 002_recipes.sql (original feature boundary)

Authoritative SQL: `src-tauri/migrations/002_recipes.sql`, embedded as version 2 / `recipes` in the existing SHA-256 registry. Both migrations are applied transactionally on clean installs. Existing M1 databases are validated and snapshotted using the online backup API before 002; preferences are preserved. Once shipped/applied, 002 is also immutable. M1 executables reject this newer schema; no down-migration exists. Use a preserved M1 snapshot with M1 only if accepting loss of later edits.

| Table | Implemented data and relationships |
| --- | --- |
| units | Code primary key; dimension mass/volume/count; canonical-unit FK; exact positive integer factor. Seeds only six real unit definitions: g→g×1, kg→g×1000, mL→mL×1, cc→mL×1, L→mL×1000, adet→adet×1. No ingredients or recipes are seeded. |
| ingredients | UUID primary key, original name, NFC/Turkish normalized search key, origin personal/catalog, UTC creation time. Partial unique index on personal search names permits future source identities without imposing a global catalog merge. Only personal creation is exposed in M2A. |
| recipes | Stable UUID, title, nullable description/notes, food/beverage kind, decimal-text servings, nullable integer preparation/cooking minutes, revision, UTC created/updated/deleted timestamps. Active/trash/kind/time listing index. |
| recipe_ingredients | Stable line UUID, recipe FK, ingredient FK, nullable positive decimal-text quantity, original unit FK, zero-based position, nullable note; unique recipe/position and ingredient reference index. |
| recipe_steps | Stable step UUID, recipe FK, zero-based position, nonempty instructions; unique recipe/position. |

STRICT types and CHECK constraints enforce kinds, positive bounded decimal syntax, string lengths, nonnegative ordering, positive revisions and 0–10080 minute durations. Rust additionally validates canonical UUIDs, individual limits, at most 500 lines/steps, no NULs, and duplicate child IDs. Decimal limits: 12 integer digits and 6 fractional digits; never SQLite REAL or JS Number for measurements. NULL means unknown quantity, not zero. Units and referenced ingredient definitions use RESTRICT. CASCADE only applies to recipe-owned lines/steps if an explicit future purge is introduced; no permanent recipe or ingredient deletion command is exposed now.

Save creates/updates the recipe and replaces its owned line/step rows in one IMMEDIATE transaction, preserving supplied stable child IDs and array order. Invalid references, constraint violations or stale revisions abort the whole aggregate. Updates require the expected revision and an active recipe. Repeated creation with the same ID conflicts rather than duplicating. Reads collect header, child rows and ingredient names in a read transaction. A save is explicit, not a durable autosave draft.

Soft deletion/restore uses a conditional update with expected revision and prior deletion state, increments revision, changes updated_at, and retains children and personal ingredients. Active lists exclude deleted records; trash lists include them. Current list returns all matching recipes; scalable pagination/summary queries and FTS indexing remain later library work. FTS5 readiness remains verified, but no recipe FTS index is claimed in M2A.

Category/localization/product/variation/nutrition/media/beverage entities will attach through FKs and joins in additive migrations. The six unit definitions are a foundation; normalized quantities, contextual conversions and rounding will be introduced by the measurement milestone without rewriting the original entered quantities/unit references.

## M2B — migration 003_reliability.sql

Embedded version 3 / `reliability`, SHA-256 checked by the existing migration ledger. Clean installations apply 001–003; existing M1/M2A databases receive only pending migrations, preceded by an online snapshot. There are no destructive schema rewrites or catalog seeds. Existing quantities, units, IDs, steps and preferences are preserved. Older M2A executables reject schema 3; no downgrade exists. Restoring a pre-upgrade snapshot loses subsequent changes and is not a substitute for the future portable backup UI.

| Entity/change | Data and constraints |
| --- | --- |
| recipes.archived_at | Nullable UTC timestamp; active requires both archived_at/deleted_at NULL. Added kind/state/time index. |
| ingredients.notes | Nullable personal notes, max 2000 characters. |
| ingredients.preferred_unit | Nullable FK to units, RESTRICT. No implicit quantity conversion. |
| ingredients.revision / updated_at | Positive revision default 1; UTC timestamp initialized from existing created_at, updated by metadata changes. |
| editor_drafts | Session UUID PK; recipe_id nullable FK ON DELETE SET NULL; positive CAS revision; format_version=1; active/closed state; bounded validated JSON payload; updated UTC timestamp. CHECK requires payload for active and NULL for closed. Active/time and recipe indexes. |
| draft_ingredients | Unique draft/ingredient join; draft CASCADE, ingredient RESTRICT; reference index. Includes incomplete editor lines, preventing deletion of definitions used only by drafts. |

The payload retains the recipe ID, immutable expected recipe revision, editable header, precise raw quantity strings and ordered child arrays. It accepts blank titles/instructions and incomplete numeric text without relaxing committed recipe constraints. New drafts have NULL recipe_id; edit drafts reference their committed recipe. Purged edit drafts also have NULL recipe_id, but retain their edit base in the payload and are clearly handled as save-as-new, never resurrected silently.

Draft saves use IMMEDIATE transactions, CAS revisions and immutable target/base validation; ingredient-reference replacement is in the same transaction. Save validates committed data, checks active draft/revision and recipe base, writes the aggregate, removes draft references/payload, and closes the session atomically. Failure preserves both the last committed recipe and its recoverable draft. Discard closes only the expected active session. Closed tombstones intentionally survive to reject delayed writes; payloads and ingredient references do not. New sessions always use new UUIDs.

Duplicate reads a revision-checked consistent source within an IMMEDIATE transaction, creates new recipe/line/step UUIDs and reuses canonical ingredient IDs, quantities, units and order. The copy starts active and independent. Archive/unarchive and soft-delete/restore are revision-checked conditional updates; each increments revision. Archived drafts are retained but stale for committing. Purge requires deleted_at and the expected revision; only owned rows cascade and edit drafts become orphaned. Shared definitions never cascade.

Personal ingredient metadata changes are transactional, personal-only and revision checked. Duplicate normalized names return INGREDIENT_DUPLICATE. Deletion is permitted only for an unreferenced personal definition; both saved lines and draft references are checked, with FKs as a final safeguard. Search returns actual personal count and at most 50 normalized substring matches plus hasMore; exact names sort first. No global catalog records are imported.


## Migration 004 — catalog foundation

New canonical ingredients, bilingual names/aliases, hierarchical category/membership/dimension/relationship tables, source/provenance/optional exact observations, release/import audit, customization and collision tables; nullable `ingredients.catalog_id` FK. Full table dictionary/policies: INGREDIENT_CATALOG_SCHEMA.md. No recipe, draft, ingredient ID, quantity, archive/trash or preference rewrite. Existing online snapshot/checksum safeguards remain; validation seed is explicit CLI opt-in. Historical sections above describe the schema at their original milestone. Clean installs now apply 001–004. M2B binaries reject schema 4; no downgrade or automatic reset.

M3B-1 uses existing schema 4 without a new migration. Catalog release version 2 and new immutable source-extraction snapshots do not change the SQLite schema version. See INGREDIENT_CATALOG_SCHEMA.md and CATALOG_IMPORT_PIPELINE.md for upgrade/preservation behavior.
