# M3D-1 catalog coverage assessment

Counts below come from isolated SQLite databases populated by the production Rust importer. Category counts are **direct unique membership counts**; an ingredient may belong to several categories, so rows cannot be summed into a catalog total.

Release 6 baseline: **508 identities**, **260 aliases**, **50 categories**; food 411, beverage 97.
Current release 7: **568 identities**, **269 aliases**, **50 categories**; food 464, beverage 104.

| Category key | Parent | Baseline | Current |
| --- | --- | ---: | ---: |
| `baking` | `food` | 37 | 37 |
| `beers` | `beverage` | 7 | 7 |
| `beverage` | `—` | 3 | 3 |
| `bitters` | `beverage` | 1 | 1 |
| `brandy` | `spirits` | 6 | 6 |
| `cheeses` | `dairy` | 30 | 30 |
| `chocolate-cocoa` | `baking` | 5 | 5 |
| `coffee` | `coffee-tea` | 4 | 4 |
| `coffee-tea` | `beverage` | 2 | 2 |
| `condiments` | `food` | 13 | 13 |
| `dairy` | `food` | 42 | 42 |
| `dairy-alternatives` | `food` | 5 | 5 |
| `dried-fruit` | `fruit` | 9 | 10 |
| `eggs` | `food` | 7 | 7 |
| `fermented` | `food` | 4 | 4 |
| `flours` | `baking` | 22 | 23 |
| `food` | `—` | 0 | 0 |
| `fruit` | `food` | 47 | 52 |
| `garnishes` | `beverage` | 6 | 6 |
| `gin` | `spirits` | 2 | 2 |
| `grains` | `food` | 33 | 36 |
| `herbs` | `food` | 20 | 20 |
| `infusions` | `beverage` | 2 | 2 |
| `international-specialty` | `food` | 7 | 24 |
| `juices` | `beverage` | 13 | 19 |
| `legumes` | `food` | 19 | 21 |
| `liqueurs` | `beverage` | 8 | 8 |
| `meat` | `food` | 7 | 10 |
| `milk-powders` | `dairy` | 2 | 2 |
| `mixers` | `beverage` | 3 | 3 |
| `mushrooms` | `vegetables` | 8 | 10 |
| `noodles-pasta` | `grains` | 9 | 9 |
| `nuts-seeds` | `food` | 23 | 28 |
| `oils` | `food` | 18 | 19 |
| `other-beverages` | `beverage` | 5 | 5 |
| `poultry` | `food` | 7 | 7 |
| `sauces` | `food` | 16 | 16 |
| `seafood` | `food` | 21 | 26 |
| `soft-drinks` | `beverage` | 3 | 3 |
| `spices` | `food` | 28 | 29 |
| `spirits` | `beverage` | 25 | 25 |
| `sweeteners` | `food` | 7 | 7 |
| `syrups` | `beverage` | 11 | 12 |
| `tea` | `coffee-tea` | 6 | 6 |
| `traditional-drinks` | `other-beverages` | 4 | 4 |
| `turkish-regional` | `food` | 16 | 16 |
| `vegetables` | `food` | 57 | 81 |
| `water` | `beverage` | 2 | 2 |
| `whisky` | `spirits` | 8 | 8 |
| `wines` | `beverage` | 5 | 5 |

## Prioritized gaps

1. **Cuisine-specific cooking ingredients:** the baseline has only 16 direct `turkish-regional` and 7 `international-specialty` memberships. These labels do not measure all cuisine coverage; South/East/Southeast Asian, Middle Eastern/North African, Mediterranean/European and Latin American ingredients still need separately screened, source-backed candidates. No cuisine-specific category is implied by a name alone.
2. **Beverage preparation:** 1 `bitters`, 3 `mixers`, 4 `coffee`, 6 `tea`, 2 `infusions`, 6 `garnishes`, and 11 `syrups` direct memberships in the baseline make these high-value search gaps. Distinguish ingredient types from prepared drinks and branded products.
3. **Food breadth:** 4 `fermented`, 5 `dairy-alternatives`, 7 `meat`, 7 `poultry`, 8 `mushrooms`, 13 `condiments`, 16 `sauces`, 19 `legumes`, and 21 `seafood` direct memberships indicate useful room for specific forms and proteins. Distinguish raw/cooked/dried/powdered forms only when source descriptors support them.
4. **Taxonomy and translation:** the current 50-node hierarchy is retained in this increment. Existing categories may contain ingredients from several cuisines; a zero cuisine-specific node is not a zero ingredient count. The three M3C-5 terminology questions remain unresolved. Additional doubtful labels belong in a review queue, outside the production seed.

The machine-readable report contains the full category table and import/integrity results. Neither category counts nor USDA descriptor availability certify the cultural appropriateness of a translation.
