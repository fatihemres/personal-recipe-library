# Application testing

## Commands

```sh
npm ci
npm run typecheck
npm run lint
npm test
npm run build
npx playwright install chromium
npm run test:ui
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --locked --offline --manifest-path src-tauri/Cargo.toml
cargo test --locked --offline --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --offline --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npm run tauri -- dev
npm run tauri -- build --debug --bundles app --no-sign -- --locked --offline
```

`--offline` requires cached Rust crates; fetch dependencies on a connected development machine first. Playwright installation is a development-only network operation, not a runtime requirement. Tests use temporary DBs/files, not the personal application DB. UI test output and compiled artifacts are ignored by Git. Frontend typechecking covers src, Vite/Vitest/Playwright configurations and browser test source.

## Evidence boundaries

- Rust storage tests open actual SQLite files: initialization, FK enforcement, real FTS5 MATCH, repeat startup, preference save/reopen, validation, migration checksum/version/ledger checks, corrupt/foreign DB preservation, read-only/invalid path failures, migration rollback, successful upgrade snapshot including WAL changes, idempotency and Unix permissions/symlinks.
- Tauri command tests dispatch through registered handlers and the real worker/repository/SQLite using Tauri MockRuntime only for the window/runtime. They prove serialization, handler registration, async worker behavior, failure/retry and persistence across independent app instances. They are not actual WKWebView automation.
- Vitest/Testing Library uses injected test clients to isolate renderer behavior: loading/errors/retry, Turkish navigation, future-module disclosure, theme selection/save/failure/system-device changes, keyboard focus, error boundary, Zod response/input validation, command names/arguments, recipe decimal/contract validation. These mocks do not prove real persistence.
- Playwright runs actual browser frontend without mocking IPC. It verifies the explicit desktop-required error, Turkish document/title, keyboard skip link, navigation and narrow-sidebar disclosure/no overflow. A browser cannot access Rust SQLite; this is not native E2E.
- Native manual verification: packaged app (bundled frontend at `tauri://localhost`, with Vite stopped), Settings live storage diagnostics, light then dark selection, save acknowledgement, full Cmd-Q, relaunch, dark still selected. This proves real UI→IPC→SQLite→restart behavior on local macOS arm64.

## Native automation/platform feasibility

For M1 use real manual native verification plus real command-dispatch tests. Official Tauri docs describe WebdriverIO's embedded cross-platform service; plain native tauri-driver does not drive macOS WKWebView. Adding a separate driver/plugin now would add production/test integration beyond the minimal foundation; evaluate it before broad native automation, retaining Playwright for applicable UI coverage. [Tauri testing documentation](https://v2.tauri.app/develop/tests/webdriver/).

macOS Command Line Tools supported this native build. Windows needs native C++ Build Tools, MSVC Rust and WebView2; Intel targets require target/SDK/runner validation. This session has no Windows or Intel test host/installer evidence. Signing, notarization, CI release automation and offline WebView2 installer provisioning remain future gates. An unsigned debug .app is an internal verification artifact, not a trusted public release.

## Future test additions

Each M2–M11 gate in IMPLEMENTATION_PLAN.md requires targeted real-storage tests and native checks. Add historical-schema fixtures before any new production migration, catalog idempotency/user-override tests before seeds, and failure-injected complete portable restore tests before M9 acceptance. Never mark an untested platform or mock-only feature passed.

## M2A verification — 2026-10-08

Final automated evidence: 21 Rust tests (including the retained M1 regression suite), 18 Vitest renderer/contract tests, and 2 Playwright browser tests passed. TypeScript checks, ESLint, cargo check, cargo fmt --check, Clippy --all-targets -D warnings and frontend production build passed. Native unsigned debug .app packaging passed with bundled frontend and locked/offline Rust dependencies.

New real-storage tests cover create/retrieve/edit; child removal/replacement and stable order/IDs; exact decimal precision/trailing zeros/unit retention and invalid values at Rust/SQL boundaries; ingredient independence, Turkish İ/i and I/ı search, NFC duplicates and injection-like names; stale revisions; invalid references/duplicate child-ID rollback; FK deletion restrictions; soft deletion/restore after reopening; M1→M2 upgrade with preference preservation, both checksums, snapshot and repeat initialization. Real registered Tauri recipe commands also round-trip SQLite across independent worker/app instances. MockRuntime hosts only those test windows; it does not substitute the repository/database.

Renderer-only tests cover ingredient inline creation/select, decimal comma, step reordering, create/detail/edit, confirmed deletion/cancel/restore, save failure retaining input, discard confirmation, and unfinished editor input across Settings navigation/storage retry. Playwright continues checking the truthful browser desktop-required boundary and responsive keyboard-accessible shell; it does not claim recipe SQLite E2E.

Native GUI verification ran the packaged app at tauri://localhost (no Vite server) under a config-only temporary identifier `com.recipeatlas.m2a-verification`. Its user-entered test records are separate from `com.recipeatlas.desktop`. Observed food creation with Turkish description/title, custom ingredient creation, precise 35.000001 cc with note, two steps reordered before save, full Cmd-Q/relaunch retaining all data, edit/title update and reuse of existing ingredient, confirmed soft deletion, trash detail and restore, another restart retaining edit/restoration, light/dark theme switch and dark checked after relaunch, real schema 2/SQLite 3.53.2/FK/FTS diagnostics. Final-code bundle was relaunched and used to remove a line/edit a step/save, and to create a simple beverage without advanced fields.

Playwright initially failed to bind its local port under the sandbox (EPERM), then passed on an approved retry. Earlier tests exposed a one-migration fixture assumption and Clippy type-complexity warning; both were fixed and checks rerun. A first offline dependency fetch lacked Unicode normalization; approved development-time fetch completed. None were timeouts or unresolved feature failures.

Windows native behavior/installers, Intel builds, signed/notarized release packaging and automated native-driver tests remain unverified. M2A was validated with actual macOS GUI interactions and real-storage tests, not a mock-only persistence claim. Full draft restart recovery, duplication/archive, catalog import, advanced conversions and FTS recipe indexes remain future gates. Migration snapshots are not complete portable backups.

Normal-identifier final bundle also launched against the actual M1 app-data directory: schema upgraded to 2, existing dark theme preserved, FK/FTS diagnostics verified, and recipe library empty. No test records were inserted there. Reusing one build path with different bundle identifiers confused the CUA launch lookup; fresh copied .app paths resolved it. The repository's final build uses the normal identifier and unchanged packaging configuration.
