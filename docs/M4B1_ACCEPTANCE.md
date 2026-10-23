# M4B-1 Recipe Organization — candidate acceptance record

Date: 2026-10-23. Scope: recipe favorites, independent To Try, nullable 1–5 whole-star rating, custom collections and many-to-many membership. M4B-2 owns private/rich notes, saved filters and the real dashboard. This is an engineering candidate, not Product Owner acceptance or merge approval.

## Storage and compatibility

Checksummed migration 007 backfills one organization row for every existing recipe and creates one for each future recipe. Organization has its own optimistic revision. Favorite/To Try/rating changes do not alter recipe content/draft revisions or exact decimal-text quantities. Collection memberships use FKs and composite uniqueness; collection deletion removes its memberships while retaining recipes. Archive/trash preserve metadata; active collection counts exclude both. Purge cascades only the purged recipe's organization and membership rows. The pre-migration online snapshot mechanism remains in place; schema-6 binaries cannot open schema 7 without restoring a compatible backup.

The isolated Rust migration test opens schema 6 containing a recipe with exact `1.250 g`, a recoverable edit draft and Catalog Release 11 with 854 canonical ingredients. It upgrades to schema 7, reopens, validates the migration checksum ledger, draft and quantity, catalog count and `PRAGMA integrity_check`. Separate tests cover FK integrity, stale revisions, Turkish-normalized collection names, no-op toggles, rollback on a failing membership trigger, archive/trash/restore/purge, and organization filters composed with FTS/type/ingredient/date/sort/pagination. No normal user database was opened.

## Local verification

At the final candidate check: `npm run typecheck`, `npm run lint`, `npm test` (**61 tests**), `npm run build`, `npm run test:ui` (**3 Chromium browser shell tests**), `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `cargo check --locked --manifest-path src-tauri/Cargo.toml`, `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `cargo test --locked --manifest-path src-tauri/Cargo.toml` (**90 Rust/SQLite tests**), `python3 tools/audit_catalog.py --check`, `python3 tools/test_catalog_audit.py` (**5 tests**), and `python3 tools/check_app_version.py` passed. The browser suite first encountered sandbox loopback `EPERM`; the authorized rerun passed. A production arm64 `.app` with temporary identifier `com.recipeatlas.m4b1verification` built and launched. Its native accessibility tree showed the offline recipe library, favorite/To Try controls, Collections navigation and version `0.1.0`. After quit and relaunch, its isolated SQLite profile reported schema 7, `integrity_check=ok`, zero FK rows, 854 canonical ingredients, 269 aliases and 50 categories. Native collection/recipe clicks were not executed; renderer tests and SQLite tests do not substitute for PO GUI acceptance. Windows and Intel native GUI remain unverified; CI package/test results will be linked after PR checks.

## Product Owner Mac acceptance checklist

1. Download the current PR arm64 artifact and verify its SHA-256 manifest. Keep a current backup of personal data before installing a schema-upgrading build.
2. Launch; confirm no storage/catalog error, catalog Release 11 remains searchable, and existing recipes/drafts retain exact amounts.
3. Favorite a recipe from card and list views, toggle To Try independently, then filter active recipes by each and both together.
4. Open detail; set, change and clear a 1–5 rating using keyboard and mouse. Confirm no rating is shown as unrated rather than zero.
5. Create two collections, add one recipe to both, rename one, and verify counts and membership. Delete a collection and confirm the recipe remains.
6. Archive, trash and restore an organized recipe; confirm active views hide it appropriately and restoration retains organization. Test confirmed permanent purge only with a disposable recipe.
7. Fully quit and relaunch. Check flags, rating, collections, recipe content, drafts and exact decimal quantity/unit remain intact.

Do not merge or tag until the PO explicitly accepts this build and separately authorizes the exact PR head for a merge commit.
