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


## M3A final resumed verification — 2026-10-09

Final automated checks passed: TypeScript typecheck, ESLint without warnings, 32 Vitest tests/10 files, 2 Playwright browser tests, production frontend build, cargo fmt --check, locked/offline cargo check, **43 Rust tests**, Clippy all targets with warnings denied. Rust count includes two subprocess helper entry points (draft/catalog) that return normally without private markers; their parent tests execute the actual child processes. Retains all M1/M2A/M2B regression coverage. No dependency/lockfile changes.

Catalog tests exercise real SQLite clean install and counts, unchanged re-import, restart, stable IDs, Turkish normalized variants and decomposed Unicode, English aliases with English I/i, one result per identity, source license/provenance, no guessed observations, preferred dimensions/units, personal exact-name collision without duplicate publication, existing recipe/draft references and quantities, explicit revision-checked metadata linking, stale customization rejection, override preservation across upgrades, downgrade/changed-version rejection, invalid/duplicate records, category hierarchy/FKs/cycles, migration 3→4 preserving trash/archive/drafts, source-license and file checksum enforcement, explicit validation opt-in and all-or-nothing rollback through an injected SQL trigger failure. Additional test rejects a changed system display name that collides with personal data, rolling back the update. Registered Tauri command tests use the real worker and previously imported SQLite data for status/alias/customization/category operations; mocked Tauri window plumbing does not mock the repository.

Unexpected importer termination: a separate test process runs the real importer, writes catalog data and success audit inside its uncommitted transaction, signals readiness via a cfg(test)-only hook, and is killed by the parent. Reopening yields zero catalog/release rows and the earlier durable running audit; retry imports all eight. This establishes pre-commit rollback, not a guarantee against hardware/storage corruption. Validation errors before database opening do not create an audit or a database. Source snapshots and existing source identity mappings remain immutable; missing records are retained rather than deleting referenced identities.

Real CLI clean import report is committed as catalog/validation/coverage.json: 8 definitions, 8 category nodes, 8 available identities, zero production definitions/collisions. Repeated import reports unchanged=true with no new records. Regeneration from the pinned official USDA archive produced identical hashes for all four generated files. Native collision library instead reports 7 available canonical identities and one personal collision. Category counts are overlapping direct memberships, not a sum of unique ingredients.

Native macOS arm64: isolated unsigned debug .app built and launched at tauri://localhost. Personal Şeker from the interrupted session remained. Saved an actual recipe with 35.000001 g and a step, quit, installed validation package into the isolated app-data directory through the real CLI, reopened and verified saved recipe unchanged. Settings displayed schema 4, SQLite 3.53.2, FK/FTS5 verified, validation 8/production 0/pending 1. ICING returned one Pudra şekeri result and keyboard selection worked. Catalog-backed unfinished draft recovered raw 12. after observed entry/Cmd-Q/relaunch. Personal manager remained personal-only; dark theme preference survived restart. A first batched type/quit did not establish AX-visible input and recovered no quantity; the observed-input repeat passed. Do not treat unobserved keystrokes as acknowledged SQLite state.

Final normal-identifier unsigned debug macOS app also built; normal personal data received no validation seed/test recipes. Last importer name-conflict error path has real SQLite regression/final-build evidence, not manually observed native error-dialog evidence. Tauri dev with config-only isolated identifier/port 1430 compiled and reached Running target/debug/personal-recipe-library, then stopped cleanly. GUI bundle lookup selected the packaged app, so dev launch is CLI smoke evidence only. No ScreenCaptureKit error blocked this resumed packaged verification.

Browser tests prove desktop-required boundary/responsive navigation, not native persistence. Frontend catalog response/error/status fixtures prove presentation/protocol handling only. First resumed Playwright bind failed with sandbox EPERM at 127.0.0.1:1420; authorized retry passed. Initial schema-expected-version/historical fixture issues were fixed and final Rust tests rerun. No unresolved failure/timeout. Windows/Intel hosts, signed/notarized installers, comprehensive production catalog and catalog-scale/fuzzy/category-management UI remain unverified/future scope.

Final small personal-manager feedback explains when creating a name reuses a catalog record rather than making a personal duplicate; type/lint/frontend tests/build pass. That message was not separately clicked in native verification.

## M3B-1 production catalog verification — 2026-10-09

Added five real SQLite tests: embedded/disk package equivalence; full production
counts/source/category report consistency and TR/EN/alias search; recipe exact
quantity persistence after database reopen; version-1 personal collisions,
customizations/notes/units, recipe/draft preservation; invalid duplicate and
mid-transaction trigger rollback/retry; actual default worker first-launch
installation/reopen/no repeated audit; future installed version retention and
same-version drift rejection. The schema-3 upgrade regression now also installs
the real production seed after migration/validation, retaining M2B state.
Historical IPC fixtures explicitly use a seed-free worker for their original
contracts; the new production worker test uses the real default path, not mocks.
Existing crash/interrupted-import and M1/M2A/M2B/M3A tests remain active.

