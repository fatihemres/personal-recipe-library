# Project progress

## Objective and current milestone
Local-first food and beverage recipe management for macOS and Windows with Tauri 2, React, TypeScript, and SQLite.

Current state: M1, M2A, M2B, M3A and M3B-1 implemented and verified. Next task is M3B-2 only when requested. The final M3B-1 handoff below is current; earlier sections/checkpoints are historical.

## Existing project
- `package.json` / `package-lock.json`: npm scripts and locked frontend dependencies; scripts for dev, build, preview, and Tauri.
- `src/`: React TypeScript starter UI, greeting IPC call, CSS, and assets.
- `public/`: starter logo assets.
- `vite.config.ts`, `tsconfig*.json`, `index.html`: Vite integration and strict TypeScript configuration.
- `src-tauri/src/lib.rs`: Tauri setup, opener plugin, and greeting command; `main.rs`: desktop entry point.
- `src-tauri/Cargo.toml` / `Cargo.lock`: Rust dependencies, using Tauri major version 2.
- `src-tauri/tauri.conf.json`: existing window, build, and bundle configuration; identifier `com.recipeatlas.desktop`.
- `src-tauri/capabilities/default.json`: core and opener permissions.
- `src-tauri/icons/` and `gen/schemas/`: packaging assets and generated schemas.
- `README.md`: original starter documentation. `node_modules/` and Rust build artifacts already existed.
- No SQLite integration, migrations, recipe domain modules, feature tests, or CI workflow found. No AGENTS.md or PROJECT_PROGRESS.md existed in the inspected project.
- Git working tree was clean before this task.

## Development environment and validation
Observed locally; these are installed versions, not a recommendation to upgrade.

| Item | Result |
| --- | --- |
| Host | macOS 27.0, Apple Silicon |
| Node / npm | 24.21.0 / 11.19.0 |
| Rust / Cargo | 1.99.0 / 1.99.0 |
| Active Rust toolchain | stable-aarch64-apple-darwin |
| Native tools | Xcode Command Line Tools, clang, and macOS SDK found; full Xcode not installed according to Tauri diagnostic |
| Installed frontend | React 19.3.0, TypeScript 6.0.3, Vite 8.3.3 |
| Installed Tauri | JS API/CLI 2.12.1; Rust Tauri 2.12.1 |
| `npm ls --depth=0` | Passed |
| `npm run build` | Passed; generated ignored `dist/` output |
| `cargo check --locked --offline --manifest-path src-tauri/Cargo.toml` | Passed using cached dependencies |
| `npm run tauri -- info` | Partial diagnostic: sandbox registry lookups failed; an approved network retry also timed out and was stopped. Full diagnostic did not complete. |

The starter compiles on this macOS environment. Desktop launch, native linking/release bundling, signing, and Windows tooling/builds have not been validated. Full Xcode availability should be revisited when testing macOS packaging. SQLite readiness will be validated when its integration is introduced.

## Product specification and planning — 2026-10-08

Completed documentation-only follow-up:
- `docs/MASTER_SPEC.md`: full normalized product baseline R01–R20; all requested functional modules retained as V1, explicit future language/video distinction.
- `docs/ARCHITECTURE.md`: modular boundaries, technology recommendations, logical entities/relations, measurements/search/localization, catalog/licensing strategy, migration/media/backup safety and risk register.
- `docs/IMPLEMENTATION_PLAN.md`: M1–M11 dependencies, detailed acceptance/tests, traceability, documentation inventory and decision gates.
- AGENTS.md now requires consulting those documents before coding/architectural decisions.

The detailed M1–M11 plan supersedes the earlier abbreviated roadmap. No source/configuration/dependency/lockfile changes, executable schemas, source-data imports, application features, CI workflows or releases were created. Current application remains the original greeting starter. The working tree already contained untracked AGENTS.md and PROJECT_PROGRESS.md from the prior task; their contents were preserved/updated. No commit made in this documentation task.

## Planning handoff before implementation — historical M1 scope

Build the Turkish modular shell with accessible light/dark themes, typed/validated IPC, real SQLite app-data bootstrap with checksummed migrations, persisted preferences, FTS5 capability verification, foundational test/lint setup, and initial schema/data dictionary/feature checklist/testing documentation. Native macOS launch and preference persistence are acceptance gates; assess Windows build/native-test feasibility early.

M1 does not implement recipe CRUD or import external catalogs. Those begin in M2/M3. Every later requested feature remains tracked; see the implementation plan for exact acceptance criteria.

## Unresolved decisions and risks

- Extensive catalog coverage threshold and representative Turkish/beverage checklist require review before M3 acceptance. No catalog imported or license clearance asserted.
- FoodOn repository declares CC BY 4.0; Open Food Facts has distinct database/contents/image licenses requiring redistribution review; USDA exact artifact terms and beverage reference rights must be verified.
- Precise unit contexts/densities, Turkish normalization/sorting, ABV uncertainty, personal seed overrides, migrations and WAL-safe backups require real tests.
- Windows tooling/runtime, supported Intel macOS packaging, offline WebView2 installation, signing/notarization credentials and native automation feasibility remain unverified.
- Proposed rusqlite/bundled SQLite, Tailwind/shadcn and validation/test dependencies are recommendations, not installed or compatibility-tested additions.

## Verification for this task

Read existing project instructions/progress/manifests and working-tree status; reviewed primary-source documentation for licenses, SQLite FTS5/backup/migrations, and Tauri platform/signing/native testing. Reviewed requirement traceability and documentation links. Validation passed: all five Markdown files have clean whitespace; all 20 requirement groups map to the implementation plan; all 11 phases and required AGENTS document references exist; `git diff --check` passed (the newly created files were also checked directly). Builds were not rerun because only documentation changed. Previous successful frontend/Rust checks above are historical; no feature test or native release claim is made for this documentation-only task.

## Milestone 1 — Foundation implementation (2026-10-08)

M1 implemented within the existing project; M2 has not started. Earlier inspection/planning entries above are historical, not the present application state.

### Implemented

