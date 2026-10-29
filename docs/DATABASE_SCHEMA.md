# Database schema and initialization

Verified M5B-1 starting main `afb161b94d257e50515a063cd21436ba884a40de` uses schema **9**, with M5A integrated and PO accepted. The **M5B-1 candidate** adds checksummed forward-only migration **010** (schema **10**); it is not integrated or PO accepted. Migrations 001–009 and Catalog Release 11 remain unchanged. Core SQLite storage remains offline.

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

New canonical ingredients, bilingual names/aliases, hierarchical category/membership/dimension/relationship tables, source/provenance/optional exact observations, release/import audit, customization and collision tables; nullable `ingredients.catalog_id` FK. Full table dictionary/policies: INGREDIENT_CATALOG_SCHEMA.md. No recipe, draft, ingredient ID, quantity, archive/trash or preference rewrite. Existing online snapshot/checksum safeguards remain; validation seed is explicit CLI opt-in. At the M3A checkpoint, clean installs applied 001–004. M2B binaries reject schema 4; no downgrade or automatic reset.

M3B-1 uses existing schema 4 without a new migration. Catalog release version 2 and new immutable source-extraction snapshots do not change the SQLite schema version. See INGREDIENT_CATALOG_SCHEMA.md and CATALOG_IMPORT_PIPELINE.md for upgrade/preservation behavior.

## Migration 005 — ingredient personalization

Version 5 is additive and checksummed. `catalog_ingredient_favorites` references canonical IDs; `personal_ingredient_favorites` references independent personal IDs. Each has a primary key that prevents duplicate favorites. `ingredient_personal_aliases` stores one user-owned alias and normalized Turkish search key per canonical ID, with an index on the key. An index on `recipe_ingredients(ingredient_id,recipe_id)` accelerates recent-use lookup; no synthetic usage rows are created. All three new tables use foreign keys and STRICT typing. Ingredient presentation name/notes/unit still use the established `ingredients` row and `ingredient_customizations` ownership markers, protected by an expected revision. Recipe quantity/unit text and every catalog source table remain unchanged. A schema-4 database upgrades in place after the existing online migration snapshot; repeat opening applies no additional SQL. Older schema-4 executables cannot open version 5 without restoring a compatible backup.

## Migration 006 — recipe search

Version 6 adds `recipe_search_docs`, the regular-content `recipe_fts` FTS5 table, three synchronization triggers, `recipe_search_state` and scope/date indexes. Docs are rebuildable derived data keyed by existing recipe UUIDs; their FK cascades only when a recipe is deliberately purged. Title, description/notes/step text, linked personal/catalog Turkish and English ingredient names and aliases are normalized before indexing. Rust updates the document inside the existing recipe transaction and refreshes affected recipes on linked ingredient/catalog changes. The triggers keep FTS rows in sync with document changes; a one-time transactional backfill after migration indexes all historical recipes. Archive/trash status is filtered from authoritative `recipes`, so status changes need no document rewrite. Search only reads bounded pages, and typo suggestions inspect at most 200 scoped candidate titles. No exact recipe measurements, drafts or catalog facts are copied into the search index. A failed write rolls back both aggregate and index changes.

The pre-existing migration runner snapshots schema-5 databases before applying 006. Previously applied SQL files 001–005 and catalog releases are unchanged. Older executables reject schema 6; restoring an earlier code revision requires a compatible pre-upgrade SQLite backup and can lose newer edits. `rebuild_recipe_search` is a transactional repair operation for derived documents, not a normal-startup full rebuild.

## Migration 007 — recipe organization (M4B-1 candidate)

Forward-only, SHA-256-registered `007_recipe_organization.sql` adds `recipe_organization` with one row per recipe: independent positive revision, Boolean favorite and To Try, nullable CHECK-constrained whole-star rating 1–5. Existing recipe rows are backfilled once; an insert trigger creates state for future recipes and duplicates. Organization writes use optimistic revision checks without changing the recipe content/draft revision or `updated_at`. Repeated identical writes are no-ops. Trashed recipes retain state but cannot be modified until restored.