Package: 235 verified source IDs, 39 category nodes, 470 production provenance
mappings, zero empirical observations; clean installation available=235,
collisions=0. Upgrade fixture with personal Şeker: production definitions=235,
available catalog definitions=234, one pending collision; personal identity and
recipe/draft references unchanged. Source history retains four snapshots on
validation→production upgrade. Hash/quantity/FK/integrity assertions are real SQL.

Generation was rerun against the pinned archive and all JSON artifacts reproduced
byte-for-byte. The CLI installed into /private/tmp/recipeatlas-m3b1-final-cli;
actual successful report is catalog/production/installation-report.json. A
second import returned unchanged=true, inserted=0, updated=0. No user database
was erased or reset. Unselected upstream rows are not reported as rejected.

Frontend fixtures verify presentation only. Initial Clippy caught two needless
borrows from the shared-loader refactor; fixed. A frontend test temporarily
failed because the updated explanatory copy removed its validation/production
distinction phrase; the phrase was retained and tests rerun. Neither was a
timeout. Final check results are recorded in PROJECT_PROGRESS.md.

Native verification uses isolated identifiers and copied unsigned debug .apps,
never the normal com.recipeatlas.desktop database. Packaged first launch (no
Vite/API server) showed schema 4, FK/FTS5 verified, production 235/validation 0,
collisions 0. Selected bundled Domates (çiğ, kırmızı) with Down/Enter, saved a
recipe with 200.000001 g and “Domatesi doğrayın.”, explicitly Quit via the macOS
menu, confirmed stopped app inventory, relaunched and observed exact data.
No physical OS network disable was performed; artifacts are embedded, Rust has
no import network client, and the packaged workflows require no online API.
Windows, Intel macOS, signing/notarization and independent Turkish culinary
expert review remain unverified. No installer/public trust claim is made.


Final isolated build `com.recipeatlas.m3b1-final-verification` also showed 235/0/0
production/validation/collision counts and revised M3B-1 Settings copy. Native
ICING returned Pudra şekeri; keyboard selection and save of 35.000001 g plus
“Pudra şekerini eleyin.” survived explicit menu Quit, confirmed stopped inventory,
and relaunch. After these observations the last recipe-help wording change was
covered by frontend checks/build; its exact wording was not manually reobserved.
The final normal-identifier unsigned debug bundle was built without launching
against the user's production database. Isolated development launch on port
1430 reached Running target/debug/personal-recipe-library and was stopped with
Ctrl-C. GUI workflow evidence is from packaged apps, not inferred from dev logs.
A final coverage consistency assertion initially used incompatible usize/i64 test
types; corrected with checked conversion, then full Rust tests/Clippy rerun.

Final browser rerun first encountered sandbox EPERM binding localhost:1420;
authorized local-port retry passed both tests. This is an environment restriction,
not a product failure or timeout. No unresolved timeouts or failed final gates.

## M3B-2 verification — 2026-10-09

Production version 3: 482 identities, 247 additions, 396 food / 86 beverage,
252 aliases, 50 categories. Final data removes the broad almond-butter synonym
instead of inventing an awkward replacement. No migration/dependency changes.

51 Rust tests pass, retaining M1/M2A/M2B/M3A/M3B-1 coverage and adding three
expansion tests: actual archived v2 upgrade (all 235 IDs, personal recipe/draft,
per-field override and pending collision intact), real failed upgrade rollback/
reopen/retry, and source boundaries/forms/Turkish aliases/search performance.
The subprocess interruption test remains real SQLite crash coverage. Two Python
audit tests run against isolated copies of actual v2/v3 artifacts, with five
injected defect subcases (identity/Unicode label/alias/source/unit). All-record
audit passes with no structural errors or exact alias collisions; 17 human
terminology questions remain, no independent human review claimed.

32 Vitest tests and 2 Playwright browser tests pass. TypeScript, ESLint, frontend
production build, Rust fmt/check, Clippy all-targets with warnings denied pass.
Browser tests are not proof of native SQLite. No check was treated as passed
merely because of mocks. A first test compile exposed assumptions about the
minimal Ingredient DTO; fixed by using stored revision/query. Upgrade assertion
was corrected to count inserted definitions including one pending collision,
not materialized rows. Both were resolved and tests rerun; no timeouts.

