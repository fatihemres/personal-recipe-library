# Project progress

## Objective and current milestone
Local-first food and beverage recipe management for macOS and Windows with Tauri 2, React, TypeScript, and SQLite.

Current state: M1 through M3D-2B implemented and verified locally, subject to the explicit host/GUI limits below. The latest dated handoff at the end of this file is authoritative; earlier checkpoints are historical.

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


## M3C-1 completed — unified ingredient library (2026-10-09)

Implemented Malzeme Kütüphanesi over real SQLite: all/catalog/personal views with global counts;30-row pages; bilingual names, aliases, partial Turkish-aware searches; hierarchical categories with descendant facets; catalog read-only details and provenance/unknown metadata; retained personal create/edit/delete with existing revisions and reference protections. New Rust read models/repository and worker IPC, Zod contracts/client, modular library/detail/personal-editor components, Turkish strings, theme-aware cards and filters replace the personal-only manager. No migration, dependency, capability, seed or identity changes. Existing schema4/508 identities/260 aliases/50 category nodes remain intact. Recipes use the same recipe-facing IDs/search; pending catalog collisions remain visible without silent merging.

Verification passed:58 Rust tests (four new real SQLite repository tests and one real-worker IPC test),38 frontend tests,2 browser regressions,4 Python audit tests,508-record audit; TypeScript, ESLint, Cargo fmt/check/Clippy warnings denied, frontend production build and unsigned macOS arm64 debug app build. No final failed automated checks. An initial IPC test incorrectly expected IPA to match only one record; substring search legitimately also finds spinach, so the precise command fixture now queries India pale ale while repository tests ensure no duplicate IPA identity. Native testing discovered a stale category-menu count race after new search input; disabling category selection until fresh facets arrive fixed it.

Native isolated profile com.recipeatlas.m3c1-verification started508 built-in/0 personal; all17 pages browsed (508 cards). Created and renamed a personal ingredient; actual totals508 built-in/1 personal/509 combined. Sugar details read-only and source169655/CC0 verified. Şek/şek/ŞEK/şeker each15matches; Granulated sugar1; Orgeat1; RAKI/rakı1; SİYEZ/siyez3; edited personal name1. Peynirler30 and combined Feta1 verified. Light/dark visual checks passed. Recipe keyboard selection saved Orgeat25.000001mL plus personal35.000001g; edit retained them. Cmd-Q followed by confirmed process absence and final rebuilt app restart recovered the unfinished edit draft, exact quantities and personal record; explicit edit save succeeded. Normal personal application profile was never reset/opened for these tests. Test data remains only in isolated profiles outside Git.

Focused warmed SQLite measurement:17 pages containing508 distinct definitions244.900ms total, including facets/memberships, with no database writes. Not a platform-wide performance guarantee. Native automation initial launch took391.24s before returning; restart1.73s. Initial tool latency does not establish SQLite initialization time or failure. Packaged GUI evidence uses tauri://localhost and embedded seed; no API/server required. Physical network disconnection was not separately tested.

Limitations: Windows/Intel macOS, signing/notarization and installers remain pending.18 existing linguistic review items remain open, no human approval claimed. Library order is deterministic normalized lexical order; advanced ranking/linguistic sorting remains future work. Source URLs are readable text. Personal ingredient forms use explicit Save/Cancel; Cancel confirms dirty edits, but sidebar navigation does not persist unfinished ingredient edits (recipe draft protection is preserved). No new UX blocker observed in tested library workflows.

Next: M3C-2 favorites, recently used ingredients, improved autocomplete ranking/interaction, safe user-specific catalog presentation overrides, explicit collision/conflict management and personalization. This increment does not implement those features or beverage measurements. Native recipe integration, typed SQLite read models and existing invariants are the base for the next authorized task.

Launch locally: npm run tauri -- dev. Exact checks and isolated package commands are in docs/TESTING.md. Local commit is created after final verification; no push.

Final isolated development smoke: Vite1434 and identifier com.recipeatlas.m3c1-dev-verification compiled and reached Running target/debug/personal-recipe-library, then stopped cleanly with Ctrl-C. Browser/native package verification is separate from this CLI launch evidence. Git diff whitespace check passed; migrations and catalog artifacts have no changes.

## M3C-2 completed — Ingredient Library personalization and intelligent selection (2026-10-09)

