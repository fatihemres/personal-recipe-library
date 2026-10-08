# Project progress

## Objective and current milestone
Local-first food and beverage recipe management for macOS and Windows with Tauri 2, React, TypeScript, and SQLite.

Current state: M1 foundation implemented and verified on macOS arm64, 2026-10-08. M2 and later modules remain pending. Initial inspection/planning history is retained below; see the M1 entry for current code, tests and next task.

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
