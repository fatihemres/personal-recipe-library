# M4B-2 candidate acceptance and verification — 2026-10-23

Issue: [#10](https://github.com/fatihemres/personal-recipe-library/issues/10). Base: protected `main` at `8663787db4b68c8f4043f4a3405ae55eaf1dddd6`. Branch: `feature/m4b2-notes-filters-dashboard`. **Engineering candidate only; PO has not accepted the native feature or authorized merge.** Application version remains 0.1.0; Catalog Release 11 remains 854 identities, 269 aliases and 50 categories.

## Delivered behavior and boundaries

- Private recipe Markdown is separate from `recipes.notes` and editor drafts. The table records format version 1 and its own optimistic revision. The UI saves/clears it explicitly, previews a constrained Markdown subset using React text nodes, and offers Save/Discard/Continue before normal in-app navigation. Raw HTML, images, embeds and unsafe URL protocols are never interpreted. Native close attempts to flush dirty note text and leaves the app open on failure. Private means local to this profile, **not encrypted**. It is not indexed in recipe FTS. Duplicate copies the private note into an independent row; archive/trash retain it; trash displays it read-only until restore; explicit permanent purge cascades it. These lifecycle choices are proposed for PO acceptance in DEC-017.
- The first isolated native GUI run found that Tauri's predefined Quit menu item bypassed the WebView flush request; an unsaved note remained at its previous revision after Cmd-Q. The candidate now routes Cmd-Q through the same `request-exit`/`finish_exit` handshake as window close. In the rebuilt arm64 bundle, Cmd-Q saved a dirty note at revision 2; a full relaunch displayed the new text.
- Saved filters use the exact M4A/M4B-1 query contract, persisted as format-versioned structured JSON with page values fixed to 0/24. Turkish-normalized unique names and optimistic revisions protect rename/update. Applying a filter resolves its current database references and rejects unsupported versions, invalid payloads and missing ingredient/collection references with recoverable errors; it never broadens silently. Results are live, and pagination resets to page 1. Collections remain separate static memberships.
- Dashboard derives active totals, food/beverage/favorite counts, bounded newly created/modified rows and collection counts from SQLite. Archived/trash rows are excluded. Buttons open the corresponding real Library filters or recipe detail. Preparation history, nutrition, pantry and full statistics remain in later milestones.

## Migration and preservation

New checksummed, forward-only migration `008_recipe_notes_filters.sql` creates `recipe_private_notes` and `saved_recipe_filters`. No earlier migration or catalog source asset changed. The migration runner validates existing schema 7 read-only, snapshots before writes and applies pending SQL transactionally. Rust fixtures exercise fresh initialization, 7→8 upgrade with existing recipe/organization/membership/decimal-text rows, repeat open, FK check, `integrity_check`, stale revisions, note lifecycle and missing saved-filter references. An older schema-7 binary cannot open schema 8; use a compatible pre-upgrade backup to downgrade, with potential loss of later edits. No normal `com.recipeatlas.desktop` profile was opened.

## Local verification

Results below are local checks for this candidate; PR CI will verify the pushed exact head separately.

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
| PR five-job CI and package checksums | Pending PR |

The local arm64 `.app` used the temporary identifier `com.recipeatlas.m4b2verification` and its separate SQLite profile. The profile reached schema 8 and Catalog Release 11 (854/269/50), with `integrity_check=ok` and zero foreign-key violations. A disposable recipe and saved filter survived a full process relaunch. The note preview displayed `<script>alert(1)</script>` as text. Dashboard counts matched one active food recipe; its recent-recipe and food links navigated to the actual Library, including when a recipe detail had previously been selected. An initial Cmd-Q test exposed the quit-menu defect above; the corrected, rebuilt bundle saved `# Bekleyen değişiklik` at revision 2 on Cmd-Q and displayed it after relaunch. The final rebuild showed the note read-only in Trash, then editable with its text intact after Restore. The app was closed after testing. This did not open or change the normal `com.recipeatlas.desktop` profile.

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
