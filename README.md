# Tarif Atlası — Personal Food & Beverage Library

Local-first macOS/Windows desktop application using Tauri 2, React, TypeScript and SQLite. **Milestones 1, 2A, 2B and 3A are implemented.** Recipes, personal ingredients, ordered steps and preferences persist in real local SQLite. No demo records or fabricated statistics are seeded.

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

Choose **Yeni tarif**, enter a title and a positive serving count, and optionally add description, durations and notes. Search a personal ingredient and click its name to add it. To create one, type its name and choose **Kişisel malzeme oluştur / mevcut olanı seç**; equivalent normalized names reuse the existing stable ingredient. Ingredients persist independently, including when the recipe is canceled.

Each ingredient line accepts an optional positive decimal quantity and g, kg, mL, cc, L or adet. Decimal commas are accepted and saved as decimal text with a dot, preserving precision and trailing zeros. Blank quantity means unknown/as needed. Units are preserved; this milestone does not convert quantities. Add preparation steps and use the up/down controls to order them. Save, open from the library, or edit an existing recipe. **Çöp kutusuna taşı** requires confirmation; open **Çöp kutusu** to restore. Permanent deletion is available only in trash, with an explicit irreversible-action confirmation. Shared ingredients remain intact.

Unfinished recipe input is saved separately in SQLite after a 500 ms pause, with a 2-second periodic checkpoint while typing. Native window close and Cmd-Q await the latest draft write; a failed write leaves the app open. On restart, **Kurtarılabilir taslaklar** distinguishes new recipes from edits. Resume or explicitly discard a draft. **Taslağı sakla ve çık** leaves the editor without losing the draft. Cancel offers Save, Discard or Continue Editing. Only explicit Save changes a committed recipe. Abrupt termination can lose input not yet acknowledged by SQLite.

Stale edits never overwrite newer revisions. Review the current record or explicitly choose **Yeni tarif olarak kaydet** to preserve your changes independently. Archive/delete operations retain edit drafts; after permanent deletion an edit draft can only be saved as a new recipe.

The **Malzemeler** screen edits personal ingredient names, notes and preferred units. Renaming preserves all references; referenced ingredients cannot be deleted. Exact normalized duplicates are reused on creation and rejected on rename. Search matches Turkish case-insensitive substrings (Şek/şek/ŞEK/şeker → Şeker, İ/i and I/ı are distinct pairs). Empty catalog, no matching results and database errors have different messages. Missing ingredients can be created inline; no global catalog is seeded.

The entire requested product remains V1 scope. Built-in catalog, advanced search/organization, specialized beverage calculations, media, pantry, shopping, planning, portable backup and release installers remain pending.


## M3A catalog foundation

Canonical bilingual ingredients, hierarchy/provenance/override schema and offline checksummed importer are implemented. Only **eight verified USDA-backed validation ingredients** are prepared; the extensive production catalog is M3B. Validation records never auto-install into personal data. Settings distinguishes real validation/production counts. See [catalog pipeline](docs/CATALOG_IMPORT_PIPELINE.md), [source licenses](docs/DATA_SOURCES_AND_LICENSES.md), [schema](docs/INGREDIENT_CATALOG_SCHEMA.md) and [actual coverage](docs/CATALOG_COVERAGE.md) for reproduction and limits. M1/M2 data/IDs and migrations 001–003 remain preserved; schema is now 4.
