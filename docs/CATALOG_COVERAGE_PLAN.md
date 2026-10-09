# Catalog coverage plan — M3B

M3B-1 delivers the first production batch, not complete V1 coverage. All food and
beverage families remain required. Counts describe canonical culinary forms;
brands, alternate spellings and languages never inflate identity counts.

## Measurable goals

Complete M3B planning target: **800–1,000 distinct reviewed identities**, with at
least 100 beverage-oriented identities and 50 documented regional Turkish
identities. Group targets below are non-additive because memberships overlap.
These are coverage goals, not achieved counts or a permission to invent records.
The pinned SR Legacy archive supplies many generic raw/cooked/preserved forms,
but scientific/product descriptions require curation and do not automatically
establish useful culinary identities. This first selection verifies 235 exact
source IDs. The next selection should expand high-priority common gaps before
more obscure forms. Reassess targets against approved-source availability at
each checkpoint; document shortfalls explicitly rather than relabeling them.

USDA covers conventional foods and some generic drinks well. Regional Turkish
forms, specialist spirits and bitters need additional rights-cleared primary
sources or clearly attributed original factual curation with verified identity.
FoodOn remains conditional; OFF and CocktailDB remain excluded. Numerical goals
for those gaps are contingent on source and terminology review, not guaranteed
by today's approved archive.

## Coverage matrix

Current counts are direct memberships, not parent rollups; unique total **235**.
The eight validation identities are separately re-reviewed in production and
reuse their IDs. There are **227 additional identities**, not 235 new records.

| Group | M3B-1 actual | Complete M3B target | Useful subcategories / important gaps |
|---|---:|---:|---|
| Vegetables | 41 | 90 | Leafy, roots, alliums, brassicas, mushrooms; seasonal varieties |
| Fruit | 33 | 75 | Citrus, berries, stone fruit, tropical, dried fruit |
| Herbs | 12 | 35 | Fresh versus dried, leafy culinary herbs; thyme versus oregano remain distinct |
| Spices | 18 | 50 | Whole versus ground, seed versus leaf, blended seasonings |
| Grains | 9 | 35 | Rice, wheat, oats, pseudocereals; pasta and local grain forms |
| Legumes | 9 | 30 | Dry versus fresh versus cooked; distinct bean types |
| Nuts and seeds | 14 | 35 | Whole, kernels, raw/roasted; nut pastes |
| Meat | 3 | 30 | Species, cuts, ground forms; beef and lamb cuts |
| Poultry | 3 | 20 | Chicken/turkey/goose, cuts, skin and cooking state |
| Seafood | 7 | 50 | Fish species, shellfish, mollusks; anchovy and local species |
| Eggs | 2 | 8 | Whole, white, yolk; different species |
| Dairy | 8 | 45 | Milk, creams, cheeses, yogurt; kaymak and Turkish cheeses |
| Dairy alternatives | 2 | 15 | Plant drinks and fermented alternatives; soy/oat drinks |
| Oils | 6 | 30 | Plant oils, butter, animal fats; ghee |
| Sweeteners | 4 | 20 | Sugars, honey, molasses; pekmez distinct from syrup |
| Flour and baking | 11 | 45 | Flours, starches, leaveners, cocoa; flour extraction/form qualifiers |
| Sauces | 7 | 40 | Tomato, soy, fish, cooking sauces; biber salçası |
| Condiments | 7 | 40 | Salt, vinegar, pickles, capers; mustard and tahini |
| Fermented foods | 1 | 25 | Vegetable, dairy, grain ferments; do not presume every pickle is fermented |
| Regional Turkish ingredients | 0 | 50 | Tarhana, pul biber, isot, sumak, salep, mahlep, pekmez, nar ekşisi; verify each identity |
| International specialty ingredients | 7 | 50 | Miso, kombu, nori, tofu, regional pastes; source and form review |
| Water and mineral water | 2 | 8 | Tap, still bottled, mineral, sparkling; mineral water absent |
| Fruit and vegetable juices | 8 | 30 | Fresh, concentrated, preserved, vegetable; preserve added-sugar qualifiers |
| Coffee | 2 | 15 | Beans, ground, instant, brewed; roasting and decaf forms |
| Tea | 3 | 20 | Leaves, brewed black/green/oolong; dry tea leaves absent |
| Herbal infusions | 2 | 20 | Dry herbs versus prepared drinks; linden and sage infusions |
| Syrups | 2 | 25 | Simple, fruit, spice; simple syrup absent, no assumed concentration |
| Mixers | 3 | 15 | Tonic, club soda, ginger ale; ginger beer |
| Sodas and soft drinks | 3 | 20 | Cola, citrus, fruit; generic versus named commercial products |
| Garnishes | 4 | 25 | Shared fruit/herb identities; peel/zest forms distinct |
| Spirits | 3 | 15 | Rum, vodka, whiskey present; gin, tequila, rakı absent |
| Liqueurs | 3 | 20 | Coffee and mint present; fruit, herbal, orange forms |
| Wines | 4 | 12 | Table red/white/rosé and sake; fortified/sparkling types |
| Beers | 2 | 10 | Regular/light generic; styles and alcohol-free forms |
| Cocktail bitters | 0 | 8 | Aromatic, orange, specialty; no cleared records yet |
| Other beverage ingredients | 1 | 15 | Coconut water; non-alcoholic bases and concentrates |

