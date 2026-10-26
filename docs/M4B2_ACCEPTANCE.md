# M4B-2 candidate acceptance and verification — updated 2026-10-26

Issue: [#10](https://github.com/fatihemres/personal-recipe-library/issues/10). Base: protected `main` at `8663787db4b68c8f4043f4a3405ae55eaf1dddd6`. Branch: `feature/m4b2-notes-filters-dashboard`. **Engineering candidate only; PO has not accepted the native feature or authorized merge.** Application version remains 0.1.0; Catalog Release 11 remains 854 identities, 269 aliases and 50 categories.

## Delivered behavior and boundaries

- Private recipe Markdown is separate from `recipes.notes` and editor drafts. The table records format version 1 and its own optimistic revision. The UI saves/clears it explicitly, previews a constrained Markdown subset using React text nodes, and offers Save/Discard/Continue before normal in-app navigation. Raw HTML, images, embeds and unsafe URL protocols are never interpreted. Native close attempts to flush dirty note text and leaves the app open on failure. Private means local to this profile, **not encrypted**. It is not indexed in recipe FTS. Duplicate copies the private note into an independent row; archive/trash retain it; trash displays it read-only until restore; explicit permanent purge cascades it. These lifecycle choices are proposed for PO acceptance in DEC-017.
- The first isolated native GUI run found that Tauri's predefined Quit menu item bypassed the WebView flush request; an unsaved note remained at its previous revision after Cmd-Q. The candidate now routes Cmd-Q through the same `request-exit`/`finish_exit` handshake as window close. In the rebuilt arm64 bundle, Cmd-Q saved a dirty note at revision 2; a full relaunch displayed the new text.
- Saved filters use the exact M4A/M4B-1 query contract, persisted as format-versioned structured JSON with page values fixed to 0/24. Turkish-normalized unique names and optimistic revisions protect rename/update. Applying a filter resolves its current database references and rejects unsupported versions, invalid payloads and missing ingredient/collection references with recoverable errors; it never broadens silently. Results are live, and pagination resets to page 1. Collections remain separate static memberships.
- Dashboard derives active totals, food/beverage/favorite counts, bounded newly created/modified rows and collection counts from SQLite. Archived/trash rows are excluded. Buttons open the corresponding real Library filters or recipe detail. Preparation history, nutrition, pantry and full statistics remain in later milestones.

## Migration and preservation

New checksummed, forward-only migration `008_recipe_notes_filters.sql` creates `recipe_private_notes` and `saved_recipe_filters`. No earlier migration or catalog source asset changed. The migration runner validates existing schema 7 read-only, snapshots before writes and applies pending SQL transactionally. Rust fixtures exercise fresh initialization, 7→8 upgrade with existing recipe/organization/membership/decimal-text rows, repeat open, FK check, `integrity_check`, stale revisions, note lifecycle and missing saved-filter references. An older schema-7 binary cannot open schema 8; use a compatible pre-upgrade backup to downgrade, with potential loss of later edits. No normal `com.recipeatlas.desktop` profile was opened.

## Local verification

Results below are local checks for the implementation. The 2026-10-23 candidate head has separate exact-head PR CI evidence; this documentation follow-up must pass its own new CI run before PO handoff.

| Check | Result |
| --- | --- |
| `npm run typecheck` | PASS |
| `npm run lint` | PASS |
| `npm test` | PASS — 69 tests / 21 files; renderer fixtures do not prove SQLite persistence. |
| `npm run test:ui` | PASS — 4 Chromium shell tests on retry; one earlier attempt could not bind localhost port 1420 (`EPERM`). Browser has no native IPC. |
| `npm run build` | PASS |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | PASS |
| `cargo check --locked --manifest-path src-tauri/Cargo.toml` | PASS |
| `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | PASS |
| `cargo test --locked --manifest-path src-tauri/Cargo.toml` | PASS — 94 Rust/SQLite library tests, 0 binary and 0 doc tests |
| `python3 tools/audit_catalog.py --check` | PASS — 854 identities, no structural errors/exact alias collisions |
| `python3 tools/test_catalog_audit.py` | PASS — 5 tests |
| `python3 tools/check_app_version.py` | PASS — 0.1.0 consistent |
| Isolated production macOS arm64 app | PASS for launch, real UI navigation, private-note preview/save/leave confirmation, saved-filter persistence, dashboard click-through and dirty-note Cmd-Q/relaunch. GUI acceptance remains with the PO. |
| Previous PR head CI | PASS — `540574047310e8c0a1933c61f4c8a7588c26f610`, [run 40004327930](https://github.com/fatihemres/personal-recipe-library/actions/runs/40004327930): Frontend and browser; Rust, SQLite and catalog; macOS Apple Silicon package; macOS Intel package; Windows x64 installer — all 5 SUCCESS. Three unsigned artifacts are present. This is **not** evidence for a later documentation commit. |
| Governance follow-up head CI and package checksums | PASS — `661ebdd65a1f1d69782426367fa5a5b300fcfdab`, [run 40287924426](https://github.com/fatihemres/personal-recipe-library/actions/runs/40287924426): all five required jobs succeeded. The downloaded Apple Silicon, Intel and Windows artifacts matched their checksum manifests. Any later UI correction needs its own exact-head CI verdict; consult PR #11 checks and the final handoff. |

The local arm64 `.app` used the temporary identifier `com.recipeatlas.m4b2verification` and its separate SQLite profile. The profile reached schema 8 and Catalog Release 11 (854/269/50), with `integrity_check=ok` and zero foreign-key violations. A disposable recipe and saved filter survived a full process relaunch. The note preview displayed `<script>alert(1)</script>` as text. Dashboard counts matched one active food recipe; its recent-recipe and food links navigated to the actual Library, including when a recipe detail had previously been selected. An initial Cmd-Q test exposed the quit-menu defect above; the corrected, rebuilt bundle saved `# Bekleyen değişiklik` at revision 2 on Cmd-Q and displayed it after relaunch. The final rebuild showed the note read-only in Trash, then editable with its text intact after Restore. The app was closed after testing. This did not open or change the normal `com.recipeatlas.desktop` profile.

## PO-facing isolated Apple Silicon bundle — 2026-10-26

The unsigned acceptance application is at `/private/tmp/tarif-atlasi-m4b2/src-tauri/target/acceptance/Tarif Atlasi M4B-2 Acceptance.app`. Its `Info.plist` has `CFBundleIdentifier=com.recipeatlas.m4b2acceptance` and app version 0.1.0. The current responsive-layout rebuild has executable SHA-256 `b3f9637865d652e00accc43aebeb3f26b400836081d355a1ae2bc2b6547f3907`. It replaces the earlier local candidate build, whose executable hash was `1caa323786f165633a1960ae85f25b1f0e305c51e19c5d5415543a5b5731be89`. This local bundle is separate from the official PR artifacts, which retain the production identifier.

Before first launch, `~/Library/Application Support/com.recipeatlas.m4b2acceptance` did not exist. The launched app created its own `library.sqlite3` there, with schema 8, Catalog Release 11, 854 canonical ingredients, 269 aliases, 50 categories, **zero recipes**, `integrity_check=ok` and zero foreign-key violations. It did not use `~/Library/Application Support/com.recipeatlas.desktop`. Native accessibility interactions then created one disposable recipe and one saved filter, confirmed dashboard active/food counts of 1/1 and food-filter navigation, saw safe Markdown as text, moved the recipe to Trash (note read-only), restored it, and changed the note without pressing Save. Cmd-Q flushed that note to SQLite at revision 2; a full relaunch displayed the new text and retained the named filter. The app was closed cleanly. The acceptance profile now contains that one clearly named **test recipe** and filter; the PO may use them for inspection or create separate test records. No real personal data was copied into this profile.

In Finder, press **Shift–Command–G**, paste `/private/tmp/tarif-atlasi-m4b2/src-tauri/target/acceptance/`, and press Return. Open `Tarif Atlasi M4B-2 Acceptance.app` from that folder. Confirm the filename and isolated identifier before testing; do not open the unsigned PR arm64 artifact against the normal daily profile. Because this is an unsigned internal build, macOS may require the usual Finder **right-click → Open** flow. This is not a signed/notarized release.

## PO visual findings and scoped correction — 2026-10-26

The PO completed the functional Mac acceptance checks and found two layout problems. The first correction added responsive Collections and note styling, but a second PO review found that Saved Filters still had an inline label, weak alignment and disconnected actions. **Visual re-acceptance and merge authorization remain pending.** Note storage, Markdown rendering, conflict handling and native exit behavior were not changed.

The focused Chromium layout test uses the actual compiled application CSS at 320, 560, 920 and 1440 px, in light and dark themes. It checks that controls remain visible, the form and editor fit their panels, no horizontal page overflow appears, long note text scrolls internally, and the collection input-to-button Tab sequence works. This CSS fixture does not stand in for native feature acceptance. In the rebuilt isolated Mac app, actual Collections and private-note panels were visually inspected in a regular window and fullscreen: the collection input/button aligned without clipping, the note editor was shorter, and Write/Preview plus Save/Clear remained visible. Native automation could not resize the window to every narrow breakpoint (`noWindowsAvailable` from the window drag/scroll API); PO review at their preferred window size remains necessary. The isolated profile and normal `com.recipeatlas.desktop` profile stayed separate.

## Second PO layout finding: systematic workspace correction

The root cause was structural: Saved Filters used the generic `.recipe-actions` flex row, so its label remained inline, while Collections had separate one-off rules. Shared panels lacked consistent internal spacing; recipe list rules mixed grid and flex, and some fixed grid minimums could exceed a narrow content area. The follow-up uses one container-aware panel/form/list pattern for Saved Filters and Collections, with a label above the input, bounded form width, predictable stacked actions and wrapping long Turkish names. The same panel spacing is applied to the private note, recipe organization and dashboard sections. The search/filter grid, cards/list, toolbars and confirmation dialog now respect their content width. This is a presentation-only change; migrations, Rust, typed IPC, catalog and business logic are unchanged.

New Chromium tests render the **actual React AppShell and recipe components** with browser-only typed-client fixtures and production CSS. They check 360, 480, 520, 640, 768, 1024, 1440 and 1920 CSS-pixel widths in light/dark themes at 768 px height, plus 640 × 560, 640 × 600 and 640 × 900. They assert containment, label placement, non-intersecting actions, long-name wrapping, note internal scrolling, keyboard Tab/focus and Escape in the dialog. Selected actual-component screenshots were visually reviewed for Saved Filters at 360/480/520/768/1024/1440/1920 and both themes, plus Collections, note, list and dashboard at 480 in both themes. This is 7/7 passing browser tests (including the earlier five); 69/69 frontend tests, typecheck, lint and frontend build also passed locally. The application minimum width is 520 px and minimum height is 560 px, so 360 and 480 are browser stress cases below the native minimum.

The rebuilt isolated arm64 app was launched with the existing acceptance profile. At its regular 1120 × 800 CSS-pixel window, native screenshots and accessibility checks showed the Saved Filters form/entry actions, Collections form, recipe detail toolbar and Private Note editor/action row without clipping. The note panel was also inspected in macOS fullscreen. Native edge dragging returned `noWindowsAvailable`, so a 520 px native window, side-by-side placement, minimization/restore and other exact native sizes were **not verified**. The PO should inspect those layouts on their Mac. The isolated profile still reports schema 8, Catalog Release 11, 854 ingredients, 269 aliases, 50 categories, `integrity_check=ok` and no foreign-key violations. The normal `com.recipeatlas.desktop` profile was not opened.

## PO Mac acceptance checklist

Use an **isolated** arm64 `.app` built with a temporary Tauri identifier. The official PR artifact retains `com.recipeatlas.desktop` and must not be opened against the daily profile until the PO chooses to upgrade it. Verify the app identifier and its own fresh app-data directory first. Then:

1. Launch with network disconnected; confirm no storage/catalog error and schema 8 / Catalog Release 11.
2. Create a food recipe with a catalog ingredient at exact quantity `1.250 g`; add private Markdown with headings, emphasis and a malicious-looking HTML/link sample. Preview must show no executable content.
3. Save, open another recipe, return and confirm the private note. Change it from two windows or stale sessions; stale save must show a conflict and preserve the typed text.
4. Duplicate, archive, trash, restore and purge only disposable test recipes; confirm documented private-note lifecycle and stable recipe/ingredient references.
5. Save a named filter with text, kind and favorite or collection criteria. Apply it, verify page 1/live results, rename it, and remove it. A deleted referenced collection must yield an explicit unavailable state.
6. Open Dashboard and compare active/food/beverage/favorite counts with Library results. Follow recent and collection links; verify archived/trash recipes do not appear in ordinary counts.
7. Quit fully and relaunch. Confirm the note, filter, collection and exact quantity survived. Check both light/dark theme and keyboard navigation.

**Separate gates:** independent review, exact-head five-job Actions PASS, PO real-Mac GUI acceptance, and explicit PO merge authorization. Windows and Intel GUI execution, signing/notarization and daily-profile upgrade are not established by this candidate.
