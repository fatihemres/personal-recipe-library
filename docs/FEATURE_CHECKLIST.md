> **M5A integrated + PO accepted — verified 2026-10-28:** optional title, duration, explicit temperature/heat, equipment, technique, tips and own-recipe ingredient-line links merged in [PR #16](https://github.com/fatihemres/personal-recipe-library/pull/16) with migration 009/schema 9. The PO reported isolated Mac 5/5 PASS and [exact-head CI](https://github.com/fatihemres/personal-recipe-library/actions/runs/40645726359) passed 5/5. Instructions remain authoritative; M5B history/relations, M5C media, M5D Cooking Mode and M6 conversions remain unimplemented. See [M5A_ACCEPTANCE.md](M5A_ACCEPTANCE.md).

> **Checkpoint note, 2026-10-23:** This checklist's tick marks and introductory M1/M2-era totals are historical and not a current integrated-feature ledger. Use [PROJECT_STATUS.md](../PROJECT_STATUS.md), [ROADMAP.md](../ROADMAP.md) and [VERIFICATION.md](../VERIFICATION.md) for current milestone claims. Any unchecked full-V1 feature still requires its owning implementation/acceptance; M4A, M4B-1 and M4B-2 are integrated and PO accepted for their scoped Mac tests; remaining M5B–M11 and other V1 requirements are not thereby complete. Detailed R01–R20 requirements remain in [MASTER_SPEC.md](MASTER_SPEC.md).
> **M4B-1 integrated 2026-10-23:** recipe favorites, To Try, nullable ratings and collections merged in [PR #8](https://github.com/fatihemres/personal-recipe-library/pull/8) with schema 007 and PO-reported isolated Mac GUI 7/7 checks PASS. M4B-2 private notes, saved filters and dashboard were integrated via PR #11; do not interpret other unchecked lines as accepted.

# Feature checklist

**M4B-2 integrated tracking (PO Mac accepted 2026-10-27):** schema 008 private Markdown note storage and constrained preview; named versioned live filters; active-only SQLite dashboard and direct library navigation. The existing R04/R15 full statistics, prepared-history and later milestone requirements remain unchecked until their own acceptance.

Scope authority: MASTER_SPEC.md R01–R20. Checked items below are implemented M1/M2A/M2B subsets only. All unchecked items remain V1 unless labeled future; no unchecked feature is represented by demo records or mock-only functionality. IMPLEMENTATION_PLAN.md defines owning milestones and acceptance tests.

## R01 — Architecture/stack

- [x] Existing Tauri/React/TypeScript/Vite project extended without regeneration.
- [x] Modular React UI, typed/Zod-validated IPC, Rust commands/services/repositories.
- [x] Bundled SQLite, real FTS5 probe, database worker off UI thread.
- [x] Tailwind theme tokens and locally owned shadcn-style Radix/CVA button.
- [x] Vitest, Playwright renderer checks, lint/type checks.
- [ ] GitHub Actions native macOS/Windows build matrix and release verification.

## R02/R03 — Localization and UX

- [x] Turkish shell/document, externalized UI strings, future locale resource boundary.
- [x] Light/dark/system themes; persisted settings; keyboard nav/skip link/focus.
- [x] Responsive sidebar, header/content, explicit loading/empty/error states.
- [x] Upcoming modules disclosed without fake functional screens/statistics.
- [ ] Turkish sorting/search/import/export/filenames verified in feature workflows.
- [ ] Quick Add and Advanced Editor with recovered unfinished drafts.
- [ ] Destructive confirmation, undo/trash recovery across relevant workflows.
- [ ] Full-app accessibility, readability, reduced-motion and window-size QA.
- [ ] Complete English translation (V2; V1 localization architecture is implemented).

## R04/R15 — Dashboard and statistics

- [ ] Database-backed total/food/beverage/favorite counts and useful statistics.
- [ ] Recent added/edited/prepared and personal collections with quick actions.
- [ ] Correctly filtered navigation from every dashboard item.
- [ ] Real recipe/category/ingredient/preparation/collection statistics.

## R05 — Library

- [x] Food/beverage create/view/edit, ordered ingredients/steps, soft delete and basic trash restore.
- [x] Independent duplication, archive/unarchive, separate trash/restore and confirmed purge (M2B).
- [x] SQLite new/edit draft recovery, serialized autosave/Save/Discard, optimistic conflicts and explicit save-as-new (M2B).
- [x] Cards/list, sorts/filters/advanced FTS search (M4A) and recipe favorites (M4B-1).
- [x] Nullable whole-star 1–5 ratings (M4B-1).
- [x] Safe private Markdown notes (M4B-2).
- [ ] Recipe variations/related recipes/history/version restore (M5).
- [ ] Ingredient substitutions and portion adjustment.
- [ ] Cover/step images, optional source URLs and attribution.

## R06 — Food taxonomy

- [ ] Hierarchical extensible taxonomy with every food family listed in R06.
- [ ] Multiple categories/tags per recipe; user-defined categories.
- [ ] Separate cuisine/country/region/method/dietary/occasion/difficulty facets.

## R07 — Beverage Studio

- [ ] Specialized alcoholic/non-alcoholic editor and all R07 beverage categories.
- [ ] Classification, spirits/alcohol types, ingredient/product ABV configuration.
- [ ] Serving size/count, individual/shared format and batch volume.
- [ ] Temperature/glassware/ice type/quantity/method/garnish.
- [ ] Country/region/traditional-inspired-custom origin/flavor profile.
- [ ] Sweetness/acidity/intensity, preparation and presentation notes.
- [ ] Mixer/dilution-aware estimated ABV, assumptions and unavailable-data behavior.

## R08 — Catalog

- [x] Independent personal ingredient create/search/select and normalized exact duplicate reuse (M2A).
- [x] Personal ingredient name/notes/preferred-unit editing, rename duplicate conflict and reference-safe deletion (M2B).
- [x] Turkish normalized substring keyboard autocomplete with actual empty/no-match/error states (M2B).
- [ ] Ingredient organization, aliases/typo tolerance and advanced duplicate review.

- [ ] Extensive cleaned catalog ships offline, Turkish culinary terminology included.
- [ ] Coverage target reviewed; real per-category validated import count report.
- [ ] USDA/FoodOn/OFF/beverage references investigated with exact reuse/licensing gates.
- [ ] Every listed food/liquid/spirit/mixer/garnish/specialty family represented.
- [ ] Stable identity, canonical/localized names, English names and aliases/synonyms.
- [ ] Category hierarchy/type/preferred unit/allowed dimensions.
- [ ] Optional density/nutrition/allergens/dietary/ABV and product brand/barcode.
- [ ] Provenance/license/user metadata, explicit unknown states.
- [ ] Personal CRUD/organization, autocomplete/typo tolerance/duplicate review.
- [ ] Seed updates preserve personal modifications; no invented source/value data.

## R09 — Measurements

- [x] Decimal-text quantities and preserved g/kg/mL/cc/L/adet references; unknown quantities supported.
- [ ] Full conversion/scaling/normalization/presets engine.

- [ ] Full volume/mass/count/culinary/bar/custom/temperature units listed in R09.
- [ ] 25/35/50 cc/custom quantities and personal presets/display preferences.
- [ ] Canonical/display separation, exact compatible factors and cc=mL.
- [ ] Valid contextual density required for mass/volume; no assumed mL=g.
- [ ] Precise scaling, unrounded storage, sensible rounding/fractional-count UX.
- [ ] Household/bar definition snapshots and clear unsafe-conversion explanations.

## R10 — Instructions

- [x] Ordered instructions, add/edit/remove/reorder persisted (M2A).

- [ ] Ordered title/description/duration/temperature/equipment/images.
- [ ] Step ingredient references/technique/tips/notes.
- [ ] Distraction-free readable Cooking Mode with mouse/keyboard step navigation.

## R11/R12/R13 — Inventory, shopping and planning

- [ ] Multiple physical pantry lots per ingredient, quantities/units/locations/expiry/prices.
- [ ] Scaled recipe availability/shortages and explicit stock movement policy.
- [ ] Multi-recipe shopping aggregation/conversions/pantry deductions.
- [ ] Manual shopping additions/completion/logical grouping.
- [ ] Daily/weekly plans, recipes-to-meals/days and portion/pantry/shopping linkage.

## R14 — Organization

- [x] Recipe favorites/custom collections/multiple memberships/To Try/ratings (M4B-1).
- [x] Named saved filters/private notes (M4B-2).
- [ ] Preparation/cooking history (M8).

## R16 — Database

- [x] Real per-user DB, singleton preferences, safe initialization/retry.
- [x] Immutable migration SQL, SHA-256 ledger, user_version validation, FKs.
- [x] Transactional changes and pre-upgrade online snapshots with rollback tests.
- [x] Normalized recipe/ingredient/unit/step foundation with constraints and atomic saves.
- [ ] Remaining catalog/media/inventory/planner entities and indexes.
- [x] Source/seed/import metadata with repeatable idempotent imports/user overrides.
- [ ] Historical production schema/seed upgrade matrix and duplicate protection.

## R17/R18 — Media, portability and safety

- [ ] Photos, DB refs, safe relative app-data storage and efficient thumbnails.
- [ ] Shared/trash/history-safe media cleanup and interrupted-write recovery.
- [ ] Video-reference architecture (V1); video UI optional V2; hosting not required.
- [ ] Complete portable archive with DB/media/manifest/schema/integrity metadata.
- [ ] WAL-safe snapshot, validated restore preview/confirmation and crash recovery.
- [ ] JSON recipes and appropriate CSV tabs, explicit conflicts/import rollback.
- [ ] Cross-OS backup interchange, version compatibility and human-readable errors.
- [x] No account/backend/cloud/telemetry; current core foundation works offline.
- [x] Supported preferences validated; SQL parameterized; existing DB never reset.
- [ ] Full feature input/path/security/corrupt-data/deletion/upgrade protection.
- [ ] Unknown nutrition != zero; missing allergen data != allergen-free.

## R19/R20 — Documentation, tests and release

- [x] Guidance/progress/master/architecture/plan/schema/dictionary/testing/checklist.
- [x] README setup/current behavior and unreleased CHANGELOG.
- [x] M1/M2A/M2B actual SQLite/command/renderer tests and native macOS restart evidence.
- [ ] Measurement/data-license/coverage/backup/release docs at owning milestones.
- [ ] Full critical feature tests listed in R20, including Turkish and cross-platform cases.
- [ ] Signed/notarized native packages, verified supported OS/architecture matrix.
- [ ] Semantic tags/GitHub Releases/historical installable versions; v1.0.0 gates.
- [ ] Upgrade/downgrade/restore limitations and preserved V1 artifacts.

M3B-1: first embedded production batch contains 235 verified identities; extensive catalog checkbox remains open. Coverage plan and machine reports track actual versus planned breadth.
