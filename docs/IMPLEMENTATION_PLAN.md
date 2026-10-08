# Implementation plan

Status: planning baseline 2026-10-08. M0 inspection completed; this documentation task completed; M1 foundation and M2A persistent recipe slice implemented and verified; M2B and M3–M11 pending. MASTER_SPEC.md is the scope authority and ARCHITECTURE.md describes proposed decisions. This replaces the earlier abbreviated roadmap without removing any scope.

## Delivery rules and common acceptance gate

Every milestone ends with the app runnable from the existing checkout. Build small vertical slices; migrate forward, preserve previously delivered workflows, and hide unavailable feature navigation rather than inventing data or inactive controls. No mock/demo records in the shipped application. Test fixtures are isolated from personal DBs and never substitute for implementation. Empty states accurately describe current data. Future functionality remains pending in the checklist.

For every code milestone: relevant strict type checks, configured lint checks, Rust formatting/check/tests, Vitest tests, applicable Playwright UI checks, production frontend build, native smoke test, regression checks, and progress/docs update. Establish missing scripts/tools in M1; do not pretend they already exist. Report passed/failed/untested with commands, platform, fixture provenance, and limitations. Resolve failures before marking complete. Use scoped logical Git commits after reviewing the diff; no automatic push/release. Initial native CI feasibility begins early; release validation does not wait to discover all Windows issues.

Tests must exercise real repositories and SQLite for data claims, real files for import/media/backup, and real native IPC for native integration claims. Browser IPC stubs may isolate presentation tests, but those results are explicitly renderer-only. Record performance budgets with representative datasets when features arrive, rather than inventing unsupported response-time promises.

## M1 — Working modular foundation and persistence bootstrap

Dependencies: this planning baseline. Scope R01/R02/R03/R16/R19/R20 foundation only.

Implement Turkish application shell with usable Recipe Library and Settings navigation, accessible light/dark theme, localized strings, error boundary, isolated typed IPC client, and Rust command/service/repository boundaries. Add compatible Tailwind/shadcn, validation and test/lint setup incrementally. Open real SQLite in Tauri app-data, enable foreign keys, verify FTS5 capability, introduce checksummed migration runner and preferences persistence. Add a native diagnostic command reporting storage/schema readiness, without exposing private paths. Keep the original greeting smoke test until the real IPC diagnostic replaces its verification value.

Define initial recipe/ingredient/step/unit DTOs and write `DATABASE_SCHEMA.md` and `DATA_DICTIONARY.md` covering implemented bootstrap tables plus the proposed full logical model. Do not create all speculative tables in migration 001; first migration needs migration metadata/preferences only. Document schema and IPC policy, identify native test route and macOS/Windows build prerequisites.

Acceptance:

- Existing checkout starts as a native macOS app; Turkish shell, keyboard focus, theme controls and honest empty Recipe Library work without network.
- A theme preference is saved through real Rust/SQLite and survives closing/reopening.
- Fresh DB initialization and repeated startup succeed without duplicate migrations; FKs and FTS5 are demonstrably available; unsupported newer schema fails without modification.
- No feature CRUD, third-party data import, or placeholder counts introduced. Clear storage-error UX; no unrestricted renderer SQL/filesystem access.
- Frontend/Rust builds and foundational tests pass; Windows CI/native feasibility recorded honestly if unavailable. New source modules remain small and justified.

Tests: real temporary DB migration repeat/checksum/newer-version/read-only-path cases; preference round trip; DTO invalid inputs; Turkish strings/theme/keyboard renderer tests; real native diagnostic + preference persistence smoke test. Verify bundled FTS5 with an actual virtual-table query.

**Exact first milestone boundary:** shell + real DB bootstrap + persisted settings + testing/contracts/schema documentation. Recipe creation is M2; catalog ingestion M3; dashboard statistics, full search, beverage calculations and other modules are later. Stop after M1 when a task requests M1 only.

## M2 — Persistent basic recipe workflow and quantity core

Depends M1. R05/R08 personal ingredients/R09 foundational units/R10/R16/R18 safety.

