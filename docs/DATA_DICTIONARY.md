# Data dictionary

Implemented M1/M2A/M2B/M3A data and contracts. Future entities remain proposed in ARCHITECTURE.md.

| Table / field | Type and meaning | Constraint / policy |
| --- | --- | --- |
| schema_migrations.version | SQLite INTEGER; ordered schema version | PK, positive; embedded registry must match ledger prefix |
| schema_migrations.name | TEXT; immutable migration name | Required; checked against embedded name |
| schema_migrations.checksum | TEXT; lowercase SHA-256 of exact SQL bytes | Required, 64 characters; exact comparison on startup |
| schema_migrations.applied_at | TEXT; UTC ISO-style timestamp | SQLite UTC timestamp default |
| preferences.id | INTEGER; singleton user preference identity | PK, exactly 1 |
| preferences.theme | TEXT; stored choice, not resolved appearance | light / dark / system |
| preferences.locale | TEXT; current UI locale | tr only until another translation is implemented |
| preferences.updated_at | TEXT; last save UTC timestamp | Updated transactionally by Rust |
| PRAGMA user_version | INTEGER; schema compatibility marker | Must equal final applied ledger version; 1 in M1 |

## IPC contract

- `bootstrap` (no arguments) → `{preferences: {theme, locale}, storage: {schemaVersion, sqliteVersion, foreignKeys, fts5}}`. Storage flags must both be true; response runtime validation rejects a malformed or unverified result.
- `save_preferences` → arguments `{preferences: {theme, locale}}`, response committed `{theme, locale}`. Only supported theme enum/locale, no extra preference fields.
- Structured errors: `code`, `messageKey`, `recoverable`; keys resolve in `src/shared/i18n/tr.ts`. Codes: STORAGE_BUSY, STORAGE_UNAVAILABLE, SCHEMA_INTEGRITY, SCHEMA_TOO_NEW, INVALID_INPUT, FTS_UNAVAILABLE. Renderer adds DESKTOP_REQUIRED, INVALID_RESPONSE, UNKNOWN. Tauri deserialization failures are normalized as unknown safe errors.
- `foreignKeys` means actual connection PRAGMA; `fts5` means a successful real temporary FTS query; `sqliteVersion` is bundled runtime version. None is a hardcoded statistic or recipe count.

## Original M1 future DTO outline (superseded by implemented contracts below)

`src/shared/contracts/recipe.ts` contains validation/type definitions only, for M2: recipe kind food/beverage, trimmed nonempty title, positive decimal-string yield, ingredient UUID references, nullable decimal-string quantities and unit codes, ordered step UUID/position/description, canonical/display ingredient names, and unit dimensions mass/volume/count/temperature. Numeric canonical strings use a dot; localized entry parsing is a future measurement concern. Null means unknown/as-needed, not zero. These contracts will gain advanced fields as their milestones arrive; they do not enable or simulate recipe features.

## Implemented M2A contracts

- `RecipeInput`: id (canonical lowercase hyphenated UUID); expectedRevision (null=create, positive integer=update); title (trimmed 1–200 characters); description (nullable, ≤10000); kind food/beverage; servings (positive decimal text); prepMinutes/cookMinutes (nullable integers 0–10080); notes (nullable, ≤20000); ingredients and steps (ordered arrays, ≤500 each).
- `Recipe`: saved input fields without expectedRevision, plus positive revision, createdAt/updatedAt UTC timestamp strings, nullable deletedAt, and ingredientNames keyed by stable ingredient ID for display. Array order is persisted position order. Snapshot names are resolved at read time, not authoritative ingredient identity.
- Ingredient line: id, ingredientId, quantity (nullable decimal text), unitCode (FK; preserves selection), note (nullable, ≤2000). No density conversion or implicit mass/volume equivalence.
- Step: id and instructions (trimmed 1–10000 characters). Position is the ordered array index, generated in Rust.
- Ingredient: id and name (trimmed 1–200 characters). NFC + Turkish-aware lowercase + collapsed whitespace key supports search/exact duplicate reuse. Source/product/localized names are later catalog extensions.
- Unit: code, dimension mass/volume/count, canonicalCode and exact integer factor. No custom unit editor yet.
- IPC: list_recipes(trash,kind), get_recipe(id), save_recipe(input), set_recipe_deleted(id,revision,deleted), search_ingredients(query), create_ingredient(name), list_units. Existing bootstrap/save_preferences unchanged. Frontend output schemas are strict Zod contracts; Rust validates authoritative input and uses the existing structured error envelope. CONFLICT and NOT_FOUND have Turkish messages; SQL constraints map to validation errors without exposing SQL/paths.

