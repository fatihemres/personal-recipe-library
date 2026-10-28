# M5A structured editor candidate — 2026-10-27

Authority: explicit PO implementation brief and [M5 parent Issue #14](https://github.com/fatihemres/personal-recipe-library/issues/14), DEC-019. Branch `feature/m5a-advanced-recipe-steps` starts at protected main `1edfec3e6da56358a6c8bd7637503014f66fbfa9`. **IMPLEMENTED + VERIFIED candidate; PO-reported scoped native Mac 5/5 acceptance PASS on 2026-10-27; not INTEGRATED at this documentation checkpoint.** Final PR/head/CI URLs are recorded in the engineering handoff and the feature PR checks. Separate proprietary-rights PR #13 is untouched.

## Delivered boundary

Plain instructions remain sufficient to save. Each step optionally expands to title, whole-minute duration, whole-number temperature with explicit C/F or qualitative heat, equipment, technique, tips and own-recipe ingredient-line checkboxes. No duplicate description field, media, history, variations, Cooking Mode, conversions, preparation log or per-step amount allocation is included. All new UI text uses Turkish localization. Detail views show saved metadata and identify ingredient lines by their own IDs, order, quantities, units and notes.

## Storage and compatibility

Checksummed migration `009_structured_recipe_steps.sql` adds nullable scalar fields and a normalized `recipe_step_ingredients` join. SHA-256: `e23512b3d449e502652f89eb77d6a2e539eb9373e5bb11c85e97101241de7d8a`. Composite FKs ensure both endpoints belong to the same recipe. Domain validation rejects unknown/cross-recipe/duplicate line links. Archive/trash preserve them; restore recovers them; purge cascades owned rows. Removing a line clears its selections with a localized notice; unrelated metadata is retained. Duplicate and stale-draft save-as-new remap recipe, step and line IDs together.

Optional details carry `formatVersion: 1`. Duration is NULL or integer 0–10080 minutes in SQLite, transported as text to preserve incomplete draft entry. Explicit zero is not unknown; leading duration zeroes normalize on commit. Temperature is signed whole-number text −999–9999 with paired C/F unit; no silent conversion, precision approximation, safety limit or physical interpretation is claimed. Qualitative heat is independent free text. See DATABASE_SCHEMA.md for length bounds and rollback/backup implications.

Legacy step objects with only `id`/`instructions` decode without advanced data or links. Bounded unfinished numeric strings survive autosave and restart; unsupported versions cannot save or commit. Existing recipe IDs/revisions, ordered steps, notes/private notes, draft base revisions/tombstones, M4B organization, catalog overrides/favorites and decimal quantities remain authoritative. Migrations 001–008 and catalog assets are unchanged. An older schema-8 binary must not open the schema-9 profile; Git rollback cannot undo a SQLite upgrade.

## Automated verification

All commands use the isolated worktree and disposable SQLite databases, never the normal Mac profile.

| Command/check | Current evidence |
| --- | --- |
| `npm run typecheck` / `npm run lint` | PASS; browser TSX harness is included in strict TypeScript checking |
| `npm test` | PASS, 74 tests / 22 files |
| `npm run test:ui -- --reporter=line --workers=1` | PASS, 8 Chromium tests; typed presentation fixtures do not establish persistence |
| `npm run build` | PASS |
| `cargo test --locked --offline --manifest-path src-tauri/Cargo.toml` | PASS, 100 tests / 0 failures, including six new structured-step tests and extended IPC/kill-recovery regressions |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, Cargo check and all-target Clippy `-D warnings` | PASS after final Rust source changes |
| `python3 tools/audit_catalog.py --check` | PASS, 854 identities, historical identities intact, no structural/exact alias collisions |
| `python3 tools/test_catalog_audit.py`, `test_m3d1_catalog.py`, `test_m3d2a_candidates.py`, `test_m3d2b_catalog.py`, `test_m3d3_catalog.py`, `test_m3d4_catalog.py`, `test_m3d5_catalog.py` | PASS, 35 tests across seven suites |
| `python3 tools/check_app_version.py` | PASS, 0.1.0 in all manifests |

Six focused SQLite tests cover fresh schema 9, 8→9 migration/reopen, legacy edit draft JSON, exact `1.250 g`, metadata NULL/zero and validation bounds, same-recipe DB/domain constraints, stable reorder, independent duplication, line/step removal, archived/trash/restore/purge, stale edits and failed transaction/draft preservation, Turkish Unicode, catalog/favorite/override/private-note preservation and migration rollback/checksums. Existing real-worker IPC test now round-trips metadata and remapped duplicate links. The existing child-process kill/recovery test also verifies unfinished advanced values and line associations after unexpected termination. Historical migration fixtures use their original SQL columns rather than calling a current-schema repository against a historical DB.

## Responsive visual evidence

The actual React AppShell → Library → RecipeEditor/RecipeSteps hierarchy is rendered with production CSS and typed browser-only clients. Advanced fields are collapsed by default; keyboard Enter expands and Tab reaches the title; reorder preserves the selected line and values. Geometry checks cover 520, 640, 768, 1024, 1440 and 1920 CSS-pixel widths in both themes at height 768, plus 520-wide windows at heights 560, 600 and 900 and 125% root text at 640×600. Checks reject page overflow, escaping/intersecting controls and unequal control heights. Native minimum is 520×560. Long Turkish instructions and line names wrap/scroll normally. Screenshots at 520/768/1440 in both themes and enlarged text were visually inspected under ignored `test-results/structured-steps-*/`, including viewport captures. Earlier screenshots exposed stretched input heights; the final shared grid alignment, consistent control height, wrapping labels and two-column core metadata group correct them. Tall element captures may include a compositor artifact from the offscreen skip link; viewport captures were checked separately.

Browser screenshots are not native WKWebView acceptance. Native window automation/platform limits and the PO checklist are below.

## Isolated Apple Silicon candidate

The final unsigned arm64 bundle is `/private/tmp/tarif-atlasi-m5a/src-tauri/target/release/bundle/macos/Tarif Atlasi M5A Acceptance.app`. `Info.plist` was checked before launch: `CFBundleIdentifier=com.recipeatlas.m5aacceptance`, app version 0.1.0. Executable SHA-256: `f3d3cfc9dfe6e802636de19fcfac09f12d02cc9bc06c6149db3784c863f0ec6e`. Build command: `npm run tauri -- build --config /private/tmp/m5a-acceptance-config.json -- --locked --offline`, with a temporary config containing only the isolated product name, identifier and app bundle target. Production config is unchanged.

The runtime created its own `~/Library/Application Support/com.recipeatlas.m5aacceptance/library.sqlite3`; this directory did not exist beforehand. Fresh read-only verification found schema 9, Catalog Release 11, 854 canonical ingredients and zero recipes, integrity `ok`, zero FK violations. Actual native accessibility interactions then created one disposable recipe with a real catalog ingredient at **1.250 g**, saved plain instructions without advanced data, edited metadata (0 minutes, 180 °C, heat/equipment/technique/tips), selected the ingredient **line**, reordered steps and saved. Detail UI and a read-only SQLite query confirmed the stable step ID, position and line link.

A later native edit entered invalid duration `1.` and changed tips. Explicit Save showed validation and left the committed duration at 0. Cmd-Q flushed the draft; full relaunch exposed recovery. Recovery restored `1.`, tips, explicit C unit, order and the checked line. The committed recipe stayed unchanged. Final rebuilt app reopened both committed and draft data. Native screenshots were inspected at the regular 1120×800 window (dark) and fullscreen (dark and light). The initial native inspection prompted wider core-field grouping; the final rebuilt screenshots showed separated labels and controls. Edge-drag attempts did not change the window size reliably; **native 520-minimum and side-by-side exact sizes remain UNVERIFIED**, as do Intel/Windows GUI. Browser geometry is distinct evidence.

The app was quit cleanly. The isolated profile intentionally retains **one clearly named test recipe and one recoverable edit draft**, no copied personal data; final schema/catalog counts are 9 / 11 / 854 / 269 / 50, integrity `ok`, no FK violations. The normal `com.recipeatlas.desktop` profile, root workbook and unrelated worktrees were not opened or modified.

Finder launch: press **Shift–Command–G**, paste `/private/tmp/tarif-atlasi-m5a/src-tauri/target/release/bundle/macos/`, press Return, then open `Tarif Atlasi M5A Acceptance.app` (right-click → Open if the unsigned internal build prompts). Official PR packages retain the production identifier; use this explicitly isolated app for PO testing.

## PO acceptance result — 2026-10-27

The Product Owner reported **5/5 PASS** for the isolated `com.recipeatlas.m5aacceptance` application: plain-instruction recipe save; advanced metadata/line links and reorder/save/reopen; safe duplication/removal; unfinished advanced draft after full quit/relaunch; and responsive UI. This is a PO-reported manual GUI check, not a remotely reproduced native log. The actual PR HEAD may change due to premerge documentation corrections; verify all five CI jobs on the final HEAD. Windows/Intel native GUI and ordinary Mac production-profile upgrade remain untested.

## PO Mac acceptance checklist

1. Launch the explicitly isolated M5A app, confirm no storage error and schema 9 / Catalog Release 11.
2. Save an ordinary recipe with only plain instructions; no advanced field is required.
3. Edit a step, expand advanced details, enter title, duration `0`, explicit `180 °C`, qualitative heat, equipment, technique and tips. Select one or more ingredient **lines**, including repeated uses of the same canonical ingredient if desired.
4. Reorder steps, collapse/reopen, save and reopen. Confirm stable line selections and exact `1.250 g`; no quantity allocation or conversion occurred.
5. Remove a referenced line and confirm the notice and safe link cleanup. Duplicate the recipe and edit its metadata independently.
6. Leave an unfinished advanced edit using Keep Draft or Cmd-Q, fully quit and relaunch; recover it. Try invalid numeric text and confirm explicit Save refuses it without losing edits.
7. Inspect narrow/minimum, side-by-side, normal and fullscreen windows, short/tall windows, light/dark themes, long text, keyboard focus and larger text. Every action must remain reachable by natural vertical scrolling.

Next gates: independent engineering review, five CI checks on final head, PO native feature/visual acceptance and separate explicit merge authorization. Windows/Intel GUI/installer use, signed/notarized distribution and ordinary-profile upgrades remain unverified unless separately reported.