Checkpoints A/B/C delivered: separate persisted favorites for canonical and personal IDs; recipe-derived recent usage (current non-deleted saved recipes, latest `recipes.updated_at`); favorite/recent library filters and recipe-selector shortcuts; bounded database-ranked Turkish/English/name/alias suggestions with exact/prefix precedence and favorite/recent tie breaks; keyboard Up/Down/Enter/Escape; canonical presentation name, personal alias, compatible unit and notes with revision-safe reset; cross-origin/alias duplicate warnings and explicit use-existing or proceed; and Save/Discard/Continue guards on ingredient creation, personal editing and catalog presentation editing. Thin typed Tauri IPC and the existing Rust worker remain the only persistence route. Migration 005 is additive/checksummed and leaves 001–004 byte-for-byte unchanged. Catalog seed version 4, all 508 identities, 260 bundled aliases and 50 category nodes are unchanged; no ingredients were fabricated or reimported on search.

Full checks pass: TypeScript, ESLint, 42 frontend tests/13 files, frontend build, 2 Playwright browser tests, Rust fmt, 60 Rust tests, Cargo check, all-target Clippy with denied warnings, 4 Python QA tests and full 508-ID audit. Real SQLite tests cover schema-4 upgrade, favorites/recent and overrides across restart, alias collision rollback, ranking, reference/quantity preservation and old regressions. Native unsigned arm64 debug package (36.33 MiB) ran with `com.recipeatlas.m3c2-verification`, isolated from personal data. Created one personal ingredient and saved a recipe with exact `35.000001 g` personal and `25.000001 g` canonical lines; native restart retained 508 built-in/1 personal, two favorites, two recently used entries and the recipe/quantities. Native UI verified canonical presentation change/reset, duplicate warning, sidebar unsaved-edit decisions, and final M3C-2 label. Isolated SQLite integrity/FK checks passed; schema version5. Exact commands and evidence: TESTING.md.

Limitations: Windows/Intel macOS, signed/notarized builds and installers remain unverified. Native network isolation was not explicitly toggled; app loaded embedded assets without Vite and runtime operations use SQLite. Eighteen catalog terminology items still need human review. Recent usage is a query over *current* saved recipe contents, not a historical use log; removing the last current reference removes that ingredient from recent. Initial built-in records blocked by an exact personal collision can be viewed/favorited but cannot be presentation-edited until explicitly linked/materialized. Duplicate warnings are advisory for similar names; there is no automatic identity merge or full fuzzy typo tolerance. These limits do not alter M3C-2 acceptance or the wider V1 scope.

Next scoped task: review remaining M3 category/collision and ingredient-use ergonomics, then plan the next authorized milestone from IMPLEMENTATION_PLAN.md without beginning beverage measurements, pantry, shopping or planner implicitly. Preserve schema-5 user data and user-owned presentation aliases in every future catalog update. No push, tag or release in this milestone.

### Resumed final verification — 2026-10-12

The interrupted M3C-2 implementation was preserved in local WIP commit `075c1c9`; this pass changes only the shared ingredient leave dialog and related tests/documentation. The existing HTML modal now gives initial focus to Continue Editing, restores the originating focus on Cancel/Escape, blocks all decisions during a save, and displays save errors inside the modal without dropping edits. Create/edit/catalog-presentation flows all pass busy/error state into the shared dialog. No migration, catalog package, data identity, quantity or recipe workflow changed after the WIP commit.

Final available checks passed: TypeScript, ESLint, 47 Vitest tests across 15 files (including new modal focus/Escape/busy and personal/catalog save-failure cases), frontend build, 2 Chromium browser tests, 60 real SQLite/Rust tests, Cargo fmt/check/Clippy with denied warnings, 4 Python audits, 508-identity catalog audit and whitespace check. An unsigned arm64 package was built and launched under the new isolated identifier `com.recipeatlas.m3c2-final-verification`; its SQLite profile initialized schema 5 with 508 catalog identities, 260 bundled aliases, 50 categories, 0 personal records and valid integrity/FKs. Repeated native package launch retained those counts and one production release. An isolated `tauri dev` smoke reached `Running target/debug/personal-recipe-library`. Previous WIP native GUI testing verified ingredient favorites, recent usage, canonical presentation/reset, duplicate warnings, recipe exact quantities and restart with 508 built-in/1 personal; the normal profile was never touched. Direct GUI automation for the new focus restoration was unavailable in this resumed session, so final-focus behavior is covered by automated component tests and native build/launch, not claimed as a fresh manual GUI observation. Windows/Intel/signing limits remain as above. M3C-2 is complete within these stated verification limits; no M3C-3 work began.

