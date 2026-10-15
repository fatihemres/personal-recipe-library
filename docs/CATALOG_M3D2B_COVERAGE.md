# M3D-2B measured release 8 coverage

All counts below come from fresh isolated SQLite imports through the real Rust catalog importer. Category memberships overlap and must not be summed as a catalog total.

Release 7: **568 identities, 464 food / 104 beverage, 269 aliases, 50 categories.**

Release 8: **647 identities, 536 food / 111 beverage, 269 aliases, 50 categories.** The 79 additions are 72 food and 7 beverage identities. No new aliases or category nodes were introduced.

Release-7→8 upgrade inserted 79, updated zero; repeated import changed zero. SQLite integrity was `ok` with zero foreign-key violations. The normal user profile was not opened.

| Category | Parent | Release 7 direct | Release 8 direct | Change |
| --- | --- | ---: | ---: | ---: |
| `baking` | `food` | 37 | 37 | +0 |
| `beers` | `beverage` | 7 | 7 | +0 |
| `beverage` | `—` | 3 | 3 | +0 |
| `bitters` | `beverage` | 1 | 1 | +0 |
| `brandy` | `spirits` | 6 | 6 | +0 |
| `cheeses` | `dairy` | 30 | 31 | +1 |
| `chocolate-cocoa` | `baking` | 5 | 6 | +1 |
| `coffee` | `coffee-tea` | 4 | 5 | +1 |
| `coffee-tea` | `beverage` | 2 | 2 | +0 |
| `condiments` | `food` | 13 | 13 | +0 |
| `dairy` | `food` | 42 | 42 | +0 |
| `dairy-alternatives` | `food` | 5 | 5 | +0 |
| `dried-fruit` | `fruit` | 10 | 16 | +6 |
| `eggs` | `food` | 7 | 7 | +0 |
| `fermented` | `food` | 4 | 4 | +0 |
| `flours` | `baking` | 23 | 26 | +3 |
| `food` | `—` | 0 | 0 | +0 |
| `fruit` | `food` | 52 | 70 | +18 |
| `garnishes` | `beverage` | 6 | 6 | +0 |
| `gin` | `spirits` | 2 | 2 | +0 |
| `grains` | `food` | 36 | 38 | +2 |
| `herbs` | `food` | 20 | 23 | +3 |
| `infusions` | `beverage` | 2 | 2 | +0 |
| `international-specialty` | `food` | 24 | 29 | +5 |
| `juices` | `beverage` | 19 | 22 | +3 |
| `legumes` | `food` | 21 | 23 | +2 |
| `liqueurs` | `beverage` | 8 | 8 | +0 |
| `meat` | `food` | 10 | 10 | +0 |
| `milk-powders` | `dairy` | 2 | 2 | +0 |
| `mixers` | `beverage` | 3 | 3 | +0 |
| `mushrooms` | `vegetables` | 10 | 11 | +1 |
| `noodles-pasta` | `grains` | 9 | 10 | +1 |
| `nuts-seeds` | `food` | 28 | 31 | +3 |
| `oils` | `food` | 19 | 21 | +2 |
| `other-beverages` | `beverage` | 5 | 6 | +1 |
| `poultry` | `food` | 7 | 7 | +0 |
| `sauces` | `food` | 16 | 21 | +5 |
| `seafood` | `food` | 26 | 29 | +3 |
| `soft-drinks` | `beverage` | 3 | 4 | +1 |
| `spices` | `food` | 29 | 29 | +0 |
| `spirits` | `beverage` | 25 | 25 | +0 |
| `sweeteners` | `food` | 7 | 7 | +0 |
| `syrups` | `beverage` | 12 | 12 | +0 |
| `tea` | `coffee-tea` | 6 | 7 | +1 |
| `traditional-drinks` | `other-beverages` | 4 | 4 | +0 |
| `turkish-regional` | `food` | 16 | 16 | +0 |
| `vegetables` | `food` | 81 | 110 | +29 |
| `water` | `beverage` | 2 | 2 | +0 |
| `whisky` | `spirits` | 8 | 8 | +0 |
| `wines` | `beverage` | 5 | 5 | +0 |

These direct counts are database measurements, not human linguistic validation. The 27 deferred USDA concepts, 88 FoodOn concepts and seven historical questions remain outside this release. Regional Turkish, specialty bar/bitters and dairy-alternative gaps still need separate legally cleared sources.