Implement Quick Add and basic detail/edit screens for food/beverage kind, title, positive yield, user-created ingredient definitions/lines, standardized mass/volume/count quantities, ordered step descriptions, timestamps. Basic recipe/ingredient CRUD, transactional aggregate writes, duplication, archive, soft delete/trash restore, delete confirmation and draft recovery. Unknown/as-needed quantities allowed explicitly. Introduce exact unit factors/decimal representation before persisting quantities; no density or household guesses. Add `MEASUREMENT_ENGINE.md` core specification and schema/data dictionary updates.

Acceptance: create/edit/view/list/duplicate/archive/trash/recover a real recipe offline; ingredient/step edits persist after restart and rollback together on failure; referenced ingredients cannot be destructively deleted; simple recipe requires no advanced fields; recover unfinished edits and handle stale revisions. Recipe list is actual DB data.

Tests: recipe/ingredient CRUD, invalid yields/quantities, rollback and revision conflicts, delete/restore, ingredient referential integrity, draft restart recovery, Turkish text round trip, exact cc/mL and mass/volume conversion cases, native create→restart→edit smoke workflow, Playwright form/accessibility checks.

### User-directed split: M2A and M2B

M2A delivers the real persistent vertical slice: food/beverage recipe create/read/edit/list, independent personal ingredient creation/search/exact normalized deduplication, nullable precise decimal quantities with six standard unit references, ordered steps, safe soft deletion and basic trash/restore UI. Transaction rollback, referential integrity, stale revisions, M1→M2 migration, Turkish Unicode and native restart are acceptance gates. No catalog fixtures, mock persistence or full measurement engine. This is the explicitly requested first part of M2, not completion of all M2 requirements above.

M2B remains responsible for duplication, archive state/filtering, durable draft autosave/recovery with crash/restart tests and clear saved-vs-draft policy; broader ingredient editing/deletion with reference protection; richer validation/accessibility/keyboard workflows and conflict-resolution UX. Refine list queries/pagination as needed ahead of large catalogs. Preserve existing stable identities and decimal strings. Basic trash and restore already work; expanded recovery/purge policies must remain compatible with future media/history safety. Full Quick Add/Advanced editor coverage remains tracked across M2/M5, and the complete conversion engine remains M6. Do not begin M2B or M3 during the M2A task.

M2B acceptance: duplicate has a distinct recipe identity and correctly remapped child IDs; archive is reversible and distinct from trash; unfinished drafts recover after quit/crash without overwriting saved revisions; reference-safe ingredient edits preserve recipes; M1/M2A regression suites and native restart smoke pass. Each migration is additive/checksummed; keep the application runnable between increments.

## M3 — Taxonomy and extensive offline ingredient catalog

Depends M2; license/coverage decisions may be researched during M1 without importing runtime data. R06/R08/R16.

Implement hierarchical food/beverage and ingredient categories, all listed category families, multi-category recipe assignment, tags and separate cuisine/country/region/method/diet/occasion/difficulty vocabularies. Build pinned-source staging/normalization/import utilities, aliases/localized names, ingredient/product separation, provenance/licenses, nutrition/allergen/dietary optional states, user overrides and explicit duplicate review/merge. Bundle validated seeds, idempotent source/seed versions and import audit. Publish `EXTERNAL_DATA_AND_LICENSES.md` and actual `CATALOG_COVERAGE.md`.

Acceptance: approve concrete extensive-coverage threshold and category/Turkish/beverage checklist before acceptance; fulfill it with real validated records shipped offline. First install needs no downloads. Licenses checked for every redistributed artifact and notices included. Repeated seed import has no duplicates; upgrades preserve personal edits and references. Cycles rejected; multiple categories persist. No fabricated nutrition/aliases/provenance; unknown values stay unknown.

Tests: seed repeat/upgrade/rollback, stable IDs/source mappings, user override preservation, duplicate detection vs distinct product/preparation states, taxonomy relationships/cycle rejection, unknown-vs-zero nutrients/allergens, normalization/coverage report counts, offline clean-install seed test and personal ingredient editor UI.