Clean CLI install: 482 inserted/available, zero collisions. Actual archived v2
upgrade: 235 updated + 247 inserted, 482 available, zero collisions. Artifacts:
installation-report.json / upgrade-report.json. No personal DB reset/opened for
these tests. Measured in focused concurrent test run: v2 upgrade 675 ms, 100 real
Turkish partial SQLite searches 287 ms (~2.9 ms/query). These are local debug
observations, not a cross-hardware guarantee; gross search regression test uses
a generous 10-second bound. Re-import skips unchanged packages/startup audits.

Native unsigned debug .app was built with locked/offline dependencies under
isolated identifier com.recipeatlas.m3b2-verification. Packaged tauri://localhost
(with no Vite dependency) showed schema 4, SQLite 3.53.2, FK/FTS verified and
482 production / 0 validation / 0 pending records. Turkish SİYEZ and English alias
London dry gin resolved real bundled entries. Recipe “M3B-2 Siyez Denemesi” saved
with Siyez bulguru 35.000001 g, London gin 25 mL and a step. Menu Quit, process
absence in inventory, relaunch and detail inspection confirmed exact persistence.
This profile is separate from the user's database. Initial automation getApp call
took 341 seconds; subsequent launch 1.2 seconds. That is tool-call latency, not a
confirmed app initialization timeout/failure. Catalog initialization completed.

Windows/Intel native execution, signed/notarized releases and hardware network
disabling remain unverified. Normal recipe workflows contain no runtime API call;
embedded package installs without a development server or external seed file.
Final package verification and dev-launch details are appended in PROJECT_PROGRESS.

Final reviewed package (`com.recipeatlas.m3b2-final-verification`) additionally
confirmed updated M3B-2 copy, schema/FK/FTS and 482/0/0 counts, Turkish SİYEZ search,
keyboard selection, actual recipe save and confirmed full quit/relaunch retaining
Siyez bulguru 35.000001 g. Final first getApp automation call took 888 seconds;
relaunch took 0.7 seconds. Cause of first-launch tool/OS latency remains unknown;
subsequent app flows and SQLite checks succeeded. No false timeout claim.
Isolated development launch on port1433 compiled and ran the native binary and
was intentionally stopped afterward. Deterministic reproduction left all five
artifact hashes unchanged; all eight archived v2 files match the M3B-1 commit.


## M3B-3 final verification — 2026-10-09

- 53 Rust tests: M1/M2A/M2B/M3A regressions, actual archived v2/v3→v4 preservation, new search/aliases, zero-write repeat bootstrap, shared referential integrity and transaction rollback, actual validation and full production killed-import recovery.
- 32 Vitest tests; 2 Playwright browser tests. Browser tests remain presentation evidence only. Initial sandbox Vite listen failed EPERM; rerun with local-server permission passed.
- 4 Python audit tests (13 semantic defect subcases plus separate checksum tampering), all-record audit --check; UTF-8 CSV round trip verifies 18 unique review rows and zero human approvals.
- TypeScript/ESLint, Rustfmt/check/Clippy all-targets with denied warnings and frontend production build pass. Initial updated Rust test expectations/type assertions and CSV exporter shape needed correction; final runs pass, not ignored failures or timeouts.
- Real CLI offline verification: 8 process runs, network denied via sandbox-exec; clean 508 / 0 collisions; archived v2→v4 273 new / 235 updated; archived v3→v4 26 new / 482 updated; all repeats unchanged. Temporary databases only. Exact reports/timings under catalog/production/m3b3-installation-report.json.
- Unsigned debug macOS arm64 app bundle builds. Isolated identifier com.recipeatlas.m3b3-verification, packaged tauri://localhost (no Vite server), shows schema 4/FK/FTS verified and 508 production / 0 validation / 0 pending. Native recipe editor searches `TOZ ŞEK`, `Domates (çiğ`, `Limon (`, `Brewed coffee`, `All-purpose wheat flour`, `YOĞURT`, `Orgeat`, `IPA`, `SİYEZ`, `RAKI` succeeded. Keyboard selection saved Toz şeker 35.000001 g and Orgeat 25.000001 mL. Menu Quit confirmed process absence via pgrep; relaunch/detail retained both exact quantities and references. Native GUI connectivity was not disabled; separate real importer processes had network denied.

Local focused measurements: CLI clean initialization/import 238.593 ms; current-version repeat open 80.939 ms; v2 upgrade 232.644 ms; v3 upgrade 230.483 ms (process wall time). Rust focused imports 190.415 / 186.139 ms; 100 actual Turkish partial queries 80.595 ms. Debug/local observations, not portable SLA or release-startup benchmark.

Windows / Intel macOS runtime/installers, signing/notarization remain pending. Initial native tooling reused a stale bundle identifier for the repository app path; copying to a unique temporary .app path resolved discovery. No user database reset/opened; native test profile retained. No GUI redesign or M3C implementation.

