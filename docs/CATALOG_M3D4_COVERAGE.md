# Release 10 SQLite-backed category coverage

Fresh isolated SQLite imports through the real Rust importer measured 745 Release-9 and 795 Release-10 identities. Direct category memberships overlap, so category counts cannot be summed into the catalog total.

**Release 10:** 795 identities (666 food / 129 beverage), 269 aliases, 50 categories. Fifty identities were added: 35 USDA Foundation and 15 pinned Wikidata concepts. No prior canonical object, alias, or category node changed.

| Category | Release 9 direct | Release 10 direct | Change |
| --- | ---: | ---: | ---: |
| `baking` | 40 | 40 | +0 |
| `beers` | 7 | 7 | +0 |
| `beverage` | 3 | 3 | +0 |
| `bitters` | 1 | 1 | +0 |
| `brandy` | 6 | 6 | +0 |
| `cheeses` | 40 | 43 | +3 |
| `chocolate-cocoa` | 7 | 7 | +0 |
| `coffee` | 7 | 7 | +0 |
| `coffee-tea` | 2 | 2 | +0 |
| `condiments` | 20 | 20 | +0 |
| `dairy` | 44 | 46 | +2 |
| `dairy-alternatives` | 10 | 13 | +3 |
| `dried-fruit` | 16 | 16 | +0 |
| `eggs` | 7 | 7 | +0 |
| `fermented` | 6 | 10 | +4 |
| `flours` | 34 | 44 | +10 |
| `food` | 0 | 0 | +0 |
| `fruit` | 73 | 73 | +0 |
| `garnishes` | 6 | 6 | +0 |
| `gin` | 2 | 2 | +0 |
| `grains` | 45 | 53 | +8 |
| `herbs` | 27 | 27 | +0 |
| `infusions` | 2 | 2 | +0 |
| `international-specialty` | 47 | 72 | +25 |
| `juices` | 23 | 28 | +5 |
| `legumes` | 34 | 37 | +3 |
| `liqueurs` | 8 | 8 | +0 |
| `meat` | 14 | 16 | +2 |
| `milk-powders` | 2 | 2 | +0 |
| `mixers` | 5 | 5 | +0 |
| `mushrooms` | 11 | 15 | +4 |
| `noodles-pasta` | 10 | 11 | +1 |
| `nuts-seeds` | 38 | 39 | +1 |
| `oils` | 24 | 24 | +0 |
| `other-beverages` | 6 | 9 | +3 |
| `poultry` | 8 | 8 | +0 |
| `sauces` | 24 | 27 | +3 |
| `seafood` | 31 | 31 | +0 |
| `soft-drinks` | 4 | 4 | +0 |
| `spices` | 32 | 35 | +3 |
| `spirits` | 25 | 25 | +0 |
| `sweeteners` | 7 | 7 | +0 |
| `syrups` | 12 | 12 | +0 |
| `tea` | 9 | 9 | +0 |
| `traditional-drinks` | 4 | 4 | +0 |
| `turkish-regional` | 16 | 27 | +11 |
| `vegetables` | 120 | 124 | +4 |
| `water` | 2 | 2 | +0 |
| `whisky` | 8 | 8 | +0 |
| `wines` | 5 | 5 | +0 |

**Turkish regional:** 16 → 27 direct memberships, including cheeses, cured meat, fermented tarhana, spice and mastic forms. These are editorially identified culinary ingredients, not claims of protected-origin certification.

The 50 additions do not achieve the aspirational 900–1,000 total. Remaining gaps include regional herbs and grains, North African and Latin American pantry staples, East/Southeast Asian sauces, botanical infusions and bar ingredients. FoodOn term lineage is still unresolved. The separate source and decision audit records each exclusion.

Fresh/import/upgrade integrity and foreign-key results are in `catalog/production/m3d4-sqlite-report.json`. Times there include debug CLI startup and are environment-specific.