- Turkish desktop brand/shell, responsive sidebar/header, real Settings screen, keyboard/skip-link/focus support, system/light/dark themes, reduced-motion styles and honest upcoming-module/library states. No fake recipes or statistics.
- External translation resources in `src/shared/i18n`, native Turkish menu labels, renderer error boundary and localized loading/storage/validation/retry states.
- Typed and Zod-validated `bootstrap`/`save_preferences` IPC; Rust independently validates preferences. UI applies theme after a successful real save; startup loads it from SQLite.
- `src-tauri/src/{commands,domain,services,persistence}` with dedicated DB worker, parameterized preference repository and safe error envelope. `src/app`, `src/features/{recipes,settings}`, `src/shared/{api,contracts,i18n,ui}` introduced only for actual M1 behavior. Future recipe DTOs are types/validators, not a mock repository.
- Real `library.sqlite3` in Tauri-resolved per-user app-data; private newly created Unix files/directories; foreign keys, WAL and real temporary FTS5 MATCH verification. Bootstrap diagnostics report actual schema and SQLite runtime, not constants standing in for tests.
- Embedded version 1 STRICT migration for ledger/singleton preferences. SHA-256, name/version/user_version checks, read-only validation of existing data, IMMEDIATE transaction, pre-upgrade online SQLite snapshot, rollback and no reset/delete operations.
- Foundation tooling/tests and required schema/dictionary/checklist/testing/README/changelog docs. Original greeting replaced by tested actual storage IPC. Opener plugin/dependency/capability removed because unused; production CSP scoped to app assets/IPC.

### Dependency and architecture decisions

Added rusqlite 0.40.2 (`bundled`, `backup`), sha2 0.10.9, Tokio sync; tempfile and Tauri test feature are development-only. Added Zod 4.6.5, Tailwind/Vite plugin 4.3.3, Lucide 1.53.0, Radix Slot, CVA, clsx/tailwind-merge; test/lint tooling includes Vitest 5.0.3, Playwright 1.64.0, Testing Library/jsdom, ESLint/typescript-eslint/hooks and Node typings. Versions locked in existing lockfiles; existing React/TypeScript/Vite maintained. The local Button follows the shadcn Radix/CVA pattern; no generated application or wholesale UI framework introduced.

### Verification evidence

- `npm run typecheck`: passed, including frontend and Vite/Vitest/Playwright/test configuration types.
- `npm run lint`: passed with zero warnings.
- `npm test`: 13 tests passed in 4 files (renderer, IPC client contracts, future DTO contracts, error boundary); injected clients are explicitly renderer-only evidence.
- `npm run build`: production frontend passed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo check --locked --offline --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --locked --offline --manifest-path src-tauri/Cargo.toml`: 15 tests passed, with real SQLite, migration rollback/upgrade/snapshot/idempotency, no-mutation rejects, preferences, FTS5/FKs, Unix paths/permissions and registered command dispatch using real worker/storage.
- `cargo clippy --locked --offline --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`: passed.
- `npm run test:ui`: 2 Playwright Chromium tests passed; actual browser with no IPC mocks, desktop-required state, Turkish page, keyboard and narrow sidebar behavior. Not native E2E.
- `npm run tauri -- dev`: native process built and launched on local macOS arm64. Unbundled binary could not be bound by the computer-use UI tool; switched to the .app bundle for observable verification.
- `npm run tauri -- build --debug --bundles app --no-sign -- --locked --offline`: unsigned debug .app built successfully. Native UI opened at `tauri://localhost` with Vite stopped, confirmed schema 1, SQLite 3.53.2, FK and FTS5 readiness; light then dark switching succeeded; full Cmd-Q/relaunch retained selected dark theme. This is actual UI→IPC→SQLite evidence, not browser mocks.

Initial integration failures were corrected (Node typings, rusqlite backup API, test selector typing, one hook lint issue and a Clippy test borrow). Sandbox denied Playwright's local-server bind; approved rerun proceeded. First browser run lacked Chromium; official test browser installed and rerun passed. Native computer-use permission setup was pending temporarily, then completed. Dependency registry DNS failures were retried with approved network access. No verification timeout is mislabeled as a test failure; no command still failing is marked passed.

### Limitations / unresolved problems

No known M1 functional failure remains. Windows/Intel macOS builds, installer behavior, signed/notarized release, complete cross-platform ACL validation, and automated native WebDriver coverage were not performed. Native startup/restart was verified on arm64 macOS only. SQLite is not encrypted. Internal pre-migration snapshots have no restore/retention UI yet; complete portable backups belong to M9. Future catalog coverage/licensing, conversions/ABV and all unimplemented modules remain open as documented.

### Next task and Git handoff

**M2 only:** persistent basic food/beverage recipe workflow, personal ingredient definitions, ordered steps, yield/quantity core, transactional CRUD, duplication/archive/trash recovery and drafts. Implement exact standardized unit/decimal foundation before storing quantities, without density/household guesses. Continue from M1's real DB/IPC; do not introduce external seeds until M3.

The local foundation commit groups M1 with the previously untracked planning documents, preserving their content and the starter history. No push, tag or GitHub release is authorized in this task. The resulting commit hash is reported in the session final response; `git log -1 --oneline` retrieves it without a self-referential hash inside its own commit.

## M2A completed — Persistent Recipe Management, 2026-10-08

### Implemented

- Real recipe create/read/edit/list through typed Tauri commands, existing Rust storage worker and SQLite. Stable canonical UUIDs, food/beverage kind, title/description, decimal-text servings, nullable preparation/cooking minutes/notes, created/updated/deleted timestamps and optimistic revisions.
- Independent persistent personal ingredients: inline creation, Turkish Unicode/NFC search, exact normalized duplicate reuse, stable IDs; definitions survive canceled/deleted recipes. No fake built-in catalog or demo recipes seeded.
- Multiple ingredient lines with reference, optional precise decimal-text quantity, original unit reference (g/kg/mL/cc/L/adet), stable ID, ordering and optional note. Quantity limit 12 integer/6 fractional digits; comma accepted by UI without floating-point parsing; NULL is unknown. No unsupported conversions or density assumptions.
- Ordered preparation steps with add/edit/remove/up/down and stable identity/order after restart.
- Turkish library cards, kind/title filters, clear empty/loading/errors, clean editor/detail, save/cancel, discard confirmation, confirmed soft delete, basic trash/restore. Light/dark/system M1 themes/settings retained. Editor stays mounted across Settings and diagnostic retries; no durable unfinished draft recovery yet.
- Migration 002 (`recipes`) adds units, ingredients, recipes, recipe_ingredients and recipe_steps with STRICT constraints/FKs/indexes. Migration 001 untouched. All aggregate writes transactional; bad references/child collisions rollback and stale revisions conflict. Read transactions collect consistent recipe data.
- Existing migration checksums, pre-upgrade online snapshots, WAL/FK/FTS5, private application-data paths, safe errors and initialization retry retained. No renderer SQL/filesystem capabilities added.

### Changed modules and documents

Backend: Cargo manifest/lockfile (pinned uuid v4 and Unicode normalization), migration 002/registry, recipe domain/repository/tests, service job queue, commands/IPC tests/registration and safe constraint errors. Frontend: recipe contracts/client, LibraryPage, RecipeEditor, RecipeIngredients, RecipeSteps, localized resources, confirmation dialog, shell edit-preservation behavior, styles and renderer tests. No frontend dependencies added.

