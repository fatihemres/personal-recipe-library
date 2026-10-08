# Data dictionary

M1 implemented data and contracts. Future domain entities remain proposed in ARCHITECTURE.md; no recipe tables or CRUD are available.

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

## Initial future DTO contracts

`src/shared/contracts/recipe.ts` contains validation/type definitions only, for M2: recipe kind food/beverage, trimmed nonempty title, positive decimal-string yield, ingredient UUID references, nullable decimal-string quantities and unit codes, ordered step UUID/position/description, canonical/display ingredient names, and unit dimensions mass/volume/count/temperature. Numeric canonical strings use a dot; localized entry parsing is a future measurement concern. Null means unknown/as-needed, not zero. These contracts will gain advanced fields as their milestones arrive; they do not enable or simulate recipe features.
