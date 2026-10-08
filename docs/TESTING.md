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

## M2B verification — 2026-10-08

Final checks: 31 Rust tests (21 retained M1/M2A tests plus 10 reliability/IPC cases, including the subprocess helper), 28 Vitest tests across 8 files, and 2 Playwright browser tests passed. TypeScript, lint with no warnings, cargo fmt --check, cargo check, Clippy --all-targets -D warnings and production frontend build passed. Both isolated and normal-identifier unsigned debug macOS app bundles built with locked/offline dependencies. No dependencies were added in M2B.

Actual SQLite tests cover raw incomplete new/edit drafts and reopen recovery, canonical record independence, CAS/tombstone rejection, atomic commit cleanup, discard, duplication/new child identities/exact units/order, archive/unarchive/trash/restore/purge, shared ingredient survival, orphan-draft save-as-new, reference-safe rename/delete, duplicate/stale ingredient conflicts, competing database sessions and failed aggregate rollback preserving drafts. A historical schema-2 fixture upgrades through 003 with IDs/quantities/steps/preferences preserved and all migration checksums validated. Registered reliability IPC handlers use the real worker/SQLite for search, draft operations, commit, duplicate, archive, purge and ingredient reference/conflict responses.

Unexpected termination is tested by spawning this Rust test executable as a separate process: it opens a real temporary SQLite connection, acknowledges an incomplete draft, signals readiness and stays alive. The parent terminates only that child, reopens the database and verifies the raw quantity/blank step and absence of any committed recipe. This establishes recovery of acknowledged storage, not survival of keystrokes still inside the debounce window. The helper test returns normally without its private environment marker when the ordinary test suite runs.

Search tests create actual Şeker, close/reopen SQLite and verify Şek/şek/ŞEK/şeker, exact duplicate reuse, actual catalog count versus no matches, and İÇ/iç/İç → İçme suyu and IHL/ıhl/Ihl → Ihlamur. Dotted `ihl` does not match dotless Ihlamur. The prior normalized matching logic already passes; no genuine M2A search defect reproduced for a saved record. New UI distinguishes an empty personal catalog, no result and storage failure. No global catalog is seeded.

Renderer-only tests cover serialized in-flight autosave/commit/discard, write failure retry, stale-session explicit fork, recovery of blank steps/partial decimal values, leave-with-draft versus confirmed discard, scoped archive/duplicate/purge interactions, Turkish autocomplete keyboard selection, mouse focus return/dropdown closure, create missing ingredient and distinct empty/failure states. Injected clients are only interaction fixtures; persistence evidence comes from Rust/registered IPC/native checks. Playwright verifies the desktop-required boundary and responsive navigation, not SQLite recipe E2E.

Native packaged macOS arm64 verification used `com.recipeatlas.m2b-verification`, separate from personal production data. Observed: empty personal catalog; actual creation of Şeker; all four requested searches return it; keyboard ingredient selection; new draft with `35.` and a blank step; full Cmd-Q/relaunch; recovery while recipe library remains empty; complete to `35.000001` and Save; existing-recipe title edit/quit/relaunch with original title still committed and separate edit draft; resumed explicit Save; duplicate/edit independently; original unchanged; archive/separate archive view/unarchive; confirmed soft delete/trash; permanent-delete warning opened then canceled; restore; shared ingredient renamed to Toz Şeker with notes; saved recipe reads renamed ingredient and unchanged exact quantity. A rebuilt verification app reopened the persisted recipes and successfully created İçme suyu, matched `iç`, and created Ihlamur. Permanent purge/discard execution is proven by actual SQLite and command tests; their destructive native confirmation was not executed.

Later macOS ScreenCaptureKit returned error -3811, preventing further AX/GUI observations even after a computer-use session reset. Thus the subsequent native `ıhl` result, final small focus/icon/exit-listener-error/localization refinements, window-close-button flush, native save-failure feedback and normal production-identifier launch were not re-observed. These limits are not reported as product failures or passed GUI checks. Cmd-Q flush/restart was observed, and all requested Turkish variants are verified against actual saved SQLite records, including registered IPC. The final normal-identifier bundle built successfully; normal personal data received no verification recipes.

Required development launch initially failed because port 1420 was already occupied (confirmed error, not timeout). A config-only retry used port 1430 with the isolated identifier, compiled, reached `Running target/debug/personal-recipe-library`, and stayed running until the smoke-test session was stopped cleanly with Ctrl-C. GUI inspection of this dev launch was unavailable due to the capture error. Existing port-1420 process/configuration was left intact. Playwright required approved local port binding outside the sandbox. No unresolved command timeout remains.

Windows/Intel native hosts, installers, signing/notarization and native-driver automation remain unverified. Recipe lists are unpaginated; fuzzy search, catalog, conversions, media/history and portable backup remain their later milestones. Browser/injected-client tests never substitute for native storage claims.