Documentation: AGENTS, README, CHANGELOG, ARCHITECTURE, DATABASE_SCHEMA, DATA_DICTIONARY, IMPLEMENTATION_PLAN, FEATURE_CHECKLIST, TESTING, new MEASUREMENT_ENGINE, and this progress file. MASTER_SPEC unchanged; all requested V1 features remain tracked. Git was clean before M2A; no pre-existing user changes were overwritten.

### Verification

| Check | Actual result |
| --- | --- |
| npm run typecheck | Passed |
| npm run lint | Passed, no warnings |
| npm test | Passed: 18 tests across 5 files |
| npm run test:ui | Passed: 2 Chromium browser tests; renderer/platform boundary only |
| npm run build | Passed; final production frontend built in native bundle command |
| cargo fmt --manifest-path src-tauri/Cargo.toml -- --check | Passed |
| cargo check --locked --offline --manifest-path src-tauri/Cargo.toml | Passed |
| cargo test --locked --offline --manifest-path src-tauri/Cargo.toml | Passed: 21 tests, M1 regressions included |
| cargo clippy --locked --offline --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings | Passed |
| Tauri debug macOS .app build, locked/offline, no-sign | Passed for isolated verification and normal identifier |
| Actual native UI→IPC→SQLite→quit/relaunch | Passed on macOS arm64; details below |
| Windows, Intel macOS, installers/signing/notarization | Not tested; no compatible host/credentials in this session |

Rust tests use real temporary databases and registered IPC handlers with actual storage worker. They cover CRUD, step replacement/order, decimal precision/validation/SQL constraints, Turkish duplicate/search behavior, rollback, FKs, stale edits, soft delete/restore, reopen persistence and M1→M2 migration/preservation/idempotency. Frontend injected clients establish interaction behavior only, never native persistence.

Native packaged tauri://localhost app with no Vite server: food recipe with Turkish text, new ingredient, 35.000001 cc and note, reordered steps; full Cmd-Q/relaunch retained all fields/order; edit/save, existing ingredient selection, soft delete confirmation/trash/restore, another restart, theme switch and persisted dark selection. Final-code bundle read persisted records, removed a line, edited a step/saved, and created a simple beverage. Test data used identifier `com.recipeatlas.m2a-verification`, separate from the normal library. Final normal `com.recipeatlas.desktop` bundle launched, upgraded the actual M1 DB to schema 2, retained its dark theme, verified real SQLite/FKs/FTS5 and showed an empty recipe library. No verification recipes entered its production database.

The CUA app selector cached bundle identity when the build path was reused with a different test identifier; selecting fresh copied .app paths resolved it. This was a verification-tool launch lookup issue, not a runtime/storage failure. Playwright sandbox EPERM for port binding was resolved by approved retry. Initial one-migration test fixture assumption and Clippy type-complexity warning were corrected; final checks passed. Offline crate cache lacked Unicode normalization, so an approved development-time fetch was used. No unresolved command timeout is reported.

### Remaining work and limitations

- M2A is complete; all of M2 is not. M2B must implement duplicate/archive and durable draft autosave/recovery. Unfinished edits do not survive quit/crash yet; save first. Cancel discards only after confirmation. Native window-close guard behavior is not claimed as durable recovery.
- Ingredient create/search/select is implemented; full edit/organization/reference-safe deletion and catalog/source/product/localized metadata remain later work. Search is normalized substring with 50 ingredient results, not fuzzy/autocomplete ranking.
- Recipe list is unpaginated and title filtering renderer-side. Summary/pagination/FTS indexing/performance belong to subsequent library work. No fabricated dashboard statistics.
- Unit metadata only: no normalized quantity columns, conversions/scaling/density/ABV/presets yet. Implement via additive migrations in the measurement milestone, preserving entered text/unit meaning.
- No permanent purge UI or portable backup/restore. Migration snapshots are internal SQLite safety files; M1 cannot open schema 2. Restore a compatible pre-upgrade snapshot with M1 only if accepting later-data loss.
- No Windows/Intel native verification or trusted signed release. Temporary verification app/data remain outside the repository; normal workspace bundle is the final normal-identifier build.

### Next task: M2B (do not start without a new task)

Implement reversible archive/filtering and recipe duplication with distinct recipe/child IDs; durable drafts and restart/crash recovery without overwriting saved revisions; clear stale-edit/conflict recovery; reference-safe personal ingredient editing; refine keyboard/validation and list query behavior. Add new migrations rather than modifying 001/002. Retain and extend all M1/M2A tests and native restart checks. See IMPLEMENTATION_PLAN.md's explicit M2 split. Do not begin catalog ingestion (M3) in that task.

### Local launch and version control

`npm run tauri -- dev` launches the native development app. Standalone unsigned local bundle: `npm run tauri -- build --debug --bundles app --no-sign -- --locked --offline`, then `open src-tauri/target/debug/bundle/macos/personal-recipe-library.app`. `npm run dev` alone is a browser preview and truthfully cannot access native SQLite. M2A is recorded in a logical local completion commit; use `git log -1` for its full hash. No push/tag/release was performed.

## M2B completed — Recipe Reliability, Draft Recovery & Advanced Management, 2026-10-08

### Implemented and preserved

- Durable SQLite new/edit drafts separate from production recipes, recovering partial quantities/servings and blank steps. 500 ms debounced saving, 2-second periodic checkpoint, accurate pending/saving/acknowledged/failure states, Turkish recovery/resume/discard controls and leave-with-draft action.
- Stable per-editor session IDs, serialized autosave/Save/Discard, immutable recipe base revisions and draft CAS. Explicit Save atomically commits the recipe and closes its draft; tombstones reject delayed autosave resurrection. Failed writes preserve previous committed data and recoverable input. Stale recipe/shared draft sessions never silently overwrite newer work; current-record preview and explicit save-as-new are available.
- Native close/ExitRequested handshake awaits the current editor's flush before allowing exit. Cmd-Q recovery verified. Failure retains the window/input. Abrupt termination recovers acknowledged SQLite data; unacknowledged input within debounce/interval is not guaranteed.
- Recipe duplication with new recipe/line/step UUIDs, unchanged precise quantities/units/order and shared ingredient IDs. Independent editability verified.
- Active/archive/trash views, reversible archive/unarchive, soft deletion/restore and explicit permanent purge confirmation. Restore retains former archive state. Purge affects only recipe-owned rows, preserves shared ingredients and orphaned draft payloads for explicit save-as-new.
- Personal ingredient names, notes and preferred units editable with optimistic revisions; rename preserves references; duplicate rename has a Turkish conflict; deletion of any saved/draft-referenced ingredient is blocked. Existing create reuses normalized duplicates.
- Turkish keyboard/mouse autocomplete with clear empty catalog/no-match/storage-error messages, inline creation, exact-name priority and 50-result refinement notice. Actual saved/reopened Şeker matches Şek/şek/ŞEK/şeker; İ/i and I/ı pairs verified with real SQLite and IPC. Native four Şeker searches and İçme suyu/iç also verified. No actual matching defect reproduced in M2A; ambiguous empty-state UX was improved. No global catalog seeds or fabricated coverage.
- New checksummed migration 003; applied 001/002 untouched. Existing worker/SQLite/FK/WAL/FTS5/private paths/pre-migration snapshots retained. No dependency, lockfile, architecture replacement or capability expansion. M1 settings/themes and M2A workflows retain regression coverage.