```sh
npm run typecheck
npm run lint
npm test
npm run build
npm run test:ui
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --locked --offline --manifest-path src-tauri/Cargo.toml
cargo check --locked --offline --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --offline --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npm run tauri -- build --debug --bundles app --config '{"identifier":"com.recipeatlas.m3b3-verification"}' -- --locked --offline
```

Normal development launch: npm run tauri -- dev. Use an isolated identifier/config as documented for clean installation tests; never reset the normal profile.

Final rebuilt package at `/private/tmp/recipeatlas-m3b3-final.app` additionally showed the updated Aşama 3B-3 label and retained the same saved recipe/quantities after initialization. Its identifier matches the isolated verification profile; the normal user profile was untouched.

## M3C-1 — unified library verification (2026-10-09)

Passed: typecheck, ESLint, 38 Vitest tests across12 files, frontend production build; Cargo fmt/check, 58 Rust tests and all-target Clippy with warnings denied (locked/offline); two Chromium browser regressions; four Python catalog audit tests plus the508-record audit (no structural errors/exact alias collisions,18 terminology items remain for human review). Renderer tests isolate presentation/IPC validation; they do not establish native persistence.

Four new SQLite repository tests cover all508 distinct paginated canonical records, personal/global counts, Turkish casing/NFC and English aliases, descendant/combined filtering, duplicate-result prevention, invalid queries, real details/provenance, personal CAS/reference safety, exact recipe quantities, draft and restart preservation, pending collisions and overrides. A new registered-command test dispatches typed library/detail IPC through a real worker/database. Existing regressions include transactions, migration compatibility, interrupted imports and draft recovery. Seventeen real30-row pages (508 IDs) measured244.900ms total in a focused warmed test; this is one local sample including facets/memberships, not a cross-platform SLA.

Unsigned arm64 macOS debug package built twice, final bundle36.05MiB, with isolated identifier `com.recipeatlas.m3c1-verification`. Normal user profile was not opened or reset. Native verification browsed17 pages (16×30+28), saw508 catalog/0 personal initially, created and renamed a personal record, then saw508/1/509. Catalog sugar details exposed169655/CC0 provenance and unknown factual metadata with no edit action. Native searches Şek/şek/ŞEK/şeker each returned15 matches including Toz şeker; Granulated sugar and Orgeat each1; RAKI/rakı each1; SİYEZ/siyez each3. Nested Peynirler showed30, combined Feta search1. Both themes visually checked. Recipe selection by keyboard saved Orgeat25.000001mL and personal35.000001g; editor preserved both. Cmd-Q flushed an edit draft; process absence was confirmed with pgrep; final rebuilt app reopened the same isolated profile, recovered the draft and both quantities, committed the edit and retained library totals.

Initial native automation getApp took391.24s before returning a window; subsequent restart took1.73s. First tool duration is not a measured SQLite initialization time. Packaged app uses tauri://localhost with embedded offline seed, without a development server. No physical network-disconnection experiment was performed; offline behavior is additionally verified by embedded-package SQLite tests and no network calls in library services. Windows, Intel macOS, signed/notarized packages and installer testing remain unverified. Catalog terminology review is unchanged and is not human-approved by these checks.

Reproduce normal development with `npm run tauri -- dev`; isolated package:

```sh
npm run tauri -- build --debug --bundles app --config '{"identifier":"com.recipeatlas.m3c1-verification"}' -- --locked --offline
cargo test --locked --offline --manifest-path src-tauri/Cargo.toml ingredient_library
python3 -m unittest discover -s tools -p 'test_*.py'
python3 tools/audit_catalog.py
```

Isolated `npm run tauri -- dev` with identifier `com.recipeatlas.m3c1-dev-verification`, Vite port1434 and locked/offline Cargo compiled and reached `Running target/debug/personal-recipe-library`; stopped cleanly with Ctrl-C. This is CLI launch evidence; GUI acceptance above is from the packaged app.

## M3C-2 — ingredient personalization and selection (2026-10-09)