## M3C-3 checkpoint — 2026-10-12, verification underway

Started from clean commit `9ed52b9`. Existing 508 identities, 260 aliases, 50 categories, migration 005 and prior user data remain unchanged. Implemented a bounded SQLite candidate review with exact-name, alias and ambiguous similarity explanations; no identity merge. Added combined uncategorized/origin/search filtering and atomic revision-checked personal category assignment through existing `personal_category_memberships`. Existing category descendant/count queries remain authoritative. Category-linked personal deletion now removes only owned category memberships in the same transaction after reference/revision checks.

The pinned package audit produced `catalog/production/m3c3-quality-report.json` and `docs/CATALOG_M3C3_QUALITY_AUDIT.md`: zero structural errors/exact alias collisions, 25 possible form-comparison groups (not proven duplicates), and 18 terms still awaiting human review. `docs/INGREDIENT_TERMINOLOGY_REVIEW.xlsx` contains all 18 IDs, bilingual labels, aliases, categories, concern, proposal, source links and blank human-decision columns; no term was approved or silently changed. A new script regenerates the static report. Focused SQLite and renderer tests have run, while final full-suite/native checks and documentation remain in progress. Next: finish checks, verify isolated native build/launch where possible, update testing/architecture/plan/changelog, then commit only if green.

### M3C-3 final verification and handoff — 2026-10-12

All source data, 508 IDs, 260 aliases, 50 category nodes, personal records, favorites, overrides, recipe quantities/drafts and migrations 001–005 remain unchanged. Final collision candidate results are bounded to six, ranked by exact name then exact alias then weaker similarity; weak results are advisory, strong matches prevent creating a misleading new personal record, and no identity is merged. The category selector shows empty branches and an uncategorized option, while the personal editor displays the hierarchy and saves category assignments with its normal revision-checked edit. Catalog-linked personal records inherit system categories and intentionally do not expose assignment editing until direct versus inherited links are distinguishable.

Full checks passed: 50 Vitest tests/15 files, 63 Rust tests, 2 Chromium browser tests, TypeScript, ESLint, frontend build, Cargo fmt/check/Clippy, four Python catalog audit tests, fresh package audit/check and whitespace check. Real SQLite tests cover atomic category rollback/restart, Turkish collision evidence, parent distinct counts and all earlier migrations/recipe/draft/personal/favorite/override regressions. The unsigned arm64 debug bundle launched under isolated identifier `com.recipeatlas.m3c3-verification`. Native UI showed 508 built-in/0 personal fresh; one disposable personal ingredient was assigned to Otlar, found through the Gıda parent, and survived full app restart with 508/1 counts. `Şeker` surfaced Toz şeker as an alias candidate and prevented misleading creation. Isolated profile SQLite integrity and FKs passed at schema 5. The normal user profile was untouched. Details and host limits are in TESTING.md. No Windows/Intel/signing/notarization verification was performed. The 18 terminology decisions remain human-pending; 25 concept groups are review candidates, not confirmed duplicates.

Next requested scope should be explicitly chosen after review: decide the 18 terminology questions with a qualified human reviewer, then handle explicit catalog/personal linking and direct-versus-inherited category edits without merging automatically. Broader V1 modules in MASTER_SPEC.md remain planned; M3C-3 does not start them. Use `git log -1` for the local milestone commit hash; no push is authorized.

## M3C-4 completed — three user-selected ingredient names (2026-10-13)

Starting from clean M3C-3 commit `dfce250`, versioned offline catalog package 5 applies exactly three Turkish labels: `usda-sr-170918` → Frenk kimyonu tohumu, `usda-sr-168921` → Tam tahıllı mavi mısır unu, and `usda-sr-168588` → Badem püresi (sade, tuz ilavesiz). Version 4 is archived unchanged; a separate checksummed project-owned terminology snapshot records the selected names without changing USDA facts, older curation evidence, IDs, English labels, aliases, categories, observations, or recipe data. Old Turkish labels were not automatically added as aliases. All 508 identities, 260 aliases and 50 categories remain. Schema stays at 5, migrations 001–005 unchanged. The reproducible CSV/workbook/quality report records three user decisions and leaves 15 pending; 25 concept groups remain candidates only.

