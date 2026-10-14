# M3D-2A — expansion preparation, not a catalog release

Release 7 is unchanged: **568 identities, 464 food / 104 beverage, 269 aliases and 50 categories**. The baseline is commit `0e7f9850e86e960d38c1bb1972b99c2c64f246cd`. No release 8, schema, runtime, canonical name, alias, category, personal-profile or importer change is part of this milestone.

## Candidate inventory

`catalog/preparation/m3d2a/` is deliberately outside production and historical seeds. It contains **194 candidate source concepts: 106 USDA records and 88 FoodOn terms**. These are not 194 guaranteed net additions.

- **66 higher-confidence editorial proposals** (61 food / 5 beverage): rights-cleared USDA descriptors, explicit forms, no detected collision or recorded terminology concern. Human decision fields remain blank. These still require final culinary/linguistic and semantic identity review before release preparation.
- **40 USDA review candidates:** terminology, form ambiguity or similarity needs attention.
- **84 FoodOn source-clearance deferrals:** potentially useful terms; imported lineage and attribution must be reviewed before production authorization.
- **3 probable existing concepts:** Ancho, Pasilla and malt syrup; treat as mapping/review opportunities, not new identities.
- **1 rejected brand-specific proposal:** Cambozola; retained in the ledger to explain exclusion.

All 88 FoodOn entries remain unauthorized, including the three probable duplicates and rejected record. Total proposed distribution is 154 food / 40 beverage. At most 61 food / 5 beverage additions are the immediate lower-risk review tranche, not a promised release count. No factual observations or proposed aliases were added: source labels and project-proposed Turkish names suffice for review without inventing equivalence. A reviewer may propose aliases later with evidence.

The aspirational 200–500 range was not used as a quota. The 194 selected records improve breadth without padding with brand variants, salted/unsalted permutations, complete restaurant meals or unsupported Turkish translations. Regional Turkish coverage and specialty spirits remain weaker than desired. Those gaps require additional primary-reference curation, not further US sample variants.

## Coverage gaps and priorities

[Measured category coverage](CATALOG_M3D2A_COVERAGE.md) contains all 50 nodes, direct unique counts and descendant rollups from a new isolated SQLite import. Category memberships overlap. Cuisine is not a persisted exhaustive attribute, so no invented “Italian ingredient count” or regional completeness percentage is reported.

| Area | Evidence in release 7 | Material gaps / next research | Candidate contribution and boundary |
|---|---|---|---|
| Turkish/regional | 16 direct regional memberships; siyez grain/flour/bulgur, pekmez, Kars/Edirne cheese and selected traditional drinks | Tulum/lor/otlu cheese identities, tarhana forms, yufka/kadayıf, isot/mahlep, local pulses, pickles and plant terminology | Nettle and pastırma proposals are conditional FoodOn research, not approved Turkish coverage. TürKomp cannot be copied to fill this gap. |
| Mediterranean, Italian | 30 cheese, 16 sauce, 9 noodle/pasta memberships | Mascarpone, stracchino, specific pasta/dough forms, preserved lemon; regional olive/cheese boundaries | Cheese/condiment candidates, fresh and dried produce. Do not equate Romano with every Pecorino. |
| French and Spanish | General cheeses, dairy, spices and wine already present | Dijon, named generic cheese styles, culinary pastry forms, sherry; Manchego identity vs generic sheep cheese | FoodOn named cheese and wine proposals remain legally gated; brand-specific Cambozola rejected. |
| Middle Eastern | Tahini variants, bulgur, sumac-related project coverage | Labneh scope, freekeh, za'atar blends, preserved lemon, date products | Labneh needs identity review; do not duplicate existing tahini. No permission inferred from cuisine familiarity. |
| North African | General grains/spices but only 24 international-specialty memberships across all cuisines | Harissa, regional preserved citrus and spice blends | Harissa source-backed proposal; no unsupported blend composition. |
| Indian / South Asian | Legumes 21, spices 29, some international preparations | Paneer, curry leaves, fresh/dried fenugreek leaves, black/green cardamom, asafoetida forms | FoodOn proposals plus moringa/produce USDA candidates; compound asafoetida vs pure resin remains unresolved. |
| East Asian | Miso, shoyu, tamari, fuyu, oyster/fish sauces; mushrooms 10 | Kimchi, miso subtypes, yuzu, wasabi root, kanpyo, wonton wrappers, mirin | Never substitute imitation wasabi for root or generic miso for every subtype. Mirin alcohol/seasoning scope needs review. |
| Southeast Asian | Selected noodles, seaweed, soy foods and produce | Pandan, lemongrass parts/forms, young jackfruit, regional curry pastes | Leaf vs stalk vs powder remains explicit; no invented regional paste records. |
| Mexican / Latin American | Ancho, Pasilla, tomatillo, plantain and some sauces already installed | Cotija/Oaxaca, epazote, nopal, ají amarillo, fruit forms | Ancho/Pasilla proposals are probable duplicates. Nopal/epazote Turkish terminology is provisional. |
| General produce / mushrooms | Vegetables 81, fruit 52, mushrooms 10 | Leafy roots/tubers, tropical fruit, dried fruit and preserved mushrooms | Largest USDA tranche; no inference that raw entries are suitable for raw consumption. Preparation/safety advice is outside catalog metadata. |
| Meat / poultry / seafood / plant proteins | Meat 10, poultry 7, seafood 26; legumes 21 | Useful cuts/species and regionally recognizable proteins remain incomplete | Small roe/crab/legume selection, pastırma conditional. Frog-leg categorization unresolved; no bulk species/cut dump. |
| Dairy alternatives | 5 direct memberships | Cashew beverage/cheese alternative, almond cheese alternative, fermented plant products | Three conditional FoodOn proposals. No nutrition or allergen equivalence to milk. |
| Grains / flour / nuts / oils / baking | Grains 36, flours 23, nuts/seeds 28, oils 19, baking 37 | Bran, seed flours, triticale, specialty starch forms and pastry ingredients | USDA exact forms plus conditional tapioca pearls/sago/wonton; ambiguous sago not silently treated as tapioca. |
| Sauces / condiments / fermentation | Sauces 16, condiments 13, fermented 4 | Tartar/horseradish/peanut sauces, Dijon, kimchi/miso forms | A prepared sauce may be used as an ingredient; it is not its constituent raw food or a complete meal recipe. |
| Alcohol / bar | Spirits 25, liqueurs 8, bitters 1, mixers 3, syrups 12, garnishes 6 | Additional generic liqueurs/bitters, vermouth types, fortified wines, mixers | Conditional vermouth/sherry/Madeira/uzo/arak and syrup proposals. No generic ABV, brand-derived identity or invented bitters entry. |
| Non-alcoholic / specialty beverages | Coffee 4, tea 6, infusions 2, juices 19 | Beans vs brewed coffee, dry teas, matcha, botanical materials, purées and specialty mixers | Coffee bean/ground forms, dry oolong/rooibos/Darjeeling, juices and syrups. Matcha and specific fruit purées remain discovery gaps, not unsupported candidate rows. |