Passed on macOS arm64: `npm run typecheck`, `npm run lint`, `npm test` (42 tests/13 files), `npm run build`, `npm run test:ui` (2 Chromium tests), `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `cargo test --locked --offline --manifest-path src-tauri/Cargo.toml` (60 Rust tests), `cargo check --locked --offline --manifest-path src-tauri/Cargo.toml`, `cargo clippy --locked --offline --all-targets --manifest-path src-tauri/Cargo.toml -- -D warnings`, four Python catalog audit tests, and `python3 tools/audit_catalog.py --check` (508 identities; 18 terminology items awaiting human review). Debug arm64 `.app` bundle built with `npm run tauri -- build --debug --bundles app --config '{"identifier":"com.recipeatlas.m3c2-verification"}' -- --locked --offline` (36.33 MiB). Browser tests exercise only the browser shell, not SQLite; renderer tests use fixtures only for interaction behavior.

New real SQLite tests upgrade an actual schema-4 fixture through checksummed migration 005, preserve a recipe and exact `0.123456` quantity, verify bounded deterministic suggestions/exact alias priority, duplicate-result prevention, favorite toggling for canonical/personal IDs, recent saved-recipe lookup, presentation update/reset, personal alias search, rejected alias collision without partial write, full database reopen, and unchanged canonical name/recipe quantity. Existing import/seed idempotency, killed-process rollback, draft, catalog-override and recipe regression tests all pass. The 508-ID package and its 260 bundled aliases/50 categories are unchanged. Local isolated-profile `PRAGMA integrity_check` returned `ok`, `foreign_key_check` returned no rows, and schema version was 5; one canonical and one personal favorite were present after restart.

Native GUI verification used only `com.recipeatlas.m3c2-verification`, separate from the normal application profile. Fresh start showed 508 built-in/0 personal; created one personal ingredient, favorited it and a canonical item, then saved a recipe with that personal ingredient `35.000001 g` and built-in sugar `25.000001 g` selected by keyboard from Turkish/English searches. The recent view showed those two saved-recipe ingredients. Personal rename plus sidebar Save-and-Continue was verified; Continue Editing retained text. Canonical sugar's local display name and search alias were changed through its presentation editor, detail still showed original `Toz şeker`, and reset restored the original display and removed the personal alias. A full Cmd-Q, confirmed process absence, and launch of the final rebuilt package retained 508/1 counts, both favorites, recent usage, recipe identity and exact quantities. Native duplicate search for `Şeker` placed canonical Toz şeker first and offered its detail instead of creating a second record. Unsaved new personal creation prompted Save/Discard/Continue on sidebar navigation; Continue preserved the typed text. No actual user database was reset or used for test records.

The final rebuilt package shows the M3C-2 label and corrected empty-recent messaging. The native test profile retains disposable verification records outside Git. Native network-disconnection was not separately simulated; the embedded package loaded without a Vite server and all normal ingredient operations use SQLite, while existing network-denied importer tests cover offline seed installation. Windows, Intel macOS, signed/notarized release and installer behavior remain unverified. The 18 terminology items have not received independent human approval. Future seed updates must continue to preserve override markers/personal aliases; current regression tests cover the existing update strategy and migration compatibility.

## M3C-3 — collision/category quality verification (2026-10-12)

Passed on macOS arm64: `npm run typecheck`, `npm run lint`, `npm test` (50 tests/15 files), `npm run build`, `npm run test:ui` (2 Chromium shell tests), `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `cargo test --locked --offline --manifest-path src-tauri/Cargo.toml` (63 Rust tests), `cargo check --locked --offline --manifest-path src-tauri/Cargo.toml`, all-target Clippy with denied warnings, four Python audit tests, `tools/audit_catalog.py --check`, `tools/report_m3c3_quality.py --check`, and `git diff --check`. Renderer/browser tests cover controls, not native persistence. Rust tests use isolated real SQLite databases and check category edit rollback, revision/reference safety, unique descendant counts, uncategorized filtering, Turkish NFC/İ/i/I/ı, alias and qualified-form candidates, unchanged catalog counts, migration and existing recipe/draft/favorite/override regressions. Migrations 001–005 and package 4 were not changed.

An unsigned debug Apple Silicon `.app` built with `npm run tauri -- build --debug --bundles app --config '{"identifier":"com.recipeatlas.m3c3-verification"}' -- --locked --offline`. Native startup used only that isolated bundle identifier. Fresh app data showed 508 built-in, 0 personal, 260 aliases and 50 categories. In the real Tauri UI, all 50 hierarchy nodes appeared; a personal ingredient was created, assigned to Otlar, and found under the Gıda parent with unique count 1. The Kategorisiz view showed zero after assignment. Entering `Şeker` displayed canonical Toz şeker as an alias candidate, weaker qualified-name candidates separately, and blocked a new record with that strong match. Save/Discard/Continue modal appeared when leaving an unfinished creation. Cmd-Q and relaunch of the final rebuilt bundle retained 508 built-in, one isolated personal record and the Otlar assignment; the final shell showed Aşama 3C-3. The isolated database reported schema 5, `PRAGMA integrity_check=ok`, no foreign-key violations, 260 aliases, 50 categories and one personal category membership. One successful catalog import audit and one installed release remained after relaunch, confirming no repeat import. The normal personal profile was not opened or reset. The initial sandboxed `open -n` returned `kLSNoExecutableErr`; an approved unsandboxed launch succeeded with the same built bundle, so the first result was a LaunchServices sandbox limitation rather than a package failure.

