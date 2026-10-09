# Catalog coverage — M3A validation only

Measured by the real offline CLI against a fresh SQLite test directory on 2026-10-08. Captured report: `catalog/validation/coverage.json`. **8 validated canonical definitions; 8 available recipe-facing catalog ingredients; 0 production definitions; 0 pending collisions on clean install.** This is not the comprehensive catalog promised for V1.

Eight culinary identities: granulated sugar (USDA 169655), powdered sugar (169656), honey (169640), unfortified whole milk 3.25% (172217), brewed coffee (171890), raw lime (168155), olive oil (171413), raw whole egg (171287). Stable keys and IDs are in the seed; all have TR/EN names and two source mappings (USDA factual description plus authored curation). No density, nutrition, allergen or ABV observations are present.

| Category key | Direct validated membership count |
|---|---:|
| sweeteners | 3 |
| dairy | 1 |
| fruit | 1 |
| oils | 1 |
| eggs | 1 |
| coffee-tea | 1 |
| beverage (root, cross-use assignments) | 3 |
| food (root) | 0 |

Eight category nodes total; `food` is the parent of five leaf categories, `beverage` is the parent of coffee-tea. Root counts are direct assignments, not subtree totals. A culinary ingredient may be in several groups, so the counts above overlap and must not be summed as catalog size.

Real SQLite collision test with existing personal Şeker: 8 canonical definitions imported, 7 catalog identities available, 1 pending collision; original personal ID/name, recipe reference, draft reference and decimal quantity preserved. Explicit metadata linking reuses the personal ID. Re-import adds no duplicate definitions/operational rows. English/Turkish aliases find the linked personal row. Native and automated evidence are recorded in TESTING.md.

M3B must establish a reviewed quantitative target and representative checklist across **every** requested food/beverage/Turkish/international family. This fixture leaves vegetables, meat, poultry, seafood, grains/legumes, nuts/seeds, spices, sauces/fermentation, regional specialties, teas, juices/mixers, spirits/liqueurs/wine/beer/bitters and other specialty groups unpopulated. These are pending V1 requirements, not optional deletions. Count accepted unique forms after validation; report rejected/colliding records separately, and never inflate counts with aliases, brands or duplicated source rows.


## M3B-1 production coverage — 2026-10-09

The validation figures above remain the historical eight-record fixture. The
separate production package version 2 contains **235 verified identities**,
**39 category nodes**, **470 production provenance mappings**, **zero empirical
observations**. Eight common identities were re-reviewed and retain IDs; 227
are additional. Clean real CLI import: inserted/available 235, collisions 0;
repeat: unchanged=true, inserted/updated 0. Reports:
`catalog/production/coverage.json` (package) and `installation-report.json`
(actual SQLite import). Direct category memberships overlap; source counts are
235 USDA identities plus 235 project-curation mappings, not 470 ingredients.

All accepted records have TR/EN labels; incomplete accepted translations and
rejected selected records are zero. Unselected upstream records were not
exhaustively reviewed and are not counted as rejected. Eight shared validation
identities are intentional reuse, not unresolved duplicate production rows.
All 235 nutrition/density/allergen/ABV values remain unknown. Regional Turkish
and cocktail bitters groups have zero reviewed members; international specialty
has seven shared memberships. Full matrix/targets: CATALOG_COVERAGE_PLAN.md.
