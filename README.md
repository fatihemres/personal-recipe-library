# Tarif Atlası — Personal Food & Beverage Library

Local-first macOS/Windows desktop application using Tauri 2, React, TypeScript and SQLite. **Milestones 1, 2A, 2B, 3A, 3B-1/2/3 and 3C-1/2 are implemented.** Recipes, personal ingredients, ordered steps, library preferences and settings persist in real local SQLite. No demo records or fabricated statistics are seeded.

The current desktop shell is Turkish, with light/dark/system themes saved in SQLite. Settings shows verified database schema/runtime, foreign keys and FTS5 readiness. The library offers food/beverage filters, Turkish title filtering, recipe creation/detail/editing, duplication, archive/unarchive, confirmed soft deletion, trash/restore and confirmed permanent deletion. Other modules are clearly identified as upcoming.

## Development

Install Node/npm, Rust and platform prerequisites. On macOS, Command Line Tools are sufficient for desktop development. Windows requires Microsoft C++ Build Tools/MSVC and WebView2. See [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

```sh
npm ci
npm run tauri -- dev
```

`npm run dev` opens only the browser frontend; it deliberately reports that native storage requires the desktop app. It does not simulate settings persistence.

For a standalone local macOS verification bundle:

```sh
npm run tauri -- build --debug --bundles app --no-sign -- --locked --offline
open src-tauri/target/debug/bundle/macos/personal-recipe-library.app
```

This is an unsigned debug build, not a signed/notarized release. Cached Rust dependencies are required for `--offline`; omit that flag for a first connected dependency fetch. The packaged application itself runs offline without Vite.

## Data safety

Rust resolves the per-user Tauri application-data directory and stores `library.sqlite3`. Initialization never silently deletes or resets an existing DB. Applied migration SQL is immutable and SHA-256 checked. Pending changes to existing databases take an online SQLite snapshot before transactional migration. Do not edit DB/migration files to bypass a startup error. Complete portable backup/restore is planned for M9; migration snapshots are internal safety files.

## Project documentation

- [Master requirements](docs/MASTER_SPEC.md), [architecture](docs/ARCHITECTURE.md), [phases and acceptance criteria](docs/IMPLEMENTATION_PLAN.md)
- [Database schema/init](docs/DATABASE_SCHEMA.md), [data dictionary/contracts](docs/DATA_DICTIONARY.md)
- [Testing and native verification](docs/TESTING.md), [feature checklist](docs/FEATURE_CHECKLIST.md)
- [Agent guidance](AGENTS.md), [progress/handoff](PROJECT_PROGRESS.md), [changelog](CHANGELOG.md)

## Using recipes

Choose **Yeni tarif**, enter a title and a positive serving count, and optionally add description, durations and notes. Search built-in or personal ingredients by Turkish/English name or alias. The selector supports arrow keys, Enter and Escape, plus recent/favorite shortcuts when the search is empty. To create a missing personal ingredient, type its name and use the inline action; equivalent normalized names reuse the existing stable ingredient. Ingredients persist independently, including when the recipe is canceled.

Each ingredient line accepts an optional positive decimal quantity and g, kg, mL, cc, L or adet. Decimal commas are accepted and saved as decimal text with a dot, preserving precision and trailing zeros. Blank quantity means unknown/as needed. Units are preserved; this milestone does not convert quantities. Add preparation steps and use the up/down controls to order them. Save, open from the library, or edit an existing recipe. **Çöp kutusuna taşı** requires confirmation; open **Çöp kutusu** to restore. Permanent deletion is available only in trash, with an explicit irreversible-action confirmation. Shared ingredients remain intact.

Unfinished recipe input is saved separately in SQLite after a 500 ms pause, with a 2-second periodic checkpoint while typing. Native window close and Cmd-Q await the latest draft write; a failed write leaves the app open. On restart, **Kurtarılabilir taslaklar** distinguishes new recipes from edits. Resume or explicitly discard a draft. **Taslağı sakla ve çık** leaves the editor without losing the draft. Cancel offers Save, Discard or Continue Editing. Only explicit Save changes a committed recipe. Abrupt termination can lose input not yet acknowledged by SQLite.

Stale edits never overwrite newer revisions. Review the current record or explicitly choose **Yeni tarif olarak kaydet** to preserve your changes independently. Archive/delete operations retain edit drafts; after permanent deletion an edit draft can only be saved as a new recipe.

**Malzeme Kütüphanesi** browses the 508 built-in canonical ingredients and your personal ingredients together, or filters by origin, favorites, recently used and nested category. Search matches Turkish case-insensitive substrings (Şek/şek/ŞEK/şeker, with İ/i and I/ı as distinct pairs), English names and aliases. Favoriting changes only local preference tables. “Recently used” means ingredients currently present in non-deleted saved recipes, ordered by those recipes' latest edit time; viewing or drafting alone does not count. Select a record to see its known facts and source. You can change a built-in record's local display name, personal search alias, compatible preferred unit and notes, then reset its presentation without changing the canonical catalog. Personal names, notes and units remain editable with stable references. Possible duplicates are shown for review; records are never merged automatically. Unsaved ingredient forms offer Save, Discard or Continue Editing before navigation.

The entire requested product remains V1 scope. Further catalog coverage/review, recipe-wide advanced search/organization, specialized beverage calculations, media, pantry, shopping, planning, portable backup and release installers remain pending.


## M3A catalog foundation

Canonical bilingual ingredients, hierarchy/provenance/override schema and offline checksummed importer are implemented. The original **eight USDA-backed validation ingredients** remain a separate test fixture; they are not the production catalog. Settings distinguishes validation and production counts. See [catalog pipeline](docs/CATALOG_IMPORT_PIPELINE.md), [source licenses](docs/DATA_SOURCES_AND_LICENSES.md), [schema](docs/INGREDIENT_CATALOG_SCHEMA.md) and [actual coverage](docs/CATALOG_COVERAGE_REPORT.md) for reproduction and limits. All applied migrations 001–005 are retained; schema is now 5.


## M3B-1 first production catalog

235 USDA SR Legacy generic identities with project-curated Turkish/English names
and aliases ship embedded and install transactionally offline on first launch.
There are 227 additional identities and eight re-reviewed shared validation IDs;
the validation fixture remains separate and never auto-installs. Existing
personal records, collisions, recipe references and catalog overrides are
preserved. Missing nutrition, density, allergens and ABV remain unknown.

See [coverage plan](docs/CATALOG_COVERAGE_PLAN.md),
[package coverage](catalog/production/coverage.json),
[actual clean import](catalog/production/installation-report.json) and
[CC0/attribution notice](catalog/production/NOTICE.md). The M3B-3 package now
contains 508 canonical identities, 260 bundled aliases and 50 category nodes.
Eighteen terminology items still await human review; remaining gaps are recorded
in the coverage report. Full cross-platform/signed release validation is pending.
Normal local launch: `npm run tauri -- dev`. Use a separate identifier/profile
for tests; do not reset personal databases. M3C-2 added schema migration 005 for
local favorites and personal aliases. Catalog seed version 4 is unchanged.
