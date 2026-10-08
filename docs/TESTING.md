# Foundation testing

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
- Vitest/Testing Library uses injected test clients to isolate renderer behavior: loading/errors/retry, Turkish navigation, future-module disclosure, theme selection/save/failure/system-device changes, keyboard focus, error boundary, Zod response/input validation, command names/arguments, future recipe contract validation. These mocks do not prove real persistence.
- Playwright runs actual browser frontend without mocking IPC. It verifies the explicit desktop-required error, Turkish document/title, keyboard skip link, navigation and narrow-sidebar disclosure/no overflow. A browser cannot access Rust SQLite; this is not native E2E.
- Native manual verification: packaged app (bundled frontend at `tauri://localhost`, with Vite stopped), Settings live storage diagnostics, light then dark selection, save acknowledgement, full Cmd-Q, relaunch, dark still selected. This proves real UI→IPC→SQLite→restart behavior on local macOS arm64.

## Native automation/platform feasibility

For M1 use real manual native verification plus real command-dispatch tests. Official Tauri docs describe WebdriverIO's embedded cross-platform service; plain native tauri-driver does not drive macOS WKWebView. Adding a separate driver/plugin now would add production/test integration beyond the minimal foundation; evaluate it before broad native automation, retaining Playwright for applicable UI coverage. [Tauri testing documentation](https://v2.tauri.app/develop/tests/webdriver/).

macOS Command Line Tools supported this native build. Windows needs native C++ Build Tools, MSVC Rust and WebView2; Intel targets require target/SDK/runner validation. This session has no Windows or Intel test host/installer evidence. Signing, notarization, CI release automation and offline WebView2 installer provisioning remain future gates. An unsigned debug .app is an internal verification artifact, not a trusted public release.

## Future test additions

Each M2–M11 gate in IMPLEMENTATION_PLAN.md requires targeted real-storage tests and native checks. Add historical-schema fixtures before any new production migration, catalog idempotency/user-override tests before seeds, and failure-injected complete portable restore tests before M9 acceptance. Never mark an untested platform or mock-only feature passed.