## M4 — Search-first library and organization

Depends M3. R02/R04 partial/R05/R14.

Implement recipe/ingredient FTS indexes, autocomplete, bounded typo tolerance, Turkish sorting/normalization, advanced filters, saved filters, cards/list, favorites, collections/membership, to-try, private/rich notes, ratings. Sanitize rich-note rendering and record format/version. Dashboard count/favorite/recent/collection links are delivered only with real queries.

Acceptance: search recipe title/steps/ingredients/aliases and filter/sort correctly; favorites/multi-collection membership persist; dashboard links match counts and filtered views; archive/trash inclusion explicit; query latency measured on catalog-sized data.

Tests: FTS write/update/rebuild, ranking/prefix/typos, Turkish İ/I/ı/i and Unicode normalization, filter composition/injection-like queries, pagination, collections/ratings/saved filters, rich-note sanitization, keyboard search/navigation and dashboard link tests.

## M5 — Advanced recipe editor, media, history, Cooking Mode

Depends M4. R03/R05/R10/R17.

Advanced fields; substitutions and related recipes; independent variations and immutable recipe versions/history/restore. Full step title, description, duration, temperature, equipment, image, line-ingredient links, technique/tips. Cover and step media import/thumbnails; safe reference cleanup/recovery; large readable Cooking Mode. Persist drafts across modes; automatic preservation never overwrites saved recipes without explicit save policy.

Acceptance: variations/history retain original recipe and ingredient meaning; version restore creates new revision; image thumbnails render efficiently; trash recovery retains images; purge protects shared/history references; steps reordered without broken associations; Cooking Mode navigable by mouse/keyboard offline.

Tests: version/variation round trip/restore, substitutions/related links, same-recipe step references, media shared-reference and orphan cleanup, corrupt images/oversize/path traversal/Unicode filenames, interrupted file finalize, draft recovery, Cooking Mode focus/long-text/accessibility UI and native media smoke test.

## M6 — Complete measurements and Beverage Studio

Depends M3/M5 and M2 quantity foundation. R07/R09.

All requested unit families, personal household/bar profiles/presets, preferred display units, contextual density conversions, exact scaling and practical fractional-count presentation, C/F. Beverage specialized categories and all fields in R07: serving/batch, classification/spirits, product ABV, temperature/glassware/ice/method/garnish/origin/flavor/sweetness/acidity/intensity, instructions/presentation. Estimated ABV with explicit knowledge status and dilution assumptions.

Acceptance: 25/35/50 cc/custom input, personal presets persist; 1 cc = 1 mL; never automatic g↔mL; scaling preserves unrounded quantities and explains fractional pieces/incompatibility; no universal shot/cup/dash assumptions. Every beverage field saves/reopens. Known mixers/dilution included and unknown ABV/volume never displayed as exact. Complete measurement spec and golden conversion fixtures with source references.

Tests: every standardized unit, round-trip/property tests, preset-edit snapshots, density applicability/null/invalid, rounding/fraction count, comma decimal parsing, C/F affine conversion, batch/serving scaling, product overrides, all-known/unknown/zero-ABV/melt/heated-beverage estimates, editor native persistence and UI.

## M7 — Pantry and shopping lists

Depends M6. R11/R12.

Lot-based inventory, locations/expiry/purchase price/currency; stock movement history; scaled-recipe availability/shortages. Multi-recipe shopping generation, dimension-safe aggregation, explicit pantry deductions, manual items/completion/logical grouping. Define stock-consumption confirmation and expired-stock policy before acceptance; checking a shopping item must not silently invent stock.

Acceptance: multiple lots for one ingredient remain distinct; shortages and aggregate shopping quantities honor conversion limits and substitutions; incompatible dimensions remain separate/explained; deduction is repeatable and does not double-count stock; manual additions/completion persist.

Tests: multiple lots, expiry, conversion/no-density, negative-stock rejection, concurrent updates, price optional state, multi-recipe aggregation, manual lines, regeneration/deduction idempotency, transaction failure, pantry/shopping native workflows and UI.

