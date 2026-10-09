# M3B-3 final catalog coverage

Version 4 contains **508 identities**: **26 additions**, **411 food / 97 beverage**, **260 aliases**, **50 category nodes**. All 482 version-3 IDs remain intact. Eight shared validation identities are counted once, not added again.

454 identities are USDA-backed; 54 are original reference-backed project identities. All 508 carry project-authored localized labels/classification. No branded-product dump or empirical observations were added.

## Quality and review

No structural errors, missing bilingual names, exact normalized alias collisions or capitalization warnings. 25 qualified English base-label groups are heuristic concept candidates (full list in quality-audit.json); distinct raw/processed, salted/unsalted and related culinary forms are not automatically merged. This heuristic cannot establish semantic equivalence.

All 17 original terminology items remain pending; one new endive/hindiba naming concern gives **18 human-review items**. Zero independently human-approved labels. Existing 482 names/IDs were retained. CSV and detailed Markdown report include provenance, concern, retained proposal, confidence and review status. No evidence justified replacing an original disputed label silently.

Selected/imported validation failures: 0; this does not mean all investigated candidates were accepted. Black turtle bean identity overlap and Van otlu primary-page availability remain explicit exclusions. Falernum/allspice product identity and brand boundaries were not adequate for adding generic records from the consulted cocktail page. Missing nutrition/density/allergen/ABV: 508 each; unknown never means zero or allergen-free.

## Added identities

| Key | Turkish | English | Categories |
| --- | --- | --- | --- |
| project-aged-blended-rum | Harmanlanmış yıllanmış rom | Blended aged rum | spirits |
| project-agricole-rum | Agricole rom | Agricole rum | spirits |
| project-ale-beer | Ale bira | Ale beer | beers |
| project-coarse-pilaf-bulgur | İri pilavlık bulgur | Coarse bulgur for pilaf | grains, turkish-regional |
| project-edirne-cheese | Edirne beyaz peyniri | Edirne white cheese | dairy, turkish-regional, cheeses |
| project-fine-koftelik-bulgur | İnce köftelik bulgur | Fine bulgur for köfte | grains, turkish-regional |
| project-honey-syrup | Bal şurubu | Honey syrup | syrups |
| project-india-pale-ale | India pale ale | India pale ale | beers |
| project-koftelik-bulgur | Köftelik bulgur | Bulgur for köfte | grains, turkish-regional |
| project-lager-beer | Lager bira | Lager beer | beers |
| project-nar-eksisi | Nar ekşisi | Nar ekşisi (pomegranate reduction) | condiments, turkish-regional |
| project-orgeat-syrup | Orgeat badem şurubu | Orgeat almond syrup | syrups |
| project-pilaf-bulgur | Pilavlık bulgur | Bulgur for pilaf | grains, turkish-regional |
| project-porter-beer | Porter bira | Porter beer | beers |
| project-stout-beer | Stout bira | Stout beer | beers |
| usda-sr-168412 | Hindiba (endivyen, çiğ) | Endive (raw) | vegetables |
| usda-sr-168564 | Radicchio (çiğ) | Radicchio (raw) | vegetables |
| usda-sr-169236 | Yer elması (çiğ) | Jerusalem artichoke (raw) | vegetables |
| usda-sr-169722 | Buğday kepeği | Wheat bran (crude) | grains |
| usda-sr-170061 | Şalgam yaprakları (çiğ) | Turnip greens (raw) | vegetables |
| usda-sr-170068 | Su teresi (çiğ) | Watercress (raw) | vegetables |
| usda-sr-170172 | Hindistan cevizi sütü (çiğ) | Coconut milk (raw) | dairy-alternatives |
| usda-sr-170173 | Hindistan cevizi sütü (konserve) | Coconut milk (canned) | dairy-alternatives |
| usda-sr-170277 | Agave şurubu | Agave syrup | syrups |
| usda-sr-170458 | Tuz eklenmiş konserve domates suyu | Tomato juice (canned, with salt added) | juices |
| usda-sr-171423 | Haşhaş yağı | Poppyseed oil | oils |

## Category membership

Direct memberships overlap and do not sum to the identity total. Roots may have no direct members; descendant ingredients still belong to their hierarchy.

| Category | Direct members |
| --- | ---: |
| baking | 37 |
| beers | 7 |
| beverage | 3 |
| bitters | 1 |
| brandy | 6 |
| cheeses | 30 |
| chocolate-cocoa | 5 |
| coffee | 4 |
| coffee-tea | 2 |
| condiments | 13 |
| dairy | 42 |
| dairy-alternatives | 5 |
| dried-fruit | 9 |
| eggs | 7 |
| fermented | 4 |
| flours | 22 |
| fruit | 47 |
| garnishes | 6 |
| gin | 2 |
| grains | 33 |
| herbs | 20 |
| infusions | 2 |
| international-specialty | 7 |
| juices | 13 |
| legumes | 19 |
| liqueurs | 8 |
| meat | 7 |
| milk-powders | 2 |
| mixers | 3 |
| mushrooms | 8 |
| noodles-pasta | 9 |
| nuts-seeds | 23 |
| oils | 18 |
| other-beverages | 5 |
| poultry | 7 |
| sauces | 16 |
| seafood | 21 |
| soft-drinks | 3 |
| spices | 28 |
| spirits | 25 |
| sweeteners | 7 |
| syrups | 11 |
| tea | 6 |
| traditional-drinks | 4 |
| turkish-regional | 16 |
| vegetables | 57 |
| water | 2 |
| whisky | 8 |
| wines | 5 |

## Real offline installation and timings

Eight actual CLI process runs used isolated temporary databases with process-local network denial. Clean install: 508 inserted. v2 upgrade: 273 inserted / 235 updated. v3 upgrade: 26 inserted / 482 updated. All repeats unchanged. Rust upgrade fixtures additionally preserve personal ingredients, recipes, drafts and field overrides; repeated bootstrap writes zero additional SQLite changes. Killed-process rollback/recovery remains covered by the real subprocess test.

| Starting release | Installed release | Repeat | CLI wall time (ms) |
| --- | --- | --- | ---: |
| clean | 4 | False | 238.593 |
| clean | 4 | True | 80.939 |
| 2 | 2 | False | 113.096 |
| 2 | 4 | False | 232.644 |
| 2 | 4 | True | 82.956 |
| 3 | 3 | False | 219.731 |
| 3 | 4 | False | 230.483 |
| 3 | 4 | True | 82.97 |

Local debug measurements include process startup, database initialization/checks and import where applicable; not a portable SLA. Focused Rust measurements: v2→v4 import 190.415 ms, v3→v4 186.139 ms; 100 actual Turkish partial searches 80.595 ms. See TESTING.md for native/package checks and limitations.

## Remaining coverage gaps

- Regional Turkish cheeses (including Van otlu), fermentation starters and regional plants need primary identity/terminology curation.
- Asian and Latin American specialty sauces, grains and fresh culinary forms remain unevenly covered.
- Coffee beans/roasts, loose tea varieties and herbal infusions need further verified identity selections.
- Vermouth, additional liqueur/rum/vodka/tequila varieties, tonic variants and specialty bar concentrates remain incomplete.
- Generic-to-specific ingredient relationships are extensible but not comprehensively curated; do not assume interchangeability.
- Nutrition, density, allergens and ABV remain unknown for every identity in this identity-only catalog.

Full long-term catalog and V1 scope remain unchanged. M3C should improve discovery/filtering and catalog customization without manufacturing missing metadata or claiming full worldwide ingredient coverage.
