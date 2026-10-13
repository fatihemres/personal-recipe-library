# Canonical ingredient schema — M3A

Implemented by immutable-after-release migration `004_catalog.sql`. Migrations 001–003 remain byte-for-byte unchanged. Current schema version: 4. Migration framework continues checksum validation, transactional DDL, foreign keys and an online pre-upgrade snapshot. M2B executables reject this newer schema; there is no downgrade. Restoring a snapshot loses subsequent changes and is not a portable backup.

## Identity and ownership

`ingredients` remains the recipe-facing identity table. Existing personal IDs, names, metadata, revisions and recipe/draft references are untouched. A nullable `catalog_id` references a separate `catalog_ingredients` definition. Existing records remain origin `personal`; materialized built-in rows are origin `catalog`.

Catalog UUIDs are deterministic SHA-256 UUIDv8: first 16 bytes of UTF-8 `recipeatlas:ingredient:<curated-key>`, with version/variant bits set. Categories use `recipeatlas:category:<key>`. Keys are permanent curated identities, independent of changing external IDs or labels. Changing a form's meaning requires a new key and reviewed relationship. Keys are globally unique in this application's curated namespace; manifests share the `recipeatlas-ingredients` dataset. Do not reuse a key for a different food, brand, preparation state or source record.

| Table | Responsibility / invariants |
|---|---|
| catalog_ingredients | Stable ID/key, canonical identity name, food/beverage/alcohol/garnish/other type, preferred unit FK, dataset/current release, record checksum |
| catalog_names | Exactly TR/EN localized names validated by importer; one per locale/identity, persisted normalized search key |
| catalog_aliases | Alternate names, synonyms/search aliases per locale; normalized duplicates per identity prohibited |
| ingredient_categories | Stable hierarchy, TR/EN names, personal/catalog origin and dataset; parent FK, self/cycle protection |
| catalog_category_memberships | Many-to-many canonical category membership, no scientific hierarchy forced into UI |
| personal_category_memberships | Separate user-owned category assignments to recipe-facing ingredients; preserved across catalog imports |
| catalog_dimensions | Explicit allowed mass/volume/count dimensions; preferred unit must match a dimension |
| catalog_relations | Distinct culinary forms linked by `form_of` or `related`; importer rejects form hierarchy cycles |
| catalog_sources | Immutable composite source/version snapshot, URL, license, attribution, retrieval date, evidence SHA-256 |
| catalog_provenance | Multiple source/version/external-ID mappings and factual descriptions; FK to source; external identity never reassigned silently; historical mappings retained |
| catalog_observations | Optional density/nutrient/allergen/ABV/dietary observations with exact text value, unit, basis and FK to that ingredient's provenance |
| catalog_releases | Dataset/version/purpose, manifest and generated artifact checksums, installation time |
| catalog_import_runs | Running/succeeded/failed audit, structured error code, JSON coverage/statistics, timestamps |
| ingredient_customizations | Explicit name/notes/preferred-unit override markers on recipe-facing catalog records |
| catalog_collisions | Pending/linked canonical-to-personal candidates; no automatic merge |

Observation absence means unknown. Nutrients may explicitly be zero only when supported by source evidence. Allergen absence requires an explicit sourced observation; missing metadata is never “free from.” Density requires positive decimal text in g/mL and a descriptive basis. ABV uses sourced exact decimal text 0–100%, no default for a generic spirit/product. Numeric precision uses existing bounded decimal rules, not SQLite REAL or JS Number. Conversion, nutrition interpretation and ABV estimation remain later milestones. Supporting both mass and volume does not imply interchangeability.

## Personal integration and upgrades

On installation, exact normalized TR/EN names or aliases are checked against unlinked personal names. Candidates stay pending and the canonical definition is retained without publishing a duplicate recipe-facing row. Fuzzy or semantic similarities are **not** merged. Explicit revision-checked linking attaches metadata to the existing personal ID without changing its name or references. If a built-in row is already published, linking refuses implicit merging; a reviewed conflict/merge workflow is M3C work.

For unambiguous definitions a recipe-facing catalog row uses the canonical ID. User customization uses CAS and marks overridden fields transactionally; later catalog updates refresh only unoverridden name/preferred unit and never personal records or personal category assignments. Notes are user-owned and never refreshed from source data. The legacy customization API treats submitted values as explicit overrides, even when equal to defaults; the M3C-2 presentation command compares canonical defaults and provides reset-to-default UI. Canonical names/aliases continue to find renamed records.

Removed records are retained during upgrades to protect references; importer is additive/updating, never a purge/reconciliation tool. Personal records with similar non-exact names need explicit review. A candidate ledger can become stale after personal renaming; pending means review needed, not confirmed duplicate identity. UI collision review and full catalog editing are not implemented in M3A.

All normal reads/writes use the established Rust worker/repository. Catalog import is an explicit offline development CLI, not a renderer filesystem endpoint or automatic startup seed. Read-only catalog status is visible in Settings; recipe selection can use available personal/built-in rows via typed IPC. Personal ingredient manager retains its personal-only scope.

A catalog upgrade that would change an uncustomized system display name to an existing personal name is rejected with `CATALOG_NAME_CONFLICT`, rolling back the package. Review that collision rather than silently merging, renaming personal data or introducing duplicate display identities. This differs from initial installation, which retains pending candidates without materializing colliding built-in rows.