Verification: 66 Rust real-SQLite tests, 50 Vitest tests, 2 Chromium browser tests, 5 Python audit tests, TypeScript, ESLint, Rust format/check/Clippy, frontend production build, catalog audits and diff whitespace passed. Eleven real CLI imports/opens with process-local network denial verified fresh/v2/v3/v4→v5 and no-write repeats; v4→v5 updated exactly three records. The measured report is `catalog/production/m3c4-installation-report.json`. Fresh and archived-v4 upgrade tests prove idempotency, override/personal/recipe/quantity/foreign-key integrity and conservative collision rollback. An unsigned arm64 native debug app launched under isolated `com.recipeatlas.m3c4-verification`; its actual SQLite profile contained catalog release 5, 508/260/50 records, all three names, one successful import and clean integrity/FKs. Final rebuild/relaunch kept the successful-import count at one. Native GUI automation could not bind to the running window, so fresh GUI search/detail interaction is unverified here; real SQLite-backed search/detail tests passed. Normal personal app data was not touched. Windows/Intel, signed installers and notarization remain untested.

Next authorized work should address the 15 remaining human terminology decisions or an explicitly scoped collision-linking workflow; do not infer approval of those names or start catalog expansion, measurement, pantry or other modules from this task. Use `git log -1` for the local M3C-4 commit; no push.

## M3C-5 completed — final scoped terminology decisions (2026-10-13)

Production catalog release 6 was generated from the archived, checksum-pinned release 5 package. Exactly two canonical display-name fields changed: `usda-sr-174842` Turkish → **Club soda (kokteyl sodası)** and `project-siyez-grain` English/canonical → **Einkorn wheat (siyez)**. Ten further records retain their existing names under the user's current limited decision; three (`usda-sr-171319`, `usda-sr-170874`, `usda-sr-168412`) remain unresolved. The 3 earlier M3C-4 labels remain applied. No independent expert linguistic verification is claimed. Review CSV/workbook and quality reports distinguish all four groups, retain source links and have blank reviewer/date fields. The 25 possible concept groups remain unchanged. IDs/aliases/categories stay **508/260/50**; no ingredient added, identity merged, alias added or migration changed. Schema remains 5.

Verification: **69 Rust tests**, **50 frontend tests**, **2 browser tests**, **5 Python audit tests** passed, as did TypeScript, ESLint, Rust format/check/Clippy, frontend build and catalog report checks. A first combined browser run hit a transient localhost `EPERM`; a standalone rerun passed. Fourteen actual isolated CLI imports/opens under process-local network denial covered fresh/v2/v3/v4/v5→v6 plus unchanged repeats; v5→v6 updated exactly 2. Real SQLite tests cover name collision rollback, recipe and draft reference stability, exact quantity/unit, favorites/recent, user override and personal ingredient preservation. An unsigned arm64 native app bundle built and launched with isolated `com.recipeatlas.m3c5-verification`; after closing all test processes and fully relaunching, its SQLite profile still contained release 6, 508/260/50, the two names, one successful import, integrity `ok` and zero FK violations. The final test process was closed. The normal profile was not opened. The native GUI could not be bound by computer-use tooling, so fresh GUI interaction is unverified; Windows/Intel/signing/installers remain unverified. Details and exact commands are in `docs/TESTING.md`.

Next task requires new user scope. Three terminology questions still need review; other V1 modules remain in MASTER_SPEC.md. Do not infer approval of additional labels or start M3C-6/measurement work. A pre-existing untracked root-level `INGREDIENT_TERMINOLOGY_REVIEW.xlsx` was left untouched; the tracked workbook is `docs/INGREDIENT_TERMINOLOGY_REVIEW.xlsx`. No push.


## M3D-1 completion — controlled release 7 (2026-10-14)

Resumed uncommitted work from 7edf0e8 without resetting it. The prior session had already rebuilt the importer and measured 569/269/50, but the full suite still contained release-6 assumptions. Reviewed and updated current-package assertions while preserving archived v5/v6 terminology tests. No migrations or user profiles were changed.