These are editorial coverage priorities, not assertions that every unspecified form is absent. Before importing any proposal, compare both existing identities and candidate peers, including broader/narrower concepts.

## Source decisions

Machine-readable source review: `catalog/preparation/m3d2a/sources.json`. Reviewed 2026-10-14. All entries explicitly keep `productionAuthorized=false`; this preparation manifest is not the runtime allowlist.

| Source | Decision and evidence |
|---|---|
| USDA FDC | CC0 public-domain data; commercial reuse and redistribution allowed, attribution requested. Pinned **SR Legacy April 2018**, not a claim of current composition data. Exact `fdc_id`/descriptors from the previously checksummed archive; English source facts separated from proposed Turkish wording. [Official license](https://fdc.nal.usda.gov/api-guide/#licensing), [downloads/formats](https://fdc.nal.usda.gov/download-datasets/). |
| FoodOn | Official repository declares **CC BY 4.0**, permitting commercial reuse with attribution/change notices. Pinned base ontology **2026-09-20**, git `cd73540243a84bcd511a500d9a497d12d7dc02f6`. Research extracts retain labels/IRIs and lineage references under attribution; no definitions, images or imported ontology hierarchy copied. Production remains blocked until selected SIREN/LanguaL/other lineage is reviewed and notices are complete. [Repository/license](https://github.com/FoodOntology/foodon), [dependency context](https://foodon.org/), [license terms](https://creativecommons.org/licenses/by/4.0/). |
| Open Food Facts | ODbL database, DbCL individual contents, separate CC BY-SA images. Commercial use is not itself prohibited, but derivative-database attribution/share-alike/access and combination obligations need resolution for this bundle. No records acquired. [Official license documentation](https://openfoodfacts.github.io/documentation/docs/Product-Opener/api/tutorials/license-be-on-the-legal-side/), [ODbL](https://opendatacommons.org/licenses/odbl/1-0/). |
| TürKomp | Excluded. Official terms require permission, identify separate commercial-use arrangements and annual software-use fees; limited noncommercial exceptions are not a freely redistributable catalog license. Bilingual site/code access does not authorize extracting names/composition into our bundle. No records acquired. [Official conditions](https://turkomp.tarimorman.gov.tr/useofdata), [database description](https://turkomp.tarimorman.gov.tr/about). |
| Wikidata | Structured entity data CC0; other page prose separately CC BY-SA. Potential future multilingual QID/revision mapping source, not expert culinary verification. No entity snapshot/candidates acquired in this batch. [Official policy](https://www.wikidata.org/wiki/Wikidata:Licensing). |
| TÜRKPATENT | Reference-only for future regional identity checks. No verified open bulk-redistribution license; no records/descriptions/images extracted. GI registration does not mean a generic ingredient meets the protected specification. [Official portal](https://ci.turkpatent.gov.tr/). |
| TheCocktailDB | Remains excluded: offline redistribution scope unresolved and linked terms currently identify TheMealDB. API access is insufficient permission. No records acquired. [Terms](https://www.thecocktaildb.com/terms_of_use.php). |

Formats, supported languages, identifiers, exact versions/URLs and limits are in the source manifest and workbook. Unpinned/live sources are explicitly identified as such. Rights were reviewed from primary documentation; no legal counsel or independent linguistic certification is claimed.

## Identity and review gates

`collision-review.json` records 13 editorial assessments: three probable duplicate concepts, two peel synonym questions, four distinct form/variety comparisons, three lexical false positives and one rejected brand-specific record. The automated check flags 12 candidates against existing names/aliases; these signals are not equivalence decisions. `candidate-pairs.json` also flags the fresh/unspecified versus dried fenugreek leaf proposals. No automatic merge, source-ID reassignment or record deletion occurs.

All candidate Turkish labels are project-authored proposals, not USDA/FoodOn translations or independently human-approved terminology. Medium-confidence labels and specific concerns are explicit. All human decisions, reviewer identities and review dates remain blank. Missing density, nutrition, allergen and ABV observations remain empty/unknown.

The seven prior decisions are copied unchanged into `prior-review.json` and the unresolved worksheet: chili powder, cultured lowfat buttermilk, endive, acorn squash, deferred tahini, Welsh onion and northern lobster. Tahini `usda-sr-169406` stays excluded. The six already-installed terms remain installed unchanged. None is silently approved.

## Reproduction and review workflow

```sh
# Offline candidate generation/checking from committed extracts:
python3 tools/prepare_m3d2a_candidates.py --check
python3 tools/test_m3d2a_candidates.py
# Optional re-extraction verification from pinned official artifacts:
python3 tools/prepare_m3d2a_candidates.py --archive /path/to/FoodData_Central_sr_legacy_food_csv_2018-04.zip --ontology /path/to/foodon-base.owl --check
# Re-measure release 7 only, in a fresh temporary SQLite profile:
cargo build --locked --offline --manifest-path src-tauri/Cargo.toml --example catalog_import
python3 tools/report_m3d2a_coverage.py
```

The FoodOn artifact URL/hash and USDA archive URL/hash are pinned in `sources.json`. Source extracts are committed for offline review. Checksums prove reproducibility/integrity, not independent source authenticity or semantic correctness. Protected production/release/validation/migration/allowlist checksums are checked before and after generation. This preparation JSON is deliberately not a seed manifest and cannot authorize runtime installation.

Workbook: `docs/INGREDIENT_M3D2A_CANDIDATE_REVIEW.xlsx`, generated by `tools/build_m3d2a_workbook.mjs` using the bundled artifact-tool Node runtime. Four sheets: 66 higher-confidence proposals, 128 other proposals plus seven historical pending rows, actual category coverage, source licensing. Yellow decision cells are editable; filters and frozen headers are present. Blank decision is not approval. A “Kabul önerisi” reviewer choice is only a proposal: source clearance and release validation must still pass. Preserve a reviewed workbook copy before regeneration; the generator does not import human decisions. Never use the unrelated root workbook as generator input or output.

## Exact recommended M3D-2B scope

1. Review the 66 lower-risk USDA proposals first; accept only useful, distinct culinary identities and validated bilingual labels. Resolve or defer the 40 USDA questions independently; pending legacy terminology must not block unrelated accepted records.
2. Keep the rejected commercial identity and three probable duplicates out of additions. Decide peel mapping and compare all candidate pairs; record decisions by candidate ID without merging existing identities.
3. Evaluate FoodOn lineage/attribution per selected term. Clear only a bounded subset with documented rights, or leave all 88 outside production. Do not resolve OFF/TürKomp restrictions by importing via another source.
4. Only then generate release 8 from immutable release 7, new source snapshots and explicitly accepted candidates. Do not promise 194 additions or expand categories solely to inflate counts.
5. Run fresh/release-7 upgrade/idempotency/rollback/source-allowlist, personal/override/favorite/recent/draft/recipe-reference and exact quantity tests; verify Turkish/English/alias selection in the native isolated application. Archive release 7, update coverage/review docs, then commit.

No native GUI, Windows, Intel macOS or signed-release verification is claimed by this preparation task. Runtime implementation is unchanged.
