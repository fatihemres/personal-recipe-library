# Changelog

## DEVOPS-1C — resource lifecycle audit and version display (2026-10-16, unreleased)

- Replaced the stale sidebar milestone label with the installed Tauri application version and added a Turkish Settings/About section. Metadata failure leaves the sidebar usable. The version remains 0.1.0; catalog release 8 is tracked separately.
- Resolved pending ingredient leave decisions on unmount and reused one decision for repeated navigation attempts. Prevented the periodic draft checkpoint timer from queuing more writes while an earlier periodic write is stalled. No catalog, SQLite migration, or user data changed.
- Added a lightweight three-manifest version consistency check to PR CI and documented native resource evidence and remaining profiling limits.

## PR #1 startup hotfix — catalog source-snapshot compatibility (2026-10-16, unreleased)

- Fixed release-8 startup on an existing 61-record M3D-1 preview profile. The corrected 60-record source evidence now has distinct reviewed snapshot versions; old checksummed source rows and recipe-facing ingredient IDs remain intact. No SQLite migration, catalog identity, alias or category was added or removed.
- Kept the exact earlier, successfully installed release-8 preview usable without rewriting it. Other same-version manifest drift remains an error.
- Startup errors now display their stable diagnostic code in Turkish without exposing database paths or personal content. Added SQLite-backed collision and preview-compatibility regressions.

## M3D-1 — First controlled expansion release (2026-10-14, unreleased)

- Embedded catalog release 7: 60 USDA-backed additions, 568 canonical identities, 269 aliases and the existing 50 categories (464 food / 104 beverage). Archived release 6 unchanged and preserved all original 508 complete ingredient records.
- Screened 61 candidates; deferred unroasted-kernel tahini because its distinction from an existing raw tahini identity remains uncertain. Corrected candidate barley/cranberry labels and removed the misleading karalahana equivalence from the kale alias before release.
- Added reproducible offline preparation, database-backed coverage, source decisions, quality/review artifacts and a four-item review workbook. Three historical terminology questions remain pending; no independent linguistic approval is claimed.
- Verified real SQLite upgrade/rollback/restart, source allowlisting, personal/recipe/draft/favorite/override preservation, complete automated regressions and isolated unsigned macOS arm64 build/startup/restart. Native GUI interaction and other platforms remain unverified.

## M3C-5 — Final scoped terminology decisions (2026-10-13, unreleased)

- Shipped offline catalog release 6 with exactly two display-name changes: Turkish `Club soda (kokteyl sodası)` and English `Einkorn wheat (siyez)`. Preserved 508 canonical IDs, 260 aliases, 50 categories and schema 5.
- Archived release 5, recorded ten user-authorized retain-name decisions and left three ambiguous terms unresolved. The review decisions do not claim independent expert linguistic verification.
- Verified transactional release upgrades, collision rollback, user presentation/favorite/personal/recipe/draft preservation, offline CLI imports, and isolated native macOS startup.

## M3C-4 — User-selected ingredient terminology (2026-10-13, unreleased)

- Applied three explicit Turkish naming decisions through offline catalog release 5, retaining all 508 IDs, 260 aliases, 50 categories, recipe references and user overrides. No schema migration or new ingredient was added.
- Archived release 4; added checksummed decision provenance, reproducible build/audit reports and an updated review workbook. Fifteen terminology decisions remain pending. Exact personal-name collisions roll back the upgrade for review.

## M3C-3 — Catalog quality and category workflows (2026-10-12, unreleased)

- Classified personal-ingredient collision candidates as exact names, matching aliases or weaker similarities; strong matches guide users to the existing record or a more precise name. No automatic merge.
- Added uncategorized browsing and revision-checked, transactional category assignments for unlinked personal ingredients using the existing schema. Parent/child facets and unique counts remain database-derived.
- Published a repeatable 508-record quality audit and a filtered, readable workbook for the 18 terminology decisions awaiting human review. Existing bundled identities, aliases, source data and migrations remain unchanged.


## M3C-2 — Ingredient personalization and selection (2026-10-09, unreleased)

- Added checksummed migration 005 for canonical/personal favorites and user search aliases; no canonical seed changes.
- Added library favorite/recent views, recipe-derived usage, ranked bounded bilingual/alias autocomplete, keyboard selection, canonical presentation editing/reset, duplicate warnings and unsaved ingredient navigation choices.
- Preserved all 508 bundled identities, existing recipe/draft references and exact quantities. Verified real SQLite upgrades/restarts, frontend and browser regressions, Rust checks and isolated native macOS package. Windows, Intel macOS and signed release checks remain pending.
- Final accessibility pass: ingredient leave choices use the shared modal with safe initial focus, Cancel/Escape focus restoration, disabled decisions during saves and visible in-modal save errors. Expanded focused tests and reran the full suite plus isolated native build/startup checks on 2026-10-12.

## Unreleased — Milestone 1 foundation