`recipe_collections` stores UUID, Turkish-normalized unique title key, bounded title and revision. `recipe_collection_members` has a composite unique key and FKs to both collection and recipe, with cascade only for explicit collection deletion or recipe purge. Membership updates use the recipe organization revision; deleting a collection increments affected organization revisions before its membership rows cascade, so stale windows cannot silently apply outdated membership state. Active collection counts exclude archived and trashed recipes; archive/trash preserve memberships, and restore reveals them. Recipe search joins organization by its one-to-one key and uses parameterized favorite/To Try/collection predicates without duplicating search results. Existing recipe quantities remain decimal TEXT, and migrations 001–006/catalog releases are unchanged. An older schema-6 binary rejects schema 7; returning to it requires a compatible pre-upgrade backup and may lose later edits.

## Migration 008 — private notes and named filters (M4B-2 candidate)

`008_recipe_notes_filters.sql` adds two independent, STRICT-compatible tables through the existing SHA-256 ledger and pre-migration snapshot runner. `recipe_private_notes` has a recipe FK with `ON DELETE CASCADE`, format version 1, bounded Markdown text, independent positive revision and update timestamp. A recipe without a row has an empty note at revision 0. Saving uses an immediate transaction and expected revision; clearing retains the row and increments its revision. Existing `recipes.notes` and editor-draft payloads are not transformed or reinterpreted. Private Markdown is absent from FTS and is never rendered as HTML. Archive/trash retain it, restore reveals it, duplicate copies it into an independent row, and deliberate permanent recipe purge cascades it. These notes are private to the local profile, **not encrypted**; OS/user access and eventual backup/export policy remain separate concerns.

`saved_recipe_filters` stores a UUID, Turkish-NFC normalized unique title key, positive independent revision, format version 1 and JSON representation of the validated recipe search query. Only structured criteria are stored, with `offset=0` and `limit=24`; arbitrary SQL is never accepted or stored. Unsupported versions and missing ingredient/collection references produce recoverable errors on apply, never a broader query. Names and query updates use optimistic revision checks. Deleting a named filter cannot alter recipes or collections. Active dashboard aggregates and bounded recent rows are read from authoritative `recipes`/`recipe_organization`/`recipe_collections`, excluding archived and trashed recipes. Migrations 001–007 and Catalog Release 11 remain byte-for-byte unchanged. A schema-7 binary rejects schema 8; return requires a compatible pre-upgrade backup and may lose newer edits.


## M5A structured steps — schema 9 candidate

Migration `009_structured_recipe_steps.sql` adds nullable metadata columns to `recipe_steps`: `details_version` (only 1), title (200 characters), duration_minutes (INTEGER 0–10080, NULL means unknown and 0 is explicit), temperature_value (signed whole-number TEXT, −999–9999), temperature_unit (`C`/`F` paired with a value), heat_level (200), equipment/technique (1000 each), and tips (5000). Temperature text is retained exactly; no conversion or physical/culinary interpretation is inferred. Duration travels as digit text through the draft DTO to retain incomplete edits, validates on explicit save, then stores whole minutes and reads as canonical digit text. Recipe ingredient quantities still use exact decimal TEXT such as `1.250`.

`RecipeStep.details` is optional version-1 typed metadata; legacy `{id,instructions}` decodes with no metadata and no links. Instructions remain the single authoritative basic/detailed instruction field. Expanded UI state is transient, not a data switch. Unsupported metadata versions or malformed fields cannot commit; failed saves leave committed rows and durable drafts unchanged.

`recipe_step_ingredients(recipe_id,step_id,line_id)` has a unique step/line pair and **composite foreign keys** to `(recipe_id,id)` in both `recipe_steps` and `recipe_ingredients`. A cross-recipe link fails in SQLite as well as Rust. Removing a line or step cascades its owned links only; the editor removes affected selections with a notice. Archive/trash retain metadata and links, Restore preserves them, and explicit recipe purge cascades them without deleting shared ingredients. A duplicate or explicit save-as-new remaps recipe, step and ingredient-line IDs together. No per-step quantity allocation, deduction or conversion is implied.

