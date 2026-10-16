# M3D-2B source decisions and future expansion

Release 8 uses only the pinned **USDA FoodData Central SR Legacy April 2018** CSV archive. Its 79 selected `fdc_id` descriptors are public-domain/CC0 source facts; the application retains USDA attribution. The English display labels follow the descriptors. Turkish names, category assignment and unit defaults are project editorial work under the existing project CC0 dedication, not USDA translations or individually human-approved language. The official [USDA licensing statement](https://fdc.nal.usda.gov/api-guide/#licensing) and [April 2018 download listing](https://fdc.nal.usda.gov/download-datasets/) support this bounded use. The pinned archive SHA-256 remains `b80817294b8850530aaedf2e515c02593b1824f763a0ff356e5c2081643e6fd0`.

New immutable source versions are `usda-sr-legacy-m3d2b/2018-04-selection-6` and `recipeatlas-curation-m3d2b/production-5`. `catalog/production/m3d2b-provenance-report.json` maps every addition to its source URL and records all package provenance links. Release 7 and all older source snapshots remain archived. No nutrients, density, allergen status or ABV values were inferred from descriptors.

## Sources outside release 8

- **FoodOn:** The repository declares CC BY 4.0, but individual terms can have imported lineage. All 88 prepared terms remain outside production until each selected term's source lineage, attribution, changed-label notice and offline redistribution rights are checked. Three are probable existing concepts and one is a branded cheese name; neither becomes a new identity. The [official repository](https://github.com/FoodOntology/foodon) and pinned M3D-2A ontology evidence remain research material.
- **Open Food Facts:** ODbL/DbCL derivative-database and combination obligations remain unresolved for this bundle; images carry separate rights. No product records or images were copied. See the [official license guidance](https://openfoodfacts.github.io/documentation/docs/Product-Opener/api/tutorials/license-be-on-the-legal-side/).
- **TürKomp:** The [official usage terms](https://turkomp.tarimorman.gov.tr/useofdata) require permission and separate commercial arrangements. No data were copied. Future use requires an explicit license suitable for offline redistribution and software use.
- **TheCocktailDB:** API accessibility does not establish rights to redistribute an offline ingredient database; current applicable terms remain unclear. No records were copied. Seek written applicable terms or use independently cleared identity references.
- **Wikidata:** Structured data may be CC0, but a future batch must pin QIDs/revisions, verify culinary identity and Turkish labels, and avoid copying separately licensed prose. No Wikidata records were selected for release 8.
- **TÜRKPATENT and other regional references:** Use only for narrow identity/terminology checks until bulk redistribution rights are established. A geographical indication is not proof that a generic catalog item satisfies its product specification.

## Next controlled source work

1. Address 27 deferred USDA identity/terminology questions individually; neither their exclusion nor the 79 accepted decisions changes the seven prior pending items.
2. For useful FoodOn concepts, inspect each term's imported xrefs and source lineage, retain CC BY attribution/change records, then compare against installed canonical IDs. Clear only the specific records whose rights and identities are established.
3. Target the measured gaps: Turkish regional dairy/ferments/pastry/plants; South and Southeast Asian herbs and forms; Middle Eastern/North African condiments; non-alcoholic coffee/tea/infusions; generic bar mixers, liqueurs and bitters. Prefer stable, reusable source data and documented project-authored curation over large branded-product dumps.
4. Before any later release, freeze source artifacts and their hashes, review every new identity and translation, check collisions in both languages, archive the previous release, then test fresh installation, upgrade, rollback and preservation in isolated SQLite/native profiles.

These are research gates, not permission to copy unresolved datasets or to claim a catalog of thousands of verified ingredients today.