- Turkish responsive desktop shell, keyboard navigation, accessible light/dark/system theme choices, honest upcoming-module states and localized errors.
- Real app-data SQLite, bundled FTS5 probe, foreign keys, WAL, checksummed transactional migrations and pre-upgrade online snapshots.
- Dedicated Rust database worker, typed/validated IPC, durable theme preferences and initialization retry without data reset.
- Rust storage/command tests, renderer/contract tests, Playwright browser checks, lint/typecheck tooling and native macOS debug-bundle verification.
- Architecture/schema/data dictionary/testing/checklist/handoff documentation. No application features beyond foundation are enabled; no public release/tag is published.

## M2A — Persistent Recipe Management (2026-10-08, unreleased)

- Real SQLite recipe create/read/edit/list, independent custom ingredients with Turkish Unicode search and normalized duplicate reuse.
- Decimal-safe quantity strings and six standard unit references; stable ordered ingredient lines and preparation steps.
- Turkish editor/detail/library, kind/title filters, confirmed soft deletion, trash and restore, validation/error states and in-memory edit preservation across Settings navigation.
- Additive checksummed migration 002, revision conflicts, transactional rollback and FK safeguards; M1 preferences/migration/FTS infrastructure retained.
- Persistence/IPC/UI regression tests and packaged native macOS restart verification. M2B and later modules remain pending.

## M2B — Recipe Reliability & Advanced Management (2026-10-08, unreleased)

- Real SQLite new/edit drafts, incomplete input recovery, debounced/periodic checkpoints and graceful native close/Cmd-Q flush with failure retention.
- Serialized draft/Save/Discard operations, atomic draft cleanup, closed-session tombstones, stale editor protection and explicit recovery-as-new.
- Independent duplication, reversible archive/unarchive, active/archive/trash views, restore and confirmed permanent deletion preserving shared ingredients/drafts.
- Personal ingredient name/notes/preferred-unit edits, optimistic revisions, duplicate conflicts and reference-safe deletion.
- Turkish keyboard autocomplete, distinct actual empty/no-match/error states; saved SQLite Şeker and İ/i, I/ı regression coverage. No global catalog seeds.
- Additive migration 003 with unchanged applied 001/002; real storage/registered IPC tests, subprocess unexpected-termination recovery and native macOS restart verification. M3 and later milestones remain pending.


## M3A — Ingredient catalog foundation (2026-10-09, unreleased)

- Additive migration 004 for canonical bilingual identities, aliases, hierarchy, source provenance, optional exact observations, import/version audit, user overrides and collision candidates.
- Verified eight-record USDA SR Legacy validation package, source license manifest/attribution and deterministic offline preparation/transactional CLI import with checksums, idempotency and rollback.
- Personal ID/reference and customization protection, explicit CAS linking, safe upgrade name-conflict rejection, catalog-aware recipe autocomplete and real Settings counts. No automatic production seed.
- SQLite import/migration/restart/crash/collision/alias/upgrade regression tests and isolated native macOS verification; handoff/licensing/schema/pipeline/coverage documentation. M3B/M3C remain pending.

## M3B-1 — 2026-10-09 (local development, unreleased)

- Added 235 source-verified bilingual USDA ingredient identities, reproducible production extraction, coverage plan/reports and CC0 notices.
- Embedded checked offline seed installation through the SQLite worker, preserving personal collisions/overrides and existing IDs; no schema migration.
- Added production clean/upgrade/rollback/search/recipe-restart/version regressions and isolated macOS verification. Extensive catalog and signed cross-platform releases remain pending.

## M3D-2A — catalog expansion preparation

- Added a separate, reproducible 194-concept USDA/FoodOn candidate ledger, source-rights review, collision assessments and four-sheet terminology workbook. Human approvals remain blank; all imports remain unauthorized.
- Measured unchanged release 7 in isolated SQLite: 568 ingredients, 269 aliases, 50 categories. Added direct/descendant category coverage and cuisine-specific gap priorities.
- Preserved seven existing review questions and deferred tahini. No production catalog, schema or runtime changes; release 8 remains a future milestone.

## M3D-2B — controlled catalog release 8

- Added 79 distinct, source-pinned USDA ingredients after an auditable review of all 106 USDA candidates. Deferred 27 USDA concepts; retained all 88 FoodOn proposals outside production pending term-level source clearance. No human linguistic approval was implied.
- Archived release 7 and embedded checksummed release 8: 647 canonical identities, 269 aliases, 50 categories. Existing IDs, source snapshots, personal data and schema migrations remain unchanged.
- Added real SQLite fresh/upgrade/repeat/rollback, search/detail/category, recipe quantity, draft, favorite and override tests; source/coverage/decision reports and isolated macOS verification. No new application module or online requirement.

## DEVOPS-1 — PR validation and release safety

- Added a PR-only GitHub Actions workflow with independent frontend/browser and Rust/SQLite/catalog gates, followed by unsigned macOS Apple Silicon, macOS Intel and Windows packaging jobs and short-lived test artifacts.
- Documented feature-branch development, explicit Product Owner review before merge, preservation of the transition commits, milestone tags, semantic application versions, catalog releases and separate database recovery.
- Kept the repository private. Current GitHub plan rejects private-repository branch protection, so the no-direct-main-push policy is manual until enforcement becomes available.