## M3B-1 application of the existing schema

No new migration; schema remains 4, migrations 001–004 unchanged. Production
release version 2 uses 235 canonical definitions and 39 hierarchical category
nodes. Eight version-1 identities preserve canonical UUIDs; operational personal
IDs are never reassigned. Existing source/version snapshots remain immutable;
new extraction snapshots store new evidence hashes with preserved external IDs.
All 235 optional observation sets remain empty (unknown). Classification and
TR/EN labels carry project curation provenance separately from unchanged USDA
English descriptors. Empty planned categories represent gaps, not coverage.

## M3B-2 package version 3

Schema 4 unchanged. 482 stable identities (443 USDA-backed, 39 original factual
project entries), 50 category nodes, 252 aliases. All 235 historical IDs remain;
no recipe-facing personal ID merge or deletion. Reference URLs and authored
identity basis reside in the checksummed project provenance description, not
fabricated upstream identifiers. `project-*` keys have only project provenance;
USDA rows retain exact FDC IDs/descriptors and separate project labels. Unknown
observations remain absent for all entries. New consumer subgroups use existing
parent keys. Full counts and overlapping memberships: CATALOG_COVERAGE_REPORT.md.


## M3B-3 package version 4

Schema 4 unchanged; migrations001–004 remain immutable. Package4 contains508 identities (454 USDA-backed, 54 project-only), 411 food / 97 beverage, 260 aliases and 50 categories. All 482 historical v3 IDs persist. New source evidence snapshots are separate from DB schema and dataset version; all personal/recipe/draft references and catalog field overrides retain their previous identity. Exact archived v2/v3 packages test upgrades. Metadata observations remain absent for every identity; no allergens/ABV/density/nutrition inferred.

## M3C-1 read models

Schema remains version4; migrations001–004 and seed version4 are unchanged. `ingredient_library` returns paginated canonical/personal rows, global counts and descendant facets in one read transaction. Its origin-qualified identity prevents ambiguity for pending collisions. `ingredient_detail(id, origin)` returns canonical TR name, localized aliases, all memberships, dimensions, external-source facts and factual observations when present. Canonical ID and nullable recipe-facing ID are separate. Existing linked personal records retain personal origin and their original recipe IDs; category/English metadata may be inherited from their canonical link. No browse/detail operation writes to the database or invokes imports.

## M3C-2 migration 005 and user-owned presentation

Migrations 001–004 remain byte-for-byte unchanged. Migration 005 adds `catalog_ingredient_favorites(catalog_id)` and `personal_ingredient_favorites(ingredient_id)` with single-column primary keys and foreign keys; favoriting never clones ingredient data. `ingredient_personal_aliases(catalog_id, alias, search_key, updated_at)` stores one optional user alias per canonical ingredient, separate from immutable `catalog_aliases`; its normalized key is indexed. `recipe_ingredient_recent(ingredient_id, recipe_id)` supports the query-derived recent view. The migration is checksummed by the existing migration ledger and validated on schema-4 upgrades and repeat initialization. As with other schema upgrades, an older binary cannot open a newer database; restore requires a compatible backup.

`set_catalog_presentation` validates nonempty bounded optional names/aliases, compatible unit dimensions and exact duplicate alias/name conflicts before changing the recipe-facing catalog row. It compares its expected revision in an immediate transaction, changes existing `ingredient_customizations` markers by field, and stores or removes the personal alias. Reset passes null choices and restores canonical TR/default unit while clearing personal notes and alias. Bundled names, aliases, source facts, observations and IDs are never updated by this command. Future seed updates must continue to respect these user-owned markers and the separate alias table. Favorites are keyed to canonical IDs even when their recipe-facing materialization is pending; personal favorites use the original personal ID.

## M3C-3 use of existing schema

No migration was needed. Unlinked personal category edits use the existing `personal_category_memberships` join inside the same revision-checked transaction as the personal ingredient edit. The `categoryIds` IPC field is optional for earlier callers. Canonical category definitions and membership rows remain immutable. The library's optional `uncategorized` read flag tests absence of direct or inherited membership; parent filters use the existing descendant closure and distinct counts. Catalog package 4 and migrations 001–005 remain unchanged.

## M3C-4 catalog version 5

No schema migration. The versioned, checksummed shared importer updates three Turkish `catalog_names` and uncustomized recipe-facing catalog display names in one transaction. `ingredients.id`, English names, aliases, category links, source descriptors, recipe references, quantities, units, favorites, personal ingredients and overrides are preserved. `recipeatlas-terminology/user-decisions-1` records only the three user naming decisions; historical v4 source snapshots remain immutable. A changed system name colliding with an unlinked personal ingredient rejects the entire update for explicit review. Repeated startup skips the same version/hash without extra writes.

## M3C-5 catalog version 6

Schema remains 5; no migration is needed. Release 6 updates exactly one Turkish `catalog_names` value (`usda-sr-174842`) and one English `catalog_names` plus matching `catalog_ingredients.canonical_name` value (`project-siyez-grain`). Stable IDs, aliases, category links and observations remain unchanged. The separate versioned terminology evidence carries only the two approved display-name changes. The shared transaction keeps user-owned presentation overrides, favorites, personal entries and recipe/draft references; a changed label colliding with an unlinked personal entry aborts the release. Review decisions for ten unchanged and three unresolved records are documentation metadata, not ingredient updates.