### Modules/files changed

Backend: `migrations/003_reliability.sql`; `domain/reliability.rs` and recipe archive DTO; `persistence/reliability.rs`/tests, migration registry and aggregate writer composition; commands/registration/IPC tests; `lib.rs` native exit handshake. Existing M1/M2A tests now expect schema 3 without modifying their historical migration fixtures.

Frontend: `app/nativeExit.ts`, shell navigation/exit feedback; recipe contracts/client/localization; `DraftSession.ts`, `useDurableDraft.ts`, `DraftRecovery.tsx`, `RecipeDetails.tsx`; library/editor/ingredient-line UI; new ingredient search/manager public feature module; confirmation dialog/styles; renderer tests and test-only client fixture.

Documentation: AGENTS, README, CHANGELOG, ARCHITECTURE, DATABASE_SCHEMA, DATA_DICTIONARY, FEATURE_CHECKLIST, IMPLEMENTATION_PLAN, TESTING and this handoff. MASTER_SPEC and measurement scope retained unchanged.

### Verification results

| Check | Actual result |
| --- | --- |
| npm run typecheck / npm run lint | Passed, no warnings |
| npm test | Passed: 28 tests across 8 files |
| npm run test:ui | Passed: 2 Chromium browser boundary/responsive tests |
| npm run build | Passed (also final native bundle's production frontend) |
| cargo fmt --check / cargo check --locked --offline | Passed |
| cargo test --locked --offline | Passed: 31 tests, retained M1/M2A regression coverage |
| cargo clippy --locked --offline --all-targets -- -D warnings | Passed |
| Locked/offline unsigned debug macOS app builds | Passed for isolated and normal identifiers |
| Native packaged UI→IPC→SQLite workflows/restarts | Passed for new/edit recovery, explicit save, four Şeker searches, duplicate independence, archive/unarchive, trash/restore and shared rename |
| Unexpected process termination | Passed with actual SQLite/open-connection subprocess test |
| Development native launch | Compiled and ran on isolated port 1430 after default 1420 was occupied; stopped cleanly |
| Later/final GUI checks | Limited by ScreenCaptureKit -3811; see below |
| Windows/Intel, installers/signing/notarization | Not tested |

Actual schema-2→3 upgrade fixture preserves recipes/IDs/decimal units/order/preferences/checksums and startup idempotency. New registered IPC commands use actual storage, not mock repositories. Frontend clients isolate UI behavior only. Permanent purge/discard/reference safety and transaction rollback are tested against real SQLite; native purge confirmation was opened and canceled. Detailed evidence and exact coverage boundaries are in TESTING.md.

Native test data is isolated under `com.recipeatlas.m2b-verification`; no test recipes were added to the normal production database. The final repository .app bundle uses unchanged `com.recipeatlas.desktop`. Fresh copied app paths avoid the earlier CUA bundle-identity cache issue.

### Remaining issues and verification limits

- No known failing automated check. The native capture service later returned ScreenCaptureKit -3811, including after tool reset. Further native dotless-I UI observation, window-close-button flush, failure feedback, final small mouse-focus/icon/listener-error/localization refinements and normal production-identifier launch were not observed; do not claim those GUI checks passed. Four native Şeker queries, new/edit Cmd-Q restart recovery and actual SQLite İ/i/I/ı tests did pass.
- Default `npm run tauri -- dev` failed because another process occupied port 1420, not because of a timeout. Config-only port-1430 retry launched. The existing process was not interrupted. Further users may need to free their own dev server or use the temporary command below.
- Last unacknowledged input may be lost during force-quit before an autosave is committed. Renderer failure cannot participate in graceful flush; force-quit still recovers previous acknowledged drafts. Closed session tombstones intentionally remain; safe compaction policy is future maintenance work. Schema upgrades must explicitly migrate draft format 1 before changing payload contracts.
- Lists remain unpaginated and recipe title filters remain renderer-side; no fuzzy/FTS recipe search, media/history, advanced editor/measurement features or portable backups implemented. Full requested V1 scope remains tracked.
- No Windows/Intel runtime or trusted signed release verified. Unsigned macOS debug build is an internal verification artifact only.

### Next milestone and local launch

**M3 — Ingredient catalog and source pipeline.** Begin with licensing/source/coverage decisions, schema/provenance/localized alias/override design, then a real normalized versioned idempotent offline seed pipeline with user-edit protection and actual coverage report. No M3 work has begun; stop at M2B in this task.

Normal launch: `npm run tauri -- dev`. If port 1420 is occupied, a nonpersistent isolated verification command is:

```sh
npm run tauri -- dev --config '{"identifier":"com.recipeatlas.m2b-verification","build":{"beforeDevCommand":"npm run dev -- --port 1430","devUrl":"http://localhost:1430"}}' -- --locked --offline
```

This command uses separate verification data, not the personal production library. To use normal personal data on another port, omit its identifier override. Standalone normal app: `npm run tauri -- build --debug --bundles app --no-sign -- --locked --offline`, then `open src-tauri/target/debug/bundle/macos/personal-recipe-library.app`. Build output is ignored.

Git was clean at start (M2A HEAD `52d0629fb326f67723c22a153a0210642a43f917`). M2B is grouped in one verified local commit; use `git log -1` for its full hash. No push/tag/release performed, and no personal databases/test artifacts are committed.


## M3A completion / interrupted-session recovery — 2026-10-09

Resumed the existing uncommitted M3A implementation from M2B commit `df8e34df9d00a1ad04c64ca2a423125643f59685`; did not regenerate the project, discard earlier work, refetch datasets or repeat licensing research. Source review/retrieval dates remain 2026-10-08. Git contained the existing catalog/schema/importer/docs/UI changes; no unrelated changes were removed. Final code review added a transactional guard against new system-name/personal-name collisions during catalog upgrades, a real regression test, and explicit selected-source duplicate detection in preparation tooling. No dependency, lockfile or packaging configuration change.

### Implemented / actual counts

- Migration **004_catalog.sql**, retaining byte-for-byte 001–003. Separate canonical definitions and recipe-facing IDs; bilingual names/aliases, hierarchy/M:M memberships, dimensions/relations, optional exact observations, immutable source/version provenance, release/audit, collision and customization tables. Existing preferences, recipe/draft references, decimal quantities, archive/trash and personal metadata preserved.
- Reviewed source manifest: USDA SR Legacy CC0 factual data and project-authored CC0 curation enabled; Foundation needs artifact pinning; FoodOn conditional pending release/imported-term review; Open Food Facts excluded pending ODbL distribution analysis; TheCocktailDB excluded pending applicable offline redistribution rights. No images reused. Full findings/primary links: DATA_SOURCES_AND_LICENSES.md.
- Real offline CLI importer: checksummed local artifacts/evidence/license allowlist; deterministic identities; prevalidation; transactional catalog writes plus success audit; failure rollback; interruption recovery; unchanged-package no-op; downgrade/content-drift rejection; explicit revision-checked personal linking; per-field catalog overrides; user category creation. No automatic seed or renderer filesystem/network importer.
- **8 validated USDA-backed canonical ingredients, 8 category nodes, 16 provenance mappings, 0 production definitions, 0 optional empirical observations.** Clean import: 8 recipe-facing catalog rows, 0 collisions. Existing personal Şeker fixture/native test: 7 available catalog identities, 1 pending collision, original personal ID/recipe retained. Direct/overlapping category counts in CATALOG_COVERAGE.md and captured coverage.json.
- Typed IPC/client plus real Settings diagnostics; recipe autocomplete can search installed TR/EN names/aliases; personal ingredient manager remains personal-only. Full catalog/discovery/collision/customization UI is M3C, not claimed complete.

### Verification

| Check | Final result |
|---|---|
| npm run typecheck / lint / build | Passed |
| npm test | Passed: 32 tests across 10 files |
| npm run test:ui | Passed: 2 browser tests after approved local-port retry |
| cargo fmt --check / check --locked --offline | Passed |
| cargo test --locked --offline | Passed: 43 tests, including retained M1/M2A/M2B regressions and subprocess helpers |
| cargo clippy --locked --offline --all-targets -- -D warnings | Passed |
| Generator reproducibility / real CLI install and re-import | Passed; all four generated file hashes unchanged; repeat import unchanged=true |
| Native unsigned debug macOS arm64 bundle | Built for isolated and normal identifiers |
| Native packaged UI→IPC→SQLite | Observed recipe preservation after import/restart, actual validation/production/collision diagnostics, English alias selection, catalog-backed draft recovery and persisted dark theme |
| Tauri development launch | Compiled/reached Running target/debug/personal-recipe-library on isolated port 1430; stopped cleanly |
| Windows/Intel / signed installers/notarization | Unverified; no claims |

Detailed test coverage and native observation limits are in TESTING.md. Playwright's first resumed run failed with sandbox EPERM when binding 127.0.0.1:1420, then passed on authorized retry; not a product failure or timeout. Earlier pre-resume tests exposed schema-version fixture expectations and schema-3 fixture use of a new repository path, which were corrected without editing historical migrations. Final checks pass; no unresolved timeout.

### Native evidence and data protection

Used `com.recipeatlas.m3a-verification` and separate copied .app paths. Existing personal Şeker from the interrupted session was retained. Created a real recipe using it with `35.000001 g` and a step; quit, imported eight validation records through the actual CLI, relaunched and confirmed exact data unchanged. Settings showed schema 4/FK/FTS5, validation 8/production 0/pending 1. Native ICING → one Pudra şekeri result selected by keyboard; a separate incomplete catalog-backed draft recovered `12.` after observed input/Cmd-Q/relaunch. Personal manager displayed only Şeker. Dark preference survived restart. No validation recipes or catalog records were added to the normal personal production database. Isolated verification data was left intact, not purged.

Native GUI observations concern the packaged verification app. The final additional importer name-conflict guard is verified by real SQLite test and final build; its rejection dialog was not manually exercised. Development process launch was verified by CLI output; bundle lookup selected the packaged app, so GUI interactions are not misreported as dev-runtime UI proof. Initial same-call type/quit input was not confirmed in AX and did not recover the quantity; repeated test with observed input did recover `12.`. No per-keystroke durability guarantee is added. See existing M2B recovery limits.

### Remaining risks / next task

**M3B only:** define representative coverage matrix and actual acceptance count; pin approved bulk artifacts, verify forms/aliases/Turkish terminology and lawful beverage/regional sources; generate extensive reviewed production package/report/notices; increment dataset version and reviewed allowlist; bundle artifacts and install idempotently offline through the Rust worker; verify clean install and updates/collisions/overrides/recipes/drafts/interruption on real SQLite and native hosts. Detailed sequence is in IMPLEMENTATION_PLAN.md. No M3B/M3C work started.

Pending: FoodOn/OFF/CocktailDB rights/dependency review; realistic catalog search/performance testing; collision-management/reset-to-default UI; catalog retirement policy; schema downgrade/portable backup limitations; Windows/Intel/runtime and signing validation. Checksums are not signatures. All broad food/beverage/Turkish catalog families remain V1 requirements.

Normal local launch: `npm run tauri -- dev`. Isolated verified launch:

```sh
npm run tauri -- dev --config '{"identifier":"com.recipeatlas.m3a-verification","build":{"beforeDevCommand":"npm run dev -- --port 1430","devUrl":"http://localhost:1430"}}' -- --locked --offline
```

Offline validation import reproduction: docs/CATALOG_IMPORT_PIPELINE.md. Final M3A changes are committed locally; retrieve full hash with `git log -1`. No push/tag/release.

## M3B-1 checkpoint — 2026-10-09 (verification pending)

Started from clean M3A commit 4bf1d5539bf782cb144bf9293d0cc312990482bf. Prepared separate production version 2 with 230 selected USDA SR Legacy identities (222 additional, eight re-reviewed shared canonical identities), explicit curated bilingual labels and unchanged source descriptors. Validation files and migrations 001–004 untouched. Generator checks pinned archive SHA and per-ID descriptor. Three empty planned groups remain: regional Turkish, international specialty, cocktail bitters. All empirical metadata unknown. Embedded offline loader/worker installation and multi-snapshot source clearance implemented; automated/native verification and final documentation are pending. No user database reset or production-profile GUI launch. Resume using tools/prepare_production_catalog.py and catalog/production/manifest.json; do not regenerate historical seeds.


## M3B-1 completion / current handoff — 2026-10-09

Completed only M3B-1 from clean M3A commit 4bf1d5539bf782cb144bf9293d0cc312990482bf.
No project recreation, new migration, dependency/lockfile change, database reset,
production-profile GUI launch, remote push, tag or release. M3B-2/3 and M3C remain
unimplemented. All broad food/beverage/Turkish V1 families retain their scope.

### Delivered and actual data

- Separate production package **dataset version 2**, **235 canonical identities**,
  **39 hierarchical category nodes**, **470 production provenance mappings**.
  Primary type counts: 197 food, 38 beverage; 200 aliases; zero selected duplicate-name candidates.
  **227 additional identities**, eight common identities re-reviewed with stable
  validation IDs; the original eight-record validation package is unchanged.
- Reused pinned official USDA SR Legacy April 2018 archive, CC0 factual data.
  Project-authored bilingual labels/aliases/classifications dedicated CC0,
  attributed separately; no images, branded dump or inferred empirical values.
  Immutable source snapshots 2018-04-selection-2 and production-1; old snapshots
  retained. FoodOn conditional; OFF/CocktailDB excluded; Foundation unpinned.
- Deterministic offline preparation script checks archive hash and each exact
  FDC descriptor; curated forms/qualifiers reviewed explicitly, doubtful aliases
  removed. No automatic translations. Accepted TR/EN names complete, zero selected
  duplicate IDs/rejected records; no exhaustive upstream rejection claim.
  Independent Turkish culinary expert review remains outstanding.
- **All 235 nutrition, density, allergen and ABV observation sets unknown**.
  Sources provide 235 USDA and 235 curation mappings, not 470 ingredients.
  Complete category matrix/ambitious conditional M3B goals in
  docs/CATALOG_COVERAGE_PLAN.md; package coverage and actual clean SQLite import
  in catalog/production/coverage.json and installation-report.json.
- Compiled artifacts install through the existing Rust worker/shared checked
  importer before requests. No network/renderer import. Same/hash startup skips
  audit; changed installed version content rejects; newer installed versions
  remain intact. CLI supports production without the validation opt-in flag.
  Attribution/license notice physically included in macOS bundle resources.
- Recipe editor's existing catalog-aware autocomplete works with real installed
  records. Turkish Settings/helper/milestone text updated; no advanced M3C UI.
  No fake statistics or demo production records.

### Preservation and verification

| Check | Final actual result |
|---|---|
| npm run typecheck / lint / build | Passed |
| npm test | 32 passed across 10 files |
| npm run test:ui | 2 passed; final sandbox localhost EPERM resolved by authorized retry |
| cargo check --locked --offline | Passed |
| cargo test --locked --offline | 48 passed, including retained crash-process helper tests and M1/M2/M3A regressions |
| cargo fmt --check / Clippy all targets -D warnings | Passed |
| Production generator repeat | Byte-for-byte JSON reproducibility passed |
| Real clean CLI import / repeat | 235 validated/inserted/available, 0 collisions; repeat unchanged=true, inserted/updated 0 |
| Real schema-3 and validation-v1 upgrade fixtures | Passed; recipe/draft/archive/trash/IDs preserved |
| Personal Şeker/override upgrade | 235 definitions, 234 materialized catalog records plus original personal row; 1 pending collision; recipe/draft refs and honey name/notes/kg override retained |
| Transaction failure / duplicate / version drift | Rejection and rollback/retry passed; foreign-key/integrity checks passed |
| Isolated and normal macOS arm64 unsigned debug bundle | Built successfully; NOTICE resource verified |
| Native packaged UI→IPC→SQLite / complete Quit/restart | Passed on isolated profiles; details below |
| Isolated Tauri dev launch | Reached Running native executable on port 1430, stopped with Ctrl-C |
| Windows/Intel / signed installers/notarization | Unverified |

Five added SQLite production tests cover embedded/disk contracts, full coverage
report, bilingual/alias/Turkish searches, counts/provenance, recipe restart,
version-1 personal collisions/drafts/overrides, trigger rollback/retry,
default worker offline bootstrap/restart/no repeat audit, newer-version retention
and same-version drift. Existing schema-3 upgrade test now installs production
as well. Historical IPC fixtures retain explicit seed-free workers; the added
production test uses the real default worker. No frontend mock is persistence proof.

During development Clippy found two unnecessary borrows, a UI copy assertion
failed, and a new coverage assertion had mismatched integer test types; all fixed
and final suites rerun. No unresolved failed check or timeout.

### Native evidence / limits

Initial isolated com.recipeatlas.m3b1-verification package installed 235/0/0
production/validation/collision counts with real schema 4, FK and FTS5. Created
“ M3B-1 Domates Denemesi ” with bundled Domates (çiğ, kırmızı), 200.000001 g and
“Domatesi doğrayın.” by keyboard selection and save. Explicit menu Quit,
confirmed stopped app inventory, relaunch showed exact saved values.

Final isolated com.recipeatlas.m3b1-final-verification package showed updated
M3B-1 Settings copy and 235/0/0. Native ICING selected Pudra şekeri; recipe
“M3B-1 Pudra Şekeri” with 35.000001 g and “Pudra şekerini eleyin.” survived
confirmed menu Quit/relaunch. Final recipe helper copy tweak was frontend-tested
and built, not manually reobserved. GUI proof is packaged runtime; dev proof is
successful CLI launch output. No physical network disable; compiled artifacts
and packaged workflows need no API/Vite server. Isolated data was retained, not
purged. The user's normal production database was not opened by this verification.

### Remaining coverage / exact next task

M3B-1 is a first genuine batch, not completed M3B or release-ready V1. Regional
Turkish and cocktail bitters groups remain zero; seven international-specialty
members overlap existing food/drink groups. Gin/tequila/rakı, dry tea/coffee forms,
mineral water, simple syrup, many dairy/meat/fish/baking forms and regional
specialties need source/terminology review. Missing empirical metadata remains
unknown. Full coverage plan targets 800–1,000 reviewed unique identities with
explicit contingent group targets, not achieved counts.

Recommend **M3B-2**, when authorized: expand to 450–600 verified generic identities,
prioritize common gaps; review lawful primary regional/beverage references and
independent Turkish terminology; generate version 3/delta report/new immutable
source snapshots, preserving source exclusions until cleared; rerun clean,
upgrade, user-preservation and native gates. No M3C UI redesign.

Normal local launch: `npm run tauri -- dev` (now installs the approved first batch
safely on initial bootstrap). Isolated launch:

```sh
npm run tauri -- dev --config '{"identifier":"com.recipeatlas.m3b1-dev-verification","build":{"beforeDevCommand":"npm run dev -- --port 1430","devUrl":"http://localhost:1430"}}' -- --locked --offline
```

Reproduction commands and archive hash: docs/CATALOG_IMPORT_PIPELINE.md.
Completed M3B-1 changes are recorded as one local milestone commit; obtain the exact
commit hash with `git log -1`. No push requested or performed.

## M3B-2 checkpoint — 2026-10-09 (in progress)

Preserved the exact committed version-2 package in `catalog/releases/2` and its
reproducer `tools/prepare_production_v2.py`. Added reviewed USDA food batch
`catalog/production/batches/food.tsv`: 126 new identities, 361 total at this
checkpoint. Generator produces version 3 with new immutable source snapshots
`2018-04-selection-3` / `production-2`; previous snapshots remain approved.
Real CLI clean import into `/private/tmp/recipeatlas-m3b2-food-batch` validated and
inserted 361, zero collisions. No user database touched, no migration changed.
Pending: second food/drink batch, authored regional/spirit references, full
localization audit, actual version-2 upgrade tests, regression/build/native gates.
Do not treat these development version-3 artifacts as a released immutable seed;
use fresh isolated DBs after content changes. Resume from the saved batches.

### M3B-2 second checkpoint (implementation ready for full gates)

Version 3 now has 482 identities: 396 food / 86 beverage, 253 aliases,
50 category nodes; 247 additions over archived v2. USDA batches contribute
208 new source-verified forms; 39 original reference-backed entries add regional
and bar identities, without pretending to have USDA IDs or empirical metadata.
All 235 historical IDs survive. `tools/audit_catalog.py` checks all 482 labels,
IDs, provenance, dimensions, categories and exact normalized alias collisions;
17 terminology questions are in the human-review queue, zero independent human
reviews. Unsupported paprika sweetness alias removed; almond butter/paste aliases
narrowed. Focused real SQLite suite: 21 tests passed (30 other tests filtered).
Actual v2→v3 upgrade, personal collision, recipe/draft/override preservation,
failed upgrade rollback/retry and search timing passed. Full regression/build/native
checks remain pending; see files on disk, do not rerun completed source discovery.

## M3B-2 completed — 2026-10-09

### Delivered

Production **version 3**: **482 canonical identities**, **247 additions** over v2;
**396 food / 86 beverage**, **252 aliases**, **50 category nodes**. 443 unchanged
USDA source descriptors/IDs; 39 original primary-reference-backed project entries
(no fabricated USDA IDs, no third-party recipes/database/images copied). All
482 have project-authored bilingual labels/classification and unknown optional
nutrition/density/allergens/ABV. Final review removed an awkward almond-butter
alias rather than invent a synonym, reducing the interim 253 count to 252.
Eight validation identities stay separate/shared by stable ID, not added twice.

Grouped TSV batches, deterministic generator, reference rights manifest, audit
and audit safety tests added. Exact v2 package archived byte-identically against
commit 5100f37138b9f7ea3e86ed6653ba17ca7b4a7092. All five generated artifact hashes
match a repeat run. All 235 previous IDs intact; schema remains 4, migrations
001–004 unchanged. No dependencies, architecture replacement or M3C redesign.
Turkish shell/editor copy now describes expanded M3B-2 catalog.

Full category counts and source/quality distinctions:
`docs/CATALOG_COVERAGE_REPORT.md`, `catalog/production/coverage.json`.
Seventeen human terminology review items in
`docs/INGREDIENT_LOCALIZATION_REVIEW.md`; zero independent human review claimed.
Structural audit: no errors, missing bilingual labels or exact alias collisions.
Two selection exclusions recorded (black turtle bean distinction and unavailable
Van-cheese reference); zero rejected accepted-package records.

### Verification

- 51 Rust tests pass, including retained M1/M2A/M2B/M3A/M3B-1 regression coverage.
- 32 Vitest tests and 2 Playwright browser tests pass; browser mocks/UI are not native persistence evidence.
- Two Python audit tests pass, including five injected defect subcases; `audit_catalog.py --check` passes.
- TypeScript, ESLint, Rust fmt/check, Clippy --all-targets -D warnings, frontend production build and `git diff --check` pass.
- Real CLI clean import: 482 inserted/available, 0 collisions. Actual archived v2 upgrade: 247 inserted / 235 updated / 482 available / 0 collisions. Reports saved with dataset.
- Real test upgrade preserves personal Cin as a pending collision, all old IDs, custom ingredient/recipe/draft references and name/notes/unit overrides; repeat import and failed-upgrade rollback/retry pass; integrity/FK checks pass.
- Focused debug timing: upgrade ~675 ms; 100 SQLite Turkish partial queries ~287 ms. Local observations only, not a hardware-independent SLA.
- Unsigned debug macOS arm64 package builds pass. Final isolated profile `com.recipeatlas.m3b2-final-verification` showed schema4, FK/FTS verification and 482/0/0 counts, updated M3B-2 copy; SİYEZ search/keyboard selection saved “M3B-2 Son Doğrulama” with Siyez bulguru 35.000001 g. Confirmed menu Quit/process absence/relaunch/detail retained exact quantity/reference.
- Earlier isolated package also verified English alias London dry gin, selected London gin 25 mL and a preparation step, then full quit/restart retained both ingredients/step. Final package retains that search implementation/data and automated coverage; final manual recipe was simpler.
- Isolated `npm run tauri -- dev` on port1433 compiled and ran `target/debug/personal-recipe-library`, then intentionally stopped after smoke verification (no timeout/failure).

No normal personal recipe DB was opened/reset by verification. Intermediate/final
isolated profiles were retained, not erased. Native automation first-launch tool
calls took 341 and 888 seconds; later relaunches were under 2 seconds and app
initialization/flows succeeded. Treat this as unresolved automation/OS launch
latency, not a measured DB import timeout. Physical network disabling was not
performed; packaged tauri://localhost workflows require no Vite/API/external seed.
Windows/Intel runtime and signing/notarization remain unverified release gates.
Full details and exact reproduction commands are in TESTING.md and
CATALOG_IMPORT_PIPELINE.md.

### Next task and handoff

**M3B-2 is complete within its requested expansion scope; M3B as a whole is not.**
Do not begin M3B-3 or M3C without authorization. Recommend M3B-3: human culinary
review of the 17-item queue; remaining Turkish regional cheeses/ferments/bulgur/
pulses/plants, international specialties, rum/vodka/tequila styles, vermouth/beer,
coffee beans/tea leaves, tonic/bar syrups. Keep unknown metadata unknown and
source exclusions until separately cleared; generate version4/new snapshots,
retain v2/v3 fixtures and rerun upgrade/preservation/native gates.

Normal launch: `npm run tauri -- dev`. Fresh isolated launch:

```sh
npm run tauri -- dev --config '{"identifier":"com.recipeatlas.m3b2-dev-verification","build":{"beforeDevCommand":"npm run dev -- --port 1433","devUrl":"http://localhost:1433"}}' -- --locked --offline
```

Earlier development version3 profiles use intermediate hashes; do not silently
replace those applied packages. Use a fresh test profile to reproduce the final
seed, or retain its matching original intermediate artifact. Normal v2 user DBs
upgrade to final v3 safely. Completed verified changes are committed locally as
one milestone commit; obtain hash with `git log -1`. No push requested/performed.


## M3B-3 checkpoint — quality tooling (2026-10-09)

Work in progress; not a completed milestone. Archived the exact 482-record version-3 package and its reproducer for upgrade testing. The audit now protects all version-3 IDs, validates category parent graphs, source/evidence checksums, external identity ownership and ingredient types. Two focused Python tests pass (including five injected defects). Created UTF-8 CSV and detailed Markdown for all 17 original terminology items; none is human-approved and existing display names remain unchanged. Production seed is still version3/482 records at this checkpoint.

Next: evidence-backed small expansion, version4 generation, targeted QA/persistence upgrade tests, full regression/build checks, isolated native verification, final coverage/progress and local commit. Preserve all current edits; do not restart or modify migrations.

### M3B-3 checkpoint — dataset and verification

Generated version4: 508 identities (26 additions), 411 food / 97 beverage, 260 aliases, 50 category nodes; 454 USDA-backed / 54 project-only reference-backed. All 482 v3 IDs preserved. No empirical metadata added. Original 17 review items retained; new endive Turkish terminology concern adds one explicit pending item (18 total, zero human approvals). CSV regenerated and round-trip validated.

Real offline CLI installation and actual v2/v3 upgrades passed in fresh temporary databases with network denied per process; eight import/open runs recorded. Repeated imports report no changes; Rust historical-upgrade tests also assert no extra SQLite writes on repeated startup. 52 Rust tests, 32 frontend tests and 2 browser tests passed before the latest targeted search additions; final complete suite rerun remains required. Native isolated arm64 package showed 508 records and verified Turkish/English/alias searches, saved sugar 35.000001 g and Orgeat 25.000001 mL, and retained both after menu Quit, process absence and relaunch. Normal user profile was never opened.

Remaining: final documentation/coverage report, final suite and updated package label build/smoke, local commit. No M3C changes.


## M3B-3 completed increment — 2026-10-09

### Delivered

Production seed version 4 has **508 canonical identities**: **26 additions**, **411 food / 97 beverage**, **260 aliases**, **50 category nodes**. **454 USDA-backed / 54 project-only reference-backed identities**, with project-authored localization for all 508. Every existing 482 ID, display name and alias remains unchanged. No applied migrations, dependencies or runtime architecture changed. Exact version 3 fixture and reproducer retained alongside version 2. New immutable evidence snapshots selection4/production-3 are explicitly approved.

Added 11 pinned USDA identities (vegetables, bran, coconut milk forms, agave, tomato juice and oil) and 15 original primary-reference-backed identities (bulgur grades, nar ekşisi, Edirne cheese, beer styles, bar syrups and rum forms). Coverage details and known gaps are in docs/CATALOG_M3B_FINAL_REPORT.md and catalog/production/m3b3-final-report.json. Sources/rights manifest preserves narrow reference-only reuse; no restricted databases, recipes/images, nutrition, density, allergens or ABV imported.

Stronger reproducible QA checks all 482 historical IDs, category graphs, relationships, checksum/evidence ownership, approved source versions/licenses, references, aliases, type assignments and labels. 25 qualified-name groups remain transparent heuristic concept candidates, not automatic merges. No structural errors or exact alias collisions. All 17 original terminology items remain pending; one new endive/hindiba question gives **18 human-review items**. Detailed UTF-8 CSV and Markdown include identity, provenance, concern, retained proposal, confidence and human-review status. Zero independent human approvals.

### Final verification

- **53 Rust tests**, **32 Vitest tests**, **2 Playwright tests**, **4 Python audit tests** pass. Rust retains M1–M3B2 regression coverage and adds actual v3 upgrade, no-write repeat startup and real full-production killed-import rollback/retry.
- TypeScript, ESLint, frontend production build, Rustfmt/check/Clippy all-targets with warnings denied, Python audit freshness and git diff whitespace checks pass. Initial stale count/assertion and exporter-shape failures were corrected and rerun; no unresolved test failure or timeout. Playwright sandbox listen EPERM resolved by permitted local-server execution.
- Byte-for-byte version 4 manifest and ingredient regeneration verified against pinned archive. 18 complete unique UTF-8 CSV rows round-trip correctly.
- Eight actual CLI runs in new temporary profiles with process-local network denial: fresh 508 inserted; v2→v4 273 new / 235 updated; v3→v4 26 new / 482 updated; all repeats unchanged. Preservation tests retain personal IDs, recipe/draft references, notes/name/unit overrides and explicit pending collisions. Integrity/FK checks pass.
- Unsigned debug macOS arm64 package builds; actual isolated `com.recipeatlas.m3b3-verification` runtime shows 508 production / 0 validation / 0 pending and schema 4/FK/FTS verified. Turkish/English/alias search examples succeeded, including İ/i and I/ı. Keyboard-selected sugar 35.000001 g and Orgeat 25.000001 mL saved; menu Quit confirmed process absence, then relaunch/detail retained exact values. Final rebuilt package also showed Aşama 3B-3 and retained recipe. Native GUI network was not blocked; separate real importer processes were network-denied.
- Measured local debug CLI clean initialization/import 238.593 ms, repeat open 80.939 ms; v2 upgrade 232.644 ms,v3 upgrade 230.483 ms. Focused Rust imports 190.415 / 186.139 ms; 100 actual Turkish partial searches 80.595 ms. These are recorded local observations, not cross-hardware guarantees.

No real personal profile was opened/reset; temporary CLI profiles were allocated by tests, native verification profile retained. No push performed. Completed verified changes are a logical local milestone commit; use git log -1 for its hash.

### Remaining boundaries and next step

**Requested M3B-3 increment is complete.** Human terminology review remains open by design; labels are project curation, not certified translations. Global catalog still has tracked coverage gaps: regional cheeses/ferments/plants, international specialties, coffee beans/tea leaves, vermouth/additional spirits and mixer varieties. Unknown empirical metadata remains absent. Source exclusions and V1 goals are unchanged. Windows/Intel macOS runtime/installers and signing/notarization remain pending host/release gates. The frontend build is production-mode; the verified native package is an unsigned debug build.

Recommended next authorization: **M3C** richer catalog autocomplete/discovery, category/origin filtering, accessible keyboard interactions, provenance/unknown-metadata display and safe user override/collision review with real SQLite tests. Do not begin automatically. Human review can correct labels through a future version without reassigning IDs or overwriting personal edits.

Local launch: `npm run tauri -- dev`. For isolated verification package:

```sh
npm run tauri -- build --debug --bundles app --config '{"identifier":"com.recipeatlas.m3b3-verification"}' -- --locked --offline
```

Reproduction, exact offline test commands and measured reports are in CATALOG_IMPORT_PIPELINE.md and TESTING.md. No M3C or unrelated feature work implemented.
