# M4A acceptance and performance record

## Scope and architecture

M4A implements the search-first Recipe Library on the existing React → typed IPC → Rust worker → SQLite path. Migration 006 adds rebuildable FTS5 documents and indexes; prior migrations and catalog Release 11 are unchanged. Search spans title, description, notes, ordered step instructions, and linked ingredient Turkish/English names and aliases. The server validates query length, scope, kind, dates, UUID, sort and a maximum 50-row page; it binds user parameters and quotes normalized FTS terms. Turkish `İ/i` and `I/ı` remain distinct. A zero-result title suggestion checks no more than 200 candidate titles with a maximum edit distance of one or two. FTS itself does not provide fuzzy matching.

The library shows active, archived and trashed scopes; food/beverage, one included ingredient, created/modified date ranges, relevance/created/modified/Turkish alphabetical sorts, card/list views, counts and 24-row pages. Search and result controls support keyboard access. Search/filter state survives opening a recipe and returning during the current mount. Recipe favorite/category filters are absent because recipe favorites/categories are not yet stored; M4B owns those fields, saved filters and collections.

## Local verification

- Rust/SQLite: all **85** full-suite tests passed after the final rollback regression; all five focused search tests passed independently.
- Frontend: TypeScript, ESLint, 57 Vitest tests and production Vite build passed. Two Playwright browser-shell tests passed when run with loopback-port permission. Browser tests do not establish native persistence.
- Catalog audit: production Release 11 remains 854 identities, 269 aliases, 50 categories; static audit and its five Python tests passed.
- Isolated debug workload: installed the real 854-identity offline catalog, saved 500 synthetic recipes in a disposable SQLite profile, then ran 100 24-row paged searches. In the latest focused run, insert time was 231 ms; search latency median 39.551 ms, p95 45.387 ms, max 50.926 ms. An earlier concurrent run measured 240 ms, 40.205 ms, 85.217 ms and 108.027 ms respectively, demonstrating host-load variability. SQLite `integrity_check` was `ok`. These are single-host debug measurements, not guaranteed service-level targets or native UI measurements.
- Isolated production arm64 `.app`: built with temporary identifier `com.recipeatlas.m4a-verification`; LaunchServices started it and a second complete process launch reused its isolated profile. SQLite reported schema 6, catalog release 11, 854 built-in identities, search backfill ready, `integrity_check=ok` and no foreign-key violations. The app was quit after testing. Native window automation could not resolve this bundle although the app was running, so no GUI click, visual search or recipe editing is claimed. PR cross-platform jobs and package artifacts remain pending at this document checkpoint.

## Preservation and limits

Focused SQLite tests cover precise decimal-text quantity retention, recipe archive/trash/restore/purge, stable linked ingredient references, one-time rebuild, migration integrity, catalog English aliases and ingredient rename reindexing. Full M1–M3D regression remains required after final edits. The normal personal application profile and unrelated root workbook were not touched. Out-of-band manual SQL edits to recipe/ingredient tables require `rebuild_recipe_search`; supported application write paths update documents transactionally. Large-scale native GUI responsiveness and Intel/Windows runtime interaction need Product Owner/platform verification. M4B organization features are not part of M4A.

## Product Owner arm64 acceptance

On the downloaded unsigned arm64 PR artifact, use a backup or a disposable app profile. Confirm startup without storage/catalog errors; create two recipes with distinctive Turkish names, English-searchable catalog ingredients and exact decimal quantities; search by title, step, Turkish and English ingredient names and alias; combine kind, ingredient and date filters; try archive and trash scopes; switch card/list and sort; open a later page then return from detail; fully quit/relaunch and confirm the recipes, quantities, drafts and search results remain. This manual native GUI acceptance is separate from CI compilation and packaging.