## Implemented M2B contracts

- `Recipe` additionally includes nullable `archivedAt`. Active, archived and trash scope are explicit; `revision` changes on lifecycle operations as well as Save.
- `Draft`: id (session UUID), revision (positive CAS token), recipeId (nullable canonical FK), updatedAt, input (raw incomplete `RecipeInput`), ingredientNames (live display names). Raw decimal strings may be incomplete; production validation is applied only at explicit commit. The payload's expectedRevision is the immutable recipe base, separate from the draft revision.
- `DraftWrite`: id, expectedRevision (draft CAS token; null=new session), input. The two expectedRevision values refer to distinct records and must not be interchanged.
- `PersonalIngredient`: id, name, notes (nullable max 2000), preferredUnit (nullable unit code), revision, origin. Edits/deletions apply only to personal origin. Search response: items, total (all personal definitions, not result count), hasMore (more than 50 matches).
- New IPC: list_drafts(), get_draft(id), save_draft(write), discard_draft(id,revision), commit_recipe(input,draftId,draftRevision,asCopy), duplicate_recipe(id,revision), scope_recipes(scope,kind), archive_recipe(id,revision,archived), purge_recipe(id,revision), search_personal_ingredients(query), edit_ingredient(input), delete_ingredient(id,revision). Unit-returning commands serialize to JSON null. M1/M2A commands remain available.
- `finish_exit` is the native exit handshake, invoked only after the registered recipe draft guard succeeds. Renderer never receives direct DB/filesystem access.
- DRAFT_CONFLICT distinguishes a stale/closed/shared draft session; CONFLICT denotes a stale recipe/ingredient/lifecycle revision. INGREDIENT_DUPLICATE reports a rename collision; INGREDIENT_REFERENCED protects saved/draft references. These have localized messages and keep input available. NOT_FOUND remains explicit for missing records. No raw SQL/filesystem paths are exposed.


## Implemented M3A catalog contracts

Catalog table/field semantics and ownership: INGREDIENT_CATALOG_SCHEMA.md. `ingredients.catalog_id` is a nullable canonical FK, not a replacement for personal identity. Optional observations use exact decimal text and sourced basis; absence is unknown. Stable curated UUIDv8 IDs remain independent of upstream identifiers. `catalog_releases` distinguishes validation/production; `catalog_import_runs` records actual outcomes, including possibly interrupted running audits.

- `catalog_status()` → definitions, validationDefinitions, productionDefinitions, pendingCollisions (nonnegative integer database counts).
- `search_available_ingredients(query)` → items with existing ingredient metadata contract, total across all recipe-facing rows, hasMore. TR/EN names and aliases yield one result per operational ID. Personal manager continues using search_personal_ingredients; recipe editor uses available search.
- `link_personal_catalog(id,revision,catalogId)` → null; explicit CAS metadata link, no name/reference merge. Refuses merging an already published built-in row.
- `customize_catalog_ingredient(input)` → ingredient metadata contract with origin catalog; CAS and per-field override markers, no change to canonical source definition.
- `create_personal_category(tr,en,parent)` → new UUID; validated labels/optional parent FK. Full category-management UI remains M3C.
- Import manifest/report are Rust tooling contracts, not renderer filesystem APIs. See CATALOG_IMPORT_PIPELINE.md. CATALOG_* errors have localized safe feedback; immutable source/package versions, invalid artifacts/licenses, downgrade, identity/name conflict or cycle reject the operation. Source snapshots and user data are not silently overwritten.
