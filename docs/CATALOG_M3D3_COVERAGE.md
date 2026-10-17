# Release 9 measured category coverage

These counts come from fresh, isolated SQLite imports through the real Rust importer. Direct category memberships overlap. The 98 additions are 88 food and 10 beverage identities; 745 total identities, 269 aliases and 50 categories.

| Category | Release 8 direct | Release 9 direct | Change |
| --- | ---: | ---: | ---: |
| `baking` | 37 | 40 | +3 |
| `beers` | 7 | 7 | +0 |
| `beverage` | 3 | 3 | +0 |
| `bitters` | 1 | 1 | +0 |
| `brandy` | 6 | 6 | +0 |
| `cheeses` | 31 | 40 | +9 |
| `chocolate-cocoa` | 6 | 7 | +1 |
| `coffee` | 5 | 7 | +2 |
| `coffee-tea` | 2 | 2 | +0 |
| `condiments` | 13 | 20 | +7 |
| `dairy` | 42 | 44 | +2 |
| `dairy-alternatives` | 5 | 10 | +5 |
| `dried-fruit` | 16 | 16 | +0 |
| `eggs` | 7 | 7 | +0 |
| `fermented` | 4 | 6 | +2 |
| `flours` | 26 | 34 | +8 |
| `food` | 0 | 0 | +0 |
| `fruit` | 70 | 73 | +3 |
| `garnishes` | 6 | 6 | +0 |
| `gin` | 2 | 2 | +0 |
| `grains` | 38 | 45 | +7 |
| `herbs` | 23 | 27 | +4 |
| `infusions` | 2 | 2 | +0 |
| `international-specialty` | 29 | 47 | +18 |
| `juices` | 22 | 23 | +1 |
| `legumes` | 23 | 34 | +11 |
| `liqueurs` | 8 | 8 | +0 |
| `meat` | 10 | 14 | +4 |
| `milk-powders` | 2 | 2 | +0 |
| `mixers` | 3 | 5 | +2 |
| `mushrooms` | 11 | 11 | +0 |
| `noodles-pasta` | 10 | 10 | +0 |
| `nuts-seeds` | 31 | 38 | +7 |
| `oils` | 21 | 24 | +3 |
| `other-beverages` | 6 | 6 | +0 |
| `poultry` | 7 | 8 | +1 |
| `sauces` | 21 | 24 | +3 |
| `seafood` | 29 | 31 | +2 |
| `soft-drinks` | 4 | 4 | +0 |
| `spices` | 29 | 32 | +3 |
| `spirits` | 25 | 25 | +0 |
| `sweeteners` | 7 | 7 | +0 |
| `syrups` | 12 | 12 | +0 |
| `tea` | 7 | 9 | +2 |
| `traditional-drinks` | 4 | 4 | +0 |
| `turkish-regional` | 16 | 16 | +0 |
| `vegetables` | 110 | 120 | +10 |
| `water` | 2 | 2 | +0 |
| `whisky` | 8 | 8 | +0 |
| `wines` | 5 | 5 | +0 |

Release 7→9 and 8→9 import, idempotency, integrity and foreign-key results are in `catalog/production/m3d3-sqlite-report.json`. Historical-preview results are included only when an isolated 648-record profile copy is supplied.

Remaining gaps: regional Turkish cheeses/ferments/greens; cuisine-specific South/East/Southeast Asian and MENA condiments; botanical tea and coffee forms; cocktail bitters and syrups. The pinned USDA legacy archive offers limited generic coverage for these; FoodOn term clearance and alternative licensed sources require further review. Seven earlier terminology/identity questions and 29 deferred current candidates are not approved.
