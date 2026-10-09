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