The static package audit recorded zero structural errors/exact alias collisions, 25 shared-base-name form comparison groups and 18 terminology decisions awaiting human review. The workbook has filterable columns, frozen header, source links and blank human-decision fields; its source-driven builder uses the bundled spreadsheet runtime. Automated checks do not approve language choices or prove semantic identity equivalence. Windows/Intel macOS runtime, installers and signed/notarized builds remain unverified. Native M3C-3 GUI testing covered collision/category flows; recipe quantity/draft/favorites/override behavior remains supported by the full real-SQLite regression suite and prior isolated native M3C-2 session, not a fresh manual recipe run in this increment.

### Resumed M3C-2 final checks (2026-10-12)

The WIP commit `075c1c9` was preserved. Added focused modal tests for safe initial focus, restoration on Cancel and native `cancel`/Escape event, prevention of Escape/destructive actions while saving, in-modal error feedback, personal Save/Discard/Continue decisions and catalog presentation failed-save retry. Final suite: 47 Vitest tests across 15 files, 60 Rust tests (real SQLite/import/restart/migration coverage), 2 Chromium browser shell tests, 4 Python audit tests, and the 508-identity catalog audit. TypeScript, ESLint, `cargo fmt --check`, `cargo check --locked --offline`, all-target Clippy with `-D warnings`, frontend build, native unsigned arm64 app build and `git diff --check` passed. The focused tests do not substitute for native SQLite evidence; the unchanged WIP persistence suite and prior isolated GUI session provide that evidence.

The final code was bundled with identifier `com.recipeatlas.m3c2-final-verification` (36.33 MiB) and launched with `open -n` outside the command sandbox after a sandboxed LaunchServices attempt returned `kLSNoExecutableErr` despite a present arm64 binary. This was a sandbox launch limitation, not a confirmed package failure. Its fresh isolated SQLite profile reported schema5, `integrity_check=ok`, no foreign-key violations, 508 canonical identities, 260 bundled aliases, 50 categories, 0 personal ingredients and one production release. Relaunch preserved those counts and did not add another import run. The first launch briefly produced two test-app processes and two successful import audits of the same release; no catalog identities were duplicated. The test processes were closed. Separate isolated `tauri dev` at Vite port1442 compiled and reached `Running target/debug/personal-recipe-library`, then stopped cleanly.

The resumed session had no native GUI automation tool, so the final focus-restoration change was not manually exercised in the packaged GUI. Its initial focus, Cancel/Escape restoration, in-flight lockout and failed-save retention were exercised in frontend tests. The previous WIP native GUI observations (favorites/recent/override/recipe/restart) remain valid because this follow-up changed only dialog presentation and error/busy wiring. No normal user database or profile was opened or reset. Windows, Intel macOS, installers and signing/notarization remain unverified.

## M3C-4 terminology-only catalog release — 2026-10-13

- The real `catalog_import` CLI was built with locked offline Cargo dependencies and run **11 times** against fresh/v2/v3/v4 temporary profiles with process-local network denial. Fresh v5 inserted 508; v2→v5 inserted 273/updated 235; v3→v5 inserted 26/updated 482; v4→v5 inserted 0/updated exactly 3. Every repeat import was unchanged (0 inserted/updated). Raw measured results are in `catalog/production/m3c4-installation-report.json`. The first nested `sandbox-exec` attempt failed with exit 71; the same command passed under the approved unsandboxed execution path. No normal app profile was used.

- `cargo test --locked --offline --manifest-path src-tauri/Cargo.toml`: **66 passed**. New real-SQLite tests cover all three exact Turkish names and stable IDs, fresh embedded installation, archived-v4→v5 upgrade, repeat import/startup, English/Turkish/alias search, personal ingredient and explicit presentation override retention, saved recipe reference and exact `35.000001 cc` line retention, foreign keys/integrity, and transactional rollback when a new canonical name collides with an existing personal ingredient. Existing M1–M3C-3 tests passed.
- `python3 tools/test_catalog_audit.py`: **5 passed**; `tools/audit_catalog.py --check`, `tools/report_m3c3_quality.py --check`, and `tools/report_m3c4_quality.py --check` passed. The regenerated v5 report has 508 identities, 260 aliases, 50 categories, zero structural errors/exact alias collisions, 3 user-selected names, 15 pending items and 25 unchanged possible concept groups. The workbook has 18 unique IDs, 3 filled human-decision cells, 15 blank decision cells, filters and frozen header; the edited range was rendered and visually checked.
- `npm run typecheck`, `npm run lint`, `npm test` (**50 passed / 15 files**), `npm run test:ui` (**2 Chromium tests**), `cargo check --locked --offline`, `cargo fmt -- --check`, and all-target `cargo clippy -- -D warnings` passed. `npm run build` passed as the native build's `beforeBuildCommand`; the production Vite bundle was produced. Browser tests cover the renderer shell, not SQLite persistence.
- `npm run tauri -- build --debug --bundles app --config '{"identifier":"com.recipeatlas.m3c4-verification"}' -- --locked --offline` built an unsigned arm64 debug app (36.35 MiB). `open -n` launched it after the nested shell sandbox's LaunchServices error was bypassed through the normal escalation path. A final rebuild/relaunch from the commit-ready source left the successful import count at one. The *isolated* native profile `com.recipeatlas.m3c4-verification` contains schema 5/catalog release 5, 508 catalog identities, 260 aliases, 50 categories, 0 personal ingredients, all three approved names, `PRAGMA integrity_check=ok` and zero foreign-key violations. The normal user profile was neither opened nor modified. The computer-use inventory listed the test app as running but could not bind to its window, so a fresh native GUI search/detail interaction was **not** verified in this session; real SQLite-backed Rust tests cover those paths. Windows, Intel macOS, installer, signing and notarization checks remain pending.
- An uncustomized canonical-name update that exactly collides with an existing unlinked personal name aborts the entire catalog release and leaves the old catalog and personal recipe intact. This existing conservative policy needs explicit user collision review before that specific profile can complete the upgrade; no automatic merge or overwrite occurs.