The prepared 61 candidates were screened further. Corrected hulled barley to Kabuğu alınmış arpa, cranberry juice to Şekersiz turna yemişi suyu, and kale's ambiguous karalahana alias to Kale. Deferred USDA 169406 unroasted-kernel tahini because overlap with existing raw stone-ground tahini is unresolved. Its original row/evidence remains in the deferred review artifact. Final release 7: **60 additions, 568 identities, 269 aliases, 50 categories; 464 food / 104 beverage**. All 508 complete historical ingredient objects remain identical, including the three unresolved M3C-5 terms. Three new naming questions plus the deferred identity remain human-pending in docs/INGREDIENT_M3D1_REVIEW.xlsx. No linguistic expert approval is claimed.

Approved new source extracts use the same pinned USDA CC0 archive and separate CC0 project-label provenance. FoodOn remains conditional, Open Food Facts/TheCocktailDB excluded. Added release-6 archive, deterministic offline evidence reproduction, source/coverage/quality reports and source-allowlist validation. No additional third-party dataset, UI feature, schema or dependency was introduced.

Checks passed: **72 Rust tests, 50 frontend tests, 2 browser tests, 5 existing Python audits plus 4 M3D-1 tests**, TypeScript, ESLint, Rust formatting/check/Clippy with denied warnings, frontend build, deterministic generation and historical/current audits. Real SQLite tests include every new bilingual name/alias/detail/category query, stable IDs, failed-upgrade rollback and retry, unauthorized source rejection before DB creation, exact recipe quantities/units, favorites/recent/overrides, drafts, integrity/FKs and restarts. 17 isolated CLI imports/opens with process-local network denial cover fresh and v2–v6 upgrades plus repeats.

Unsigned arm64 native bundle built and launched with com.recipeatlas.m3d1-verification. Complete test-process shutdown/relaunch retained release7, 568/269/50, zero personal test records, one successful import, integrity ok and no FK violations. Only processes confirmed to hold the isolated test database were stopped. Computer-use tooling could not bind the native window, so fresh native GUI interaction is unverified; backend integration and renderer tests are separate evidence. Normal personal data was untouched. Windows/Intel macOS, installers, signing and notarization remain unverified.

Reports: CATALOG_M3D1_COVERAGE_ASSESSMENT.md, CATALOG_M3D1_QUALITY_AUDIT.md, CATALOG_M3D1_SOURCE_DECISIONS.md and catalog/production/m3d1-{coverage,quality-report,installation-report,native-report}.json. The pre-existing root INGREDIENT_TERMINOLOGY_REVIEW.xlsx remains untouched/untracked (SHA-256 0a6e3e4cef3abba34d6d5b7295f155352a831b2c8970f7c6b9fc5993810c2b93). Final local commit follows verified diff review; no push.

Next recommended scope is M3D-2 targeted source/terminology review and high-value regional/beverage gaps described in IMPLEMENTATION_PLAN.md; it has not started.

Final development smoke also reached the native Running state with the isolated dev identifier on port1438, then stopped cleanly. Port1420 was occupied, so no existing server was stopped. Native/automated evidence and exact commands are recorded in docs/TESTING.md.

## M3D-2A — multi-source expansion preparation (2026-10-14)

Preparation completed against release-7 commit `0e7f9850e86e960d38c1bb1972b99c2c64f246cd`. No release 8, runtime code, schema, canonical records or source allowlist changed. Root `INGREDIENT_TERMINOLOGY_REVIEW.xlsx` remains untouched/untracked.

