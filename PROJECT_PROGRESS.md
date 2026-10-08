# Project progress

## Objective and current milestone
Local-first food and beverage recipe management for macOS and Windows with Tauri 2, React, TypeScript, and SQLite.

Current state: M1, M2A and M2B implemented, 2026-10-08, with actual SQLite tests and native macOS arm64 evidence. Next task is M3; no catalog import has started. The M2B handoff at the end is current; earlier sections are historical and retain their original milestone boundaries.

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