## M8 — Meal planning, preparation history, complete dashboard/statistics

Depends M7. R04/R13/R14/R15.

Daily/weekly recipe-to-meal assignments, portions linked to pantry/shopping; preparation logging and recently prepared; complete personal organization/dashboard actions and meaningful statistics across recipes/categories/ingredients/preparation/collections.

Acceptance: planner available but not mandatory; changing portions/days recomputes linked shopping safely; statistics use actual records and consistent archive/trash policy; all dashboard links filter correctly; history retains event/version meaning.

Tests: local-date/week boundary and timezone cases, portions/planner-shopping regeneration, deletion/archive references, preparation logs, database aggregate correctness (including empty DB), filtered dashboard links and planner keyboard UI.

## M9 — Portable backup, restore, JSON/CSV data interchange

Depends M5–M8; recovery foundations from M1. R18.

Online SQLite snapshot plus coherent media, portable manifest/checksums/schema/seed metadata, backup archive; staged validated restore preview/confirmation/recovery snapshot. Versioned JSON recipe export/import (including relationships/media policy); CSV for appropriate ingredient/pantry/tabular data; explicit conflict resolution, staging/import audit, failure/cancel recovery. Write `BACKUP_AND_RESTORE.md`.

Acceptance: full backup restores on both OSes with media and personal settings; WAL changes included; no machine-specific media paths. Malformed/newer incompatible archives fail before replacement; interrupted restore recovers the old or fully validated new generation. Imports never silently overwrite edits. Supported schema range documented.

Tests: real DB/media round trips, uncheckpointed WAL, checksums/missing/corrupt files, zip traversal/bombs/symlinks/case collisions/reserved names, newer/older schemas, preview cancellation, crash/failure injection at swap stages, JSON versions/conflicts/Turkish text, CSV escaping/encoding/formula-handling policy, macOS↔Windows backup interchange.

## M10 — Integration, data preservation, UX/performance/security hardening

Depends M9. All requirements regression gate.

Complete requirement-by-requirement audit, accessibility/theme/window-size refinement, long-recipe/media/search performance, CSP/capability review, corrupted/missing DB graceful recovery, permissions and privacy checks. Fill any feature acceptance gaps without mocks. Write `TESTING.md`, feature checklist, README usage/development, CHANGELOG and upgrade/downgrade documentation.

Acceptance: every V1 functional criterion implemented and covered by evidence; no demo values, disabled required features, hidden untracked deferrals, or mandatory network. Upgrade every supported historical schema/seed fixture without losing personal data; explicit unsupported/downgrade UX. Record performance dataset/budgets and accessibility/native manual results.

Tests: full Rust/Vitest/Playwright/native suites; real persistence upgrade matrix, broken media/DB recovery, SQL/filesystem misuse, offline app workflows, light/dark/keyboard/screen reader/window resizing, cross-platform path cases and performance regression.

## M11 — Native packaging and v1.0.0 release

Depends M10, native CI feasibility from M1, data licensing and signing credentials. R20.

GitHub Actions reproducible locked builds for macOS Apple Silicon, macOS Intel where supported, Windows installer; installers/native launch/upgrade/uninstall tests; offline Windows WebView2 strategy; signing/notarization and artifact verification. Establish minimum supported OS matrix with real evidence. Release docs, license notices, checksums, semantic version consistency, annotated version tag and GitHub Release preserving installable history.

Acceptance: all agreed V1 gates pass; trusted signed releases distinguished from unsigned internal tests. If credentials/Intel support are unavailable, report blocker/support limit, do not claim completion. Tag v1.0.0 only after acceptance review. Existing V1 artifacts retained through future releases; downgrade requires compatible schema or older backup and matching app.

Tests: native app/installer smoke tests each supported architecture/OS, clean offline install, upgraded user DB/media, backup transfer, signature/notarization verification, release manifest checksums, install/uninstall preserving user data policy. Publishing requires a scoped release task and applicable approval; this document does not publish anything.

## Requirement traceability and feature checklist

