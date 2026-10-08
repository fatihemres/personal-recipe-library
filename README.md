# Tarif Atlası — Personal Food & Beverage Library

Local-first macOS/Windows desktop application using Tauri 2, React, TypeScript and SQLite. **Milestones 1 and 2A are implemented.** Recipes, personal ingredients, ordered steps and preferences persist in real local SQLite. No demo records or fabricated statistics are seeded.

The current desktop shell is Turkish, with light/dark/system themes saved in SQLite. Settings shows verified database schema/runtime, foreign keys and FTS5 readiness. The library offers food/beverage filters, Turkish title filtering, recipe creation/detail/editing, confirmed soft deletion, trash and restore. Other modules are clearly identified as upcoming.

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

Each ingredient line accepts an optional positive decimal quantity and g, kg, mL, cc, L or adet. Decimal commas are accepted and saved as decimal text with a dot, preserving precision and trailing zeros. Blank quantity means unknown/as needed. Units are preserved; this milestone does not convert quantities. Add preparation steps and use the up/down controls to order them. Save, open from the library, or edit an existing recipe. **Çöp kutusuna taşı** requires confirmation; open **Çöp kutusu** to restore. There is no permanent purge action.

Switching to Settings keeps the editor mounted and retains unsaved input. Cancel requires confirmation when modified. Unfinished edits are not yet persisted across application quit/crash; save before quitting. Durable draft recovery is M2B.

The entire requested product remains V1 scope. Duplication/archive/durable drafts (M2B), built-in catalog, advanced search/organization, specialized beverage calculations, media, pantry, shopping, planning, portable backup and release installers remain pending.
