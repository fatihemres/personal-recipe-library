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

For unambiguous definitions a recipe-facing catalog row uses the canonical ID. User customization uses CAS and marks overridden fields transactionally; later catalog updates refresh only unoverridden name/preferred unit and never personal records or personal category assignments. Notes are user-owned and never refreshed from source data. Customization currently treats the submitted name/notes/preferred unit as explicit overrides, even when equal to defaults. Reset-to-default UX is future M3C work. Canonical names/aliases continue to find renamed records.

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
