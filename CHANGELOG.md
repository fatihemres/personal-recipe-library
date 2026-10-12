# Changelog

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