## M3C-5 terminology release 6 — 2026-10-13

- `cargo test --locked --offline --manifest-path src-tauri/Cargo.toml`: **69 passed**. The added real-SQLite tests verify the exact two names/IDs, 10 retained names, 3 unresolved names, fresh installation, release-5 upgrade, idempotence, search, detail, recent/favorite and personal preservation, presentation override precedence, draft and saved recipe references, exact `35.000001 cc`, FK/integrity and full rollback on a colliding personal name. Historical v4→v5 tests now read immutable archived package 5. Existing M1–M3C-4 tests passed.
- `python3 tools/audit_catalog.py --check`, `report_m3c3_quality.py --check`, `report_m3c4_quality.py --check`, `report_m3c5_quality.py --check` and `python3 tools/test_catalog_audit.py` passed (**5 Python tests**). The current audit has 508 identities, 260 aliases, 50 categories, zero structural errors/exact alias collisions and 25 unresolved concept comparison groups. The 18-row workbook was regenerated, rendered and read back: 5 applied name decisions, 10 retained names, 3 unresolved; filterable table and frozen header. No independent expert linguistic review is claimed.
- `npm run typecheck`, `npm run lint`, `npm test` (**50 passed in 15 files**), `npm run test:ui` (**2 Chromium browser tests**), `npm run build`, `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `cargo check --locked --offline --manifest-path src-tauri/Cargo.toml` and all-target Clippy with warnings denied passed. Browser tests cover the renderer shell, not native SQLite.
- `cargo build --locked --offline --manifest-path src-tauri/Cargo.toml --example catalog_import` and `python3 tools/verify_catalog_installation.py --include-v5 --deny-network --output catalog/production/m3c5-installation-report.json` passed **14 isolated real CLI imports/opens** under process-local network denial: fresh release 6 inserted 508; v2→v6 inserted 273/updated 235; v3→v6 inserted 26/updated 482; v4→v6 updated 5; v5→v6 updated exactly 2. Repeats were unchanged. A nested `sandbox-exec` attempt exited 71; the same command passed outside the nested command sandbox. No normal user database was used.
- `npm run tauri -- build --debug --bundles app --config '{"identifier":"com.recipeatlas.m3c5-verification"}' -- --locked --offline` built an unsigned arm64 app (36.35 MiB). LaunchServices returned `kLSNoExecutableErr` inside the command sandbox; `open -n` outside it launched the isolated bundle. Its new `com.recipeatlas.m3c5-verification` profile has release 6/schema 5, 508 identities, 260 aliases, 50 categories, both names and stable IDs, `integrity_check=ok`, zero FK violations and one successful import. All test-app processes were then closed; a complete relaunch of the final bundle kept one successful import and all counts. The test process was closed again. The normal user profile was untouched. Computer-use inventory saw the running test app, but could not bind to its window; fresh GUI search/detail/recipe interaction was **not** exercised in this session. SQLite-backed Rust IPC/search/recipe tests provide functional evidence, but do not replace GUI verification. Windows, Intel macOS, installer, signing and notarization remain unverified.


## M3D-1 final verification — 2026-10-14

Passed on this macOS arm64 host: TypeScript, ESLint, 50 Vitest tests, 2 Chromium browser tests, 72 Rust tests, Cargo fmt/check/all-target Clippy with warnings denied, production frontend build, unsigned native debug app bundle, 5 existing Python audit tests plus 4 new release-generation tests, current/historical audit freshness and Git whitespace checks. Applied migrations 001–005 were untouched.

The first browser attempt failed to bind localhost with sandbox EPERM; the authorized run passed both tests. The default development port 1420 was already occupied; the existing server was left untouched. An isolated Tauri dev launch on port1438 reached Running target/debug/personal-recipe-library and was stopped with Ctrl-C. These were environment constraints, not passing tests or timeouts.

```sh
npm run typecheck
npm run lint
npm test
npm run test:ui
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --locked --offline --manifest-path src-tauri/Cargo.toml
cargo check --locked --offline --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --offline --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
python3 tools/test_catalog_audit.py
python3 tools/test_m3d1_catalog.py
python3 tools/audit_catalog.py --check
python3 tools/report_m3d1_quality.py --check
npm run tauri -- build --debug --bundles app --config '{"identifier":"com.recipeatlas.m3d1-verification"}' -- --locked --offline
# Normal local development:
npm run tauri -- dev
# Isolated smoke when the normal port is occupied:
npm run tauri -- dev --config '{"identifier":"com.recipeatlas.m3d1-dev-verification","build":{"beforeDevCommand":"npm run dev -- --port 1438","devUrl":"http://localhost:1438"}}' -- --locked --offline
```

Real SQLite coverage: archived release6→7 inserts60, updates0, retains508 complete source objects and50 category objects; clean7 inserts568 with269aliases. All60 additions were queried by both names and every alias, inspected through detail DTOs and combined category filters. Turkish dotted/dotless case and decomposed Ş search pass. Injected mid-expansion SQLite failure leaves release6/source/count state unchanged, retry succeeds, and unauthorized source ID fails before profile creation. Restart checks retain favorites, recent usage, custom alias/name/unit/notes, personal IDs, drafts and exact recipe quantities/units. Existing crash-interruption, migration, rollback and M1–M3C regressions also pass.

Offline CLI: rebuilt catalog_import with the current compiled registry before testing. Seventeen isolated imports/opens ran with process-local network denial; report catalog/production/m3d1-installation-report.json records fresh/v2/v3/v4/v5/v6 upgrades and repeats. Local debug wall time: fresh7 770.895ms, v6→7 262.396ms, unchanged v7 CLI open93.300ms (v6 scenario); these include process startup and are not universal performance guarantees. Worker repeat-startup tests assert no extra SQLite changes/audit rows.

Native: bundle identifier com.recipeatlas.m3d1-verification, 36.46MiB unsigned debug app. First launch and full process restart independently read568catalog/269aliases/50categories, schema5, release7, personal0, one successful import, integrity ok and zero FK violations. Test process identity was verified by its open isolated database before stopping. Native report: catalog/production/m3d1-native-report.json. Computer-use tooling resolved the bundle path to an obsolete identifier and rejected the direct M3D-1 identifier, so no fresh GUI interactions are claimed. Browser/component tests and real-worker/SQLite tests do not substitute for native GUI testing. Normal personal profile was not touched. Windows, Intel macOS, signing/notarization and installers remain unverified.

The new review workbook contains4rows/11columns with readable rendering, filter table, frozen header and blank human-decision/reviewer cells. Three historical terms remain pending in the existing review system. Root-level pre-existing workbook was not modified or used.

Focused final SQLite measurement: all60 new identity bilingual/alias lookups, detail reads and category-filter checks completed in810.175ms on this debug build (combined operations, not a single-query latency). The final lint pass also checked the new workbook builder; explicit Node process/console imports resolved its initial no-undef findings.

## M3D-2A preparation verification

Run `python3 tools/prepare_m3d2a_candidates.py --check` and `python3 tools/test_m3d2a_candidates.py` (10 tests). Optional pinned-artifact re-extraction is documented in CATALOG_M3D2A_PREPARATION.md. Rebuild `catalog_import` before `python3 tools/report_m3d2a_coverage.py`; it imports unchanged release 7 into a temporary database and verifies 568/269/50, 464 food/104 beverage, foreign keys and integrity. It never uses a personal profile. Candidate generator tests cover source traceability, duplicate IDs, invalid categories, Unicode/Turkish alias matching, source-identity detection, legal deferral, seven prior pending decisions, empty factual metadata and immutable production/history checksums.

Executed this milestone: rebuilt offline Rust example; isolated SQLite measurement; pinned USDA/FoodOn extraction comparison and offline drift check; 10 candidate + 5 catalog-audit + 4 M3D-1 Python tests; static production and M3D-1 quality checks. Workbook: all four sheets rendered/visually inspected; saved XLSX read-only inspection confirmed table filters, frozen headers, 194 candidate rows + seven prior decisions and blank human fields. Excel desktop interactions were not executed. No runtime source changes, so native GUI/Windows/Intel/signed builds and full runtime regression suites were not repeated for this documentation/preparation task.