- New `catalog/preparation/m3d2a/` ledger: 194 source concepts, 106 USDA + 88 FoodOn; 154 food / 40 beverage proposals. 66 lower-risk USDA editorial proposals (61 food / 5 beverage), 40 USDA identity/terminology questions, 84 FoodOn source-clearance deferrals, 3 probable duplicate concepts and 1 rejected commercial name. Zero import authorizations/human approvals. The 200–500 ambition was not padded with artificial variants.
- Sources reviewed from official documentation: USDA CC0 remains eligible; FoodOn CC BY base pinned to 2026-09-20/git cd73540243a84bcd511a500d9a497d12d7dc02f6, with production lineage clearance still pending. OFF ODbL combination obligations unresolved; TürKomp permission/commercial contract restrictions exclude extraction; Wikidata structured CC0 is a future option; TÜRKPATENT reference-only; CocktailDB excluded. No records from the latter five sources imported/extracted.
- Rebuilt real importer and measured a new isolated SQLite profile: 568 canonical ingredients, 464 food / 104 beverage, 269 aliases, 50 categories; integrity ok, zero foreign-key violations. Direct and descendant unique counts in `docs/CATALOG_M3D2A_COVERAGE.md`; independent cuisine/family gaps and source decisions in `docs/CATALOG_M3D2A_PREPARATION.md`.
- Twelve candidate-to-catalog similarity flags assessed; three probable duplicate concepts (Ancho/Pasilla/malt syrup), two peel synonym questions, distinct forms and lexical false positives kept separate. One candidate-pair comparison (fenugreek leaf vs dried leaf). No merges. Original seven pending records preserved separately; deferred tahini stays excluded.
- Four-sheet `docs/INGREDIENT_M3D2A_CANDIDATE_REVIEW.xlsx` contains 66 higher-confidence rows, 128 other proposals + seven prior decisions, source licensing and 50 category rows. Blank human decision/reviewer fields, filtered tables and frozen headers. Generator/UTF-8 source ledger are reproducible; preserve a filled review copy before regenerating.
- Verification: candidate extraction matches both pinned source artifacts; offline generator drift check; ten new candidate tests, five existing catalog audit tests and four M3D-1 tests pass. Existing production/static audits pass. Protected-file SHA checks cover production, historical releases, validation, migrations and runtime allowlist. Workbook exported, all sheets visually reviewed, filters/freeze/blank decision fields inspected. No native GUI/Windows/Intel/signed-release testing newly claimed; runtime is unchanged.

Next: M3D-2B only after authorization—review the 66 lower-risk proposals, explicitly resolve/defer 40 USDA questions, clear any selected FoodOn terms individually, exclude probable duplicates/brand record, then create release 8 with fresh/upgrade/rollback/preservation/native checks. All source/identity/translation gates remain required; this preparation does not promise 194 additions.

## M3D-2B checkpoint — release 8 implementation and focused verification (2026-10-15)

Starting from committed M3D-2A `0db2fcf` and preserving the untracked root workbook, the release-7 package was archived byte-for-byte under `catalog/releases/7`. All 194 candidates now have explicit auditable editorial decisions: 79 USDA accepted (66 preliminary plus 13 individually resolved), 27 USDA deferred, 84 FoodOn deferred for term-level clearance, three FoodOn probable existing concepts and one brand-specific rejection. No FoodOn record enters the production package. The seven historical questions remain pending.

Deterministic release-8 generation currently yields **647 canonical identities, 269 aliases, 50 categories; 536 food / 111 beverage**. The 79 new rows use pinned USDA descriptors and separate project-authored bilingual curation; no empirical metadata, alias equivalence or human linguistic approval is invented. A dry beverage powder/instant form uses mass rather than volume. Schema and migrations remain unchanged. All 568 complete release-7 ingredient objects remain equal in the new package.

Focused verification passed: 72 existing Rust regressions before the new release-8 test; the new real-SQLite release-7→8 rollback, idempotency, search/category/detail, recipe/quantity, draft, favorite, override and restart test passes; five new Python reproducibility tests pass; structural audit reports zero errors and zero exact alias collisions. Real isolated SQLite fresh/upgrade measurement reports 647/269/50 with integrity `ok` and zero foreign-key violations. Twenty isolated CLI imports/opens across clean and v2–v7 profiles passed without network-denial wrapper; this host's `sandbox-exec` invocation exited 71 before the first import, so process-level offline denial remains unverified. No user profile was opened.

The final reports, full suite and isolated native startup/restart now pass. **73 Rust, 50 frontend, 2 browser, 5 new release-8 Python, 10 M3D-2A Python, 4 M3D-1 Python and 5 static-audit Python tests** passed; TypeScript, ESLint, Cargo fmt/check/Clippy and frontend/native debug builds passed. Twenty isolated CLI imports/opens passed with process-local network denial outside the nested command sandbox. A fresh native test bundle launched and, after complete restart, retained release 8/schema 5, 647/269/50 records, one successful import, integrity `ok` and zero FK issues. Native GUI binding failed because the computer-use app resolver used a stale identifier, so fresh GUI clicks/search/recipe interaction are unverified; SQLite-backed tests cover the paths separately. Normal user data remained untouched; test app was closed. Windows/Intel, signing/notarization and installers remain unverified.

M3D-2B is complete within those stated platform limits. Release-8 coverage, quality, source and native reports are committed artifacts. No M3D-2C or unrelated module began. Next recommended expansion is targeted source/terminology work on Turkish regional, fermented/dairy alternatives, East/South Asian and bar/coffee/tea gaps, with FoodOn term-level clearance or other legally usable primary sources. Seven historical and 27 deferred USDA identity/terminology questions remain pending. Local commit follows final diff review; no push.