Full V1 requirement groups below remain in progress/pending; M1 foundational subsets are complete in FEATURE_CHECKLIST.md. A row is complete only when every detailed requirement in the linked MASTER_SPEC section passes its milestone gates; this table does not replace that detail.

| Requirement | Owning milestones | Release evidence |
| --- | --- | --- |
| R01 stack/modularity/offline | M1, M10, M11 | Contracts, builds, offline/native suites |
| R02 Turkish/localization | M1, M3, M4, M9 | Turkish text/search/sort/export tests |
| R03 design/Quick Add/advanced/drafts | M1, M2, M5, M10 | Usability, themes, accessibility, draft recovery |
| R04 dashboard | M4, M8 | Real counts/actions/filter links |
| R05 library/history/variations/media | M2, M4, M5, M6 | CRUD/recovery/version/scaling tests |
| R06 food taxonomy | M3 | All listed families and relationship tests |
| R07 Beverage Studio | M3, M6 | Every field, product ABV/dilution tests |
| R08 extensive ingredient catalog | M2, M3, M4 | Licenses, coverage, overrides, autocomplete |
| R09 measurements | M2, M6 | Complete unit/preset/density/scaling fixtures |
| R10 instructions/Cooking Mode | M2, M5 | Full step fields and navigation |
| R11 pantry | M7 | Lots/availability/shortages |
| R12 shopping | M7 | Aggregation/conversion/stock/manual/completion |
| R13 meal plans | M8 | Daily/weekly and linked portions/stock/list |
| R14 organization | M4, M5, M8 | Collections/history/ratings/notes/to-try |
| R15 statistics | M8 | DB-backed aggregate evidence |
| R16 normalized DB/seeding/migrations | M1–M9 | Constraints, transactions, upgrade/idempotency |
| R17 media | M5, M9 | Thumbnails/reference safety/portability |
| R18 backups/import/safety/privacy | M1–M10 | Real rollback/restore/offline/security results |
| R19 docs/handoff | Every milestone | Documentation inventory below |
| R20 tests/releases/execution | Every milestone, M10–M11 | Logged gates/native builds/release artifacts |

## Documentation deliverables and decision dependencies

Existing now: MASTER_SPEC.md, ARCHITECTURE.md, IMPLEMENTATION_PLAN.md, DATABASE_SCHEMA.md, DATA_DICTIONARY.md, FEATURE_CHECKLIST.md, TESTING.md, AGENTS.md, PROJECT_PROGRESS.md, updated README.md and CHANGELOG.md. Future required files are planned, not falsely listed as completed:

| Document | First owner / ongoing updates |
| --- | --- |
| DATABASE_SCHEMA.md, DATA_DICTIONARY.md | M1; every schema change |
| MEASUREMENT_ENGINE.md | M2 core, M6 full |
| EXTERNAL_DATA_AND_LICENSES.md, CATALOG_COVERAGE.md | M3; every source/seed version |
| FEATURE_CHECKLIST.md | M1 split R IDs into independently verifiable items; every milestone |
| TESTING.md | M1 initial tooling, M10 full evidence |
| BACKUP_AND_RESTORE.md | M9 |
| README.md, CHANGELOG.md | M1 setup; every milestone/release |
| RELEASE.md | M11; support/signing/install/upgrade/downgrade |
| PROJECT_PROGRESS.md | Every milestone with limitations/next task |

Catalog coverage size and legal source approval gate M3; household defaults and density evidence gate M6; rating/rich-note policies gate M4; history retention/purge gate M5; expiry/deduction policy gate M7; minimum OS/Intel/signing strategy gate M11 with early feasibility M1. Resolve each before its implementation acceptance; do not ask for every later decision before M1.

## V2 distinction

Only full English UI translation and optional video-reference user experience are explicitly future-facing requested capabilities; their V1 architecture support remains mandatory. Optional online sync/catalog refresh, hardware scanning, AI or video hosting are proposals, not commitments. No R01–R20 module is assigned to V2. Changing that requires a recorded user-approved scope revision, not a convenience decision during coding.
