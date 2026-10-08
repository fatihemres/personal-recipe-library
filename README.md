# Tarif Atlası — Personal Food & Beverage Library

Local-first macOS/Windows desktop application using Tauri 2, React, TypeScript and SQLite. **Milestone 1 foundation is implemented.** Recipe management is planned for M2 onward; this build contains no demo recipes or fabricated statistics.

The current desktop shell is Turkish, with light/dark/system themes saved in SQLite. Settings shows verified database schema/runtime, foreign keys and FTS5 readiness. Other modules are clearly identified as upcoming.

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

The entire requested product remains V1 scope. No recipe CRUD, ingredient catalog, beverage calculations, pantry, shopping, planning, backup or release installer is claimed complete yet.