## DEVOPS-1 PR transition checkpoint — 2026-10-15

Fetched `origin` and verified the private repository's default `main` is release-7 commit `0e7f985`, a direct ancestor of local release-8 HEAD `9738785`. The only commits ahead are M3D-2A `0db2fcf` and M3D-2B `9738785`. Created `feature/m3d-2b-release8-ci` at the same HEAD without moving local `main` or changing either milestone commit. The root untracked terminology workbook remains untouched and excluded.

Added a PR-only workflow with separate Ubuntu frontend/browser and Rust/SQLite/catalog jobs, then gated macOS arm64, macOS Intel and Windows x64 unsigned packaging jobs. It pins Node 24.21.0/Rust 1.99.0, uses npm/Cargo caches, cancellation, timeouts, read-only token permissions and seven-day artifacts. Added `docs/DEVELOPMENT_AND_RELEASE_PROCESS.md` to distinguish branch/commit/milestone tag/catalog release/app semver/GitHub Release and to require Product Owner review before merge. No application source, catalog identity, migration or user profile changed.

Local verification: `actionlint` v1.7.12 (checksum-verified temporary binary) passed; TypeScript, ESLint, 50 frontend tests, two browser tests, frontend build, Cargo fmt/check/all-target Clippy, 73 Rust tests, M3D-2A/M3D-2B/static Python audits and `git diff --check` passed. The first Playwright attempt failed to bind 127.0.0.1:1420 inside the command sandbox (`EPERM`); the outside-sandbox rerun passed. Rust tests include actual SQLite migrations, release upgrades, rollback and user-data preservation. No new native build or GUI interaction has yet been claimed for this DevOps-only change; GitHub PR results remain pending.

GitHub confirmed the repository is private with `main` as default and Actions enabled. Its branch-protection API returned HTTP 403, “Upgrade to GitHub Pro or make this repository public to enable this feature.” The project will remain private, with a documented manual no-direct-main-push/PR-checks-review policy; no settings, plan, visibility or merge changes were made. Next: commit only DEVOPS-1 files, push the feature branch, open the PR, inspect actual checks/artifacts and fix any CI failures. Do not merge without Product Owner review.

PR #1 was opened at `https://github.com/fatihemres/personal-recipe-library/pull/1`. Its first GitHub run passed frontend/browser, Rust/SQLite/catalog, macOS arm64 and macOS Intel; the Windows NSIS build, installer check and upload steps also passed while post-job cache cleanup remained pending at this checkpoint. Downloading all three artifacts exposed a checksum-manifest portability defect: macOS manifests contained runner-relative `artifacts/` prefixes and the Windows manifest used CRLF. The packages themselves were present. The workflow now writes paths relative to the artifact root and UTF-8 LF output; local `actionlint` and a downloaded-arm64 artifact checksum reproduction passed. Commit and push this correction on the same PR branch, then inspect the new full CI run and downloaded checksums before claiming final CI success.

## DEVOPS-1B cross-platform test follow-up — 2026-10-15

The Product Owner requested actual Rust/SQLite execution on all three native PR runners. Updated the existing workflow so macOS arm64, macOS Intel and Windows x64 each run the locked Rust suite against real isolated temporary SQLite databases before packaging; Ubuntu validation and all prior package/artifact steps remain. No application code, migration, catalog release or user data changed. Local `actionlint` and macOS Rust checks are the pre-push gates; the new PR run must establish the actual per-platform results before this follow-up is called complete.

Attempted native GUI smoke under new isolated app identifier `com.recipeatlas.devops1b-verification`. The debug app built, launched and fully restarted; its SQLite profile retained schema 5, catalog 647/269/50, integrity `ok` and zero FK violations. The normal profile was untouched and the test process was closed. GUI automation could not attach: computer-use bundle-ID lookup returned `Invalid app`, path lookup returned the stale M3D-1 identifier, and macOS denied `osascript` accessibility access (-1728). No GUI navigation/search/recipe behavior is claimed. Next: push the scoped workflow/docs commit to existing PR #1, inspect the new five-job run and report platform-specific Rust/SQLite results. Do not merge the PR.