## Curation and acceptance gates

1. Pin a rights-cleared source and its artifact hash; verify source identifier and
   exact culinary form. Generic and branded products are separate concepts.
2. Author explicit TR/EN labels and useful aliases, preserving raw/dried/ground,
   skin, salt/sugar, species and preparation qualifiers. Reject uncertain regional
   equivalence. Translations are marked project-curated, never source translations.
3. Review Turkish terms semantically against source descriptors. This batch has
   project review, not independent culinary expert certification. A future Turkish
   culinary reviewer should audit regional terms before releasing those families.
4. Validate deterministic identities, hierarchy, aliases, provenance and checksums.
   No metadata inference: absent density/allergens/nutrition/ABV remain unknown.
5. Import into isolated clean and upgrade databases, test selection/save/restart,
   collision/override preservation, and capture actual database counts.
6. Reproduce artifacts byte-for-byte, run regression/build gates, record gaps.

Production package coverage is catalog/production/coverage.json; CLI import
results describe actual installed rows, including suppressed personal-name
collisions. Category counts overlap and must not be summed as unique coverage.
Zero rejected selected rows means the explicit reviewed selection validated;
it does not claim that all unselected upstream rows were evaluated or rejected.
Incomplete translations are zero for accepted records; unresearched regional
terms are gaps, not fabricated translated records. Shared validation identities
are reuse candidates, not duplicate production rows.

## Exact M3B-2 recommendation

Expand the common generic selection toward 450–600 verified identities: dry tea,
coffee forms, common dairy/cuts, baking/flour/pasta, dried fruits, seeds, sauces,
mineral water and generic beverage bases. Review lawful primary references for
regional Turkish ingredients and specialty spirits separately; maintain exclusions
until redistribution rights are clear. Add independently reviewed regional terms
only with traceable identity/form evidence. Produce version 3 with delta coverage,
reproducible artifacts and the same clean/upgrade/preservation/native gates.
Do not implement M3C discovery UI or claim final M3B coverage in that increment.

## M3B-2 achieved checkpoint — production version 3

482 canonical identities (396 food / 86 beverage), 247 added over M3B-1;
443 USDA-backed / 39 original reference-backed. 252 aliases, 50 category nodes.
The 450–600 working target is met without branded dumps or invented metadata.
See CATALOG_COVERAGE_REPORT.md for every category/subcategory and coverage.json
for machine counts. Direct memberships overlap; total is not their sum.

New breadth includes cheeses/flours/noodles, Asian sauces/legume preparations,
dried fruits, raw and preserved forms, seafood, cocoa, decaffeinated beverages,
regional siyez forms/pekmez/Kars cheese/salep, gin/whisky/brandy/liqueur categories,
tequila/mezcal/rakı, bitters/syrup/citrus-peel bar ingredients. Missing translations
0; exact alias collisions 0; independent human language reviews 0, queued items 17.
The extensive 800–1,000 long-term coverage ambition remains unmet; M3B is not complete.

Recommended M3B-3: review the terminology queue with a Turkish culinary reviewer;
expand remaining Turkish cheeses, ferments, regional bulgur/pulses/plants and
Mediterranean/Middle Eastern specialties; balance rum/vodka/tequila styles,
vermouth/beer styles, coffee beans/tea leaves, tonic/bar syrups. Pin each approved
source and preserve generic/product/form boundaries. Package version 4 with new
source snapshots, retain v2/v3 upgrade fixtures, rerun preservation and native
checks. Do not mix this with M3C interface redesign.