The existing aggregate transaction, content revision checks, draft sessions/tombstones, migration snapshots and checksum validation remain authoritative. Upgrade/restart tests use disposable schema-8 databases and verify legacy drafts, recipe IDs/order, M4B organization, catalog Release 11 and decimal text. An old schema-8 binary must not open a schema-9 profile; reverting code does not reverse this migration. Restore a compatible pre-upgrade backup separately, accepting loss of later edits.

## M5B-1 committed history — schema 10 candidate

`010_recipe_versions.sql` adds `recipe_versions`, keyed by stable recipe ID and independent positive `version_number`. Fields include format 1, capture timestamp, source recipe revision, provenance (`created`, `edited`, `upgrade_baseline`, reserved future `restored`), optional same-recipe prior-version reference, positive envelope format with current Rust support for format 1, canonical content fingerprint, full payload SHA-256 and bounded JSON snapshot (128 MiB maximum, above the worst-case current validated aggregate). Source revision and content version numbers have different meanings. A baseline records the current persisted aggregate, original recipe creation timestamp and actual capture time; it does not invent earlier edits.

The typed format-1 envelope stores core content including recipe `notes`, ordered lines with IDs/ingredient IDs/exact quantity TEXT/units/notes, captured ingredient display labels, ordered steps with all M5A details and same-recipe line associations. It excludes lifecycle flags, optimistic revisions, private Markdown notes, organization and saved filters. No media paths or file lifecycle exist; later stable asset IDs require a new format/projection, leaving old bytes untouched.

Comparison projects explicit V1 fields with recursively sorted JSON keys; ordered arrays and exact strings/NULLs matter. Association IDs are a sorted set, and an otherwise empty details-version envelope equals absent metadata. Persisted trimming and integer-duration normalization occur before capture. Changing labels/timestamps/lifecycle alone does not append history. A pinned independent golden checksum guards the projection. The full payload checksum separately protects captured labels and metadata. Existing no-op Save still uses the established recipe revision increment and draft closure, but adds no content snapshot.

A SQL insert trigger enforces contiguous per-recipe numbers and derives `recipe_version_ingredients` directly from payload lines. Its ingredient and unit FKs RESTRICT definition deletion even after a line is removed from the current recipe. UPDATE and per-version DELETE are blocked; `INSERT OR REPLACE` cannot bypass numbering. Only the existing explicit trash/revision-checked recipe purge cascades owned history/reference rows. Active drafts, other recipes and their history still restrict shared ingredient deletion. No automatic pruning or standalone version deletion API exists. Header dispatch recognizes future formats without narrowing them to V1; this older reader refuses both Save and permanent purge with a recoverable `RECIPE_VERSION_UNSUPPORTED`, protecting unknown future references. The reserved restored-from FK is deferred so deliberately purging an entire owned history cannot be blocked by its own internal references; no restore operation is implemented. Internal read functions use metadata keyset pagination (1–100 rows) and single validated/checksummed payload retrieval; no renderer history IPC/UI is added.

The migration runner executes a bounded-memory Rust backfill hook (64 recipe IDs per page, one aggregate at a time) **inside the same IMMEDIATE transaction as migration 010, ledger update and user_version**. The existing pre-upgrade online backup is taken first. All active/archived/trashed recipes receive exactly one `upgrade_baseline`; count/FK/integrity validation precedes commit. Failure or process interruption leaves schema 9 intact and retryable, without a partially available history feature. Normal reopen never repeats backfill. Malformed legacy aggregates stop the upgrade without silent repair; retain the pre-migration backup for explicit investigation. Large libraries consume additional history disk space and hold the migration write lock until completion. Git rollback cannot undo schema 10; older binaries must use a separately restored compatible backup, accepting loss of newer edits.