PR run `38735778783` proved the frontend, Linux, macOS arm64 and macOS Intel test gates, but Windows `cargo test` compiled and then exited before any test executed (`0xc0000139`, `STATUS_ENTRYPOINT_NOT_FOUND`). This is a Tauri Windows test-harness Common Controls v6 manifest problem. A scoped Windows build-script correction now embeds the same dependency in app and test executables. Local macOS regression after this change: 73 Rust tests, Cargo check, Clippy, formatting and actionlint passed. Push the fix to the same branch, then require a new PR run to demonstrate Windows tests plus unchanged installer output before marking DEVOPS-1B complete.

Follow-up run `38737946424` exposed an MSVC quoting defect in the first build-script correction: the inline `/MANIFESTDEPENDENCY` link argument split at spaces (`LNK1181`), so Windows tests did not run. The Windows build script now passes a checked-in Common Controls v6 XML manifest through `/MANIFESTINPUT`, avoiding that argument parsing. The next PR run must demonstrate test execution and Windows installer packaging. Earlier failed runs are not completion evidence.

Run `38739559708` launched the Windows Rust test executable successfully (72 tests), exposing 41 catalog-backed `CATALOG_CHECKSUM` failures while 31 tests passed. The cause is Git's CRLF conversion of checksum-pinned catalog files on Windows. Added `.gitattributes` `catalog/** text eol=lf`; verified the attribute on production, historical and validation files and reproduced an identical production seed SHA-256 under `core.autocrlf=true`. The new PR run must establish Windows catalog test and installer success; no catalog content or checksums were changed.

Run `38741509466` passed all five jobs on commit `e12f028`: frontend/browser, Linux Rust/SQLite/catalog, macOS arm64, macOS Intel and Windows x64. Native Rust suites passed 73/73 on each macOS architecture and 72/72 on Windows; all platform packaging and artifact-upload steps passed. The three downloaded unsigned artifacts and five package checksums verified. Local isolated macOS app launch/restart and catalog SQLite integrity passed; actual GUI controls were not exercised because accessibility automation could not attach. Windows and Intel app launch, installer use and GUI workflows remain unverified. PR #1 remains open for Product Owner review; do not merge. This documentation checkpoint requires its own successful head-commit CI run before the PR is handed off.

## PR #1 urgent packaged-app startup repair (2026-10-16)

The Product Owner's installed arm64 binary is byte-identical to the prior PR artifact. Its existing SQLite profile's failed import audits identify the exact `AppError.code` **`CATALOG_SOURCE_CHANGED`**. The pre-existing release-7 profile has 569 identities because an early 61-record M3D-1 preview was installed; the final archived release 7 has 568. Two immutable M3D-1 source snapshots used the same source/version labels but different evidence hashes. The importer correctly rolled back release 8 and left the user database at release 7 with integrity `ok` and zero foreign-key violations.

The corrected bundled release 8 uses new reviewed source versions for the final 60-record M3D-1 evidence; original snapshots, IDs and user-owned records are retained. The exact earlier successful release-8 preview manifest/artifact pair is also allowed to remain installed without re-import, while arbitrary same-version drift stays rejected. No migrations or archived releases changed. Startup displays the stable error code without paths or private content. SQLite-backed regressions exercise the collision and preview compatibility gate.

Before the fix, the real importer returned `CATALOG_SOURCE_CHANGED` on a SQLite backup of the affected profile. After the fix, that isolated copy imported 79 new records, retained its historical extra identity (648 cumulative, versus 647 in the bundled package), and had intact recipe/draft/preference rows, exact quantities/units, old source snapshots, integrity `ok` and zero foreign-key violations. A production arm64 `.app` built with a temporary app-data override launched on a clean isolated profile and displayed the native recipe-library window; an isolated copy of the affected profile then upgraded through that same production bundle, restarted without another import and remained integral. A copy of an already successful release-8 preview profile also opened without a new import. The installed normal profile was never modified by this investigation. Local checks passed: 75 Rust tests, 50 frontend tests, 2 browser tests, Python catalog audits, TypeScript, ESLint, Cargo fmt/check/Clippy and frontend production build. One browser attempt hit sandbox localhost `EPERM`; the unsandboxed rerun passed. Final PR CI and downloadable artifact verification remain pending until the hotfix commit is pushed; Windows/Intel GUI interaction and public signing remain unverified. Next: commit and push only this branch, wait for all PR jobs, then return PR #1 for Product Owner review without merging.
