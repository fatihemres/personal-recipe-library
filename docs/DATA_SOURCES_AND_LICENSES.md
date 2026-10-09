# Ingredient sources and licensing

Reviewed 2026-10-08 against the primary documentation linked below. Machine decisions: `catalog/sources.json`; exact installed source snapshots and evidence hashes: `catalog/validation/manifest.json`. A review date is not a dataset retrieval date. Only USDA SR Legacy and project-authored validation curation are enabled in the seed allowlist. No third-party images are included. An accessible API is not blanket permission to redistribute a database.

## USDA FoodData Central — approved selected factual data

Primary sources: [API guide and licensing](https://fdc.nal.usda.gov/api-guide/), [official bulk downloads](https://fdc.nal.usda.gov/download-datasets/).

FoodData Central factual data are public domain/CC0; redistribution and commercial reuse are permitted under that dedication. USDA requests attribution; retain “USDA Agricultural Research Service. FoodData Central” with dataset/version and source links. These findings concern data, not arbitrary photographs, logos or other site material.

Technically accessible JSON REST API (API key required) and downloadable CSV/JSON archives. Bulk preparation avoids first-launch networking and API quotas. SR Legacy is the final April 2018 release; Foundation April 2026 and FNDDS 2021–2023 (October 2024 release) were observed on the download page. Branded April 2026 is disproportionately large and product-oriented, unsuitable for indiscriminate import.

Actual retrieved artifact, 2026-10-08: `FoodData_Central_sr_legacy_food_csv_2018-04.zip`, SHA-256 `b80817294b8850530aaedf2e515c02593b1824f763a0ff356e5c2081643e6fd0`. Official URL is in the machine manifest. Eight factual IDs/descriptions were extracted from `food.csv`; no nutrition values imported. Foundation is legally suitable but **not enabled** until a concrete artifact/version is downloaded, hashed and reviewed.

Coverage: common generic food forms, some coffee/tea/liquids; not a complete Turkish regional or cocktail ingredient vocabulary. English technical descriptions need authored consumer-facing Turkish names. Food forms and preparation states must be reviewed; nutrients vary by sampled food/product and do not establish arbitrary brand values or density. An old fixed release is reproducible, not necessarily current nutritional evidence.

## FoodOn — conditional candidate, excluded from this seed

Primary sources: [repository](https://github.com/FoodOntology/foodon), [actual LICENSE.txt](https://github.com/FoodOntology/foodon/blob/master/LICENSE.txt), [OBO registry](https://obofoundry.org/ontology/foodon.html), [project](https://foodon.org/).

FoodOn's own ontology is CC BY 4.0: commercial reuse and redistribution with attribution, license link and indication of changes; no implication of endorsement. OWL/RDF releases and repository synonym TSV offer stable ontology identifiers, generic food/material/process relationships and English labels/synonyms. Imported ontology dependencies must be audited by term/artifact; FoodOn's own license is not a blanket clearance for all imported material. No image rights inferred.

No release artifact was retrieved or pinned in M3A; retrieval/version fields are null. Before enabling: pin a release/commit, checksum its artifact, review imported terms/licenses and produce attribution/change notices. Scientific hierarchy is not suitable for direct consumer navigation; map selected terms to curated culinary categories. Turkish coverage requires independent curation. Candidate for identity mapping, not a replacement for a useful culinary catalog.

## Open Food Facts — excluded from combined bundled seed

Primary sources: [official license guide](https://openfoodfacts.github.io/documentation/docs/Product-Opener/api/tutorials/license-be-on-the-legal-side/), [official API documentation source](https://github.com/openfoodfacts/openfoodfacts-server/blob/main/docs/api/index.md), [data page](https://world.openfoodfacts.org/data).

Database: ODbL 1.0. Individual database contents: DbCL 1.0. Database attribution/share-alike obligations require analysis of derivative databases, public distribution and offered database access before incorporation. Merely separating tables or calling material “canonical” does not settle compatibility. Commercial use is possible under applicable conditions, not an unconditional permission for this combined seed.

Product images have separate CC BY-SA conditions; exact image license/version, photographer attribution and third-party packaging/logo rights must be assessed before any asset reuse. No images approved here.

JSON product API and bulk data exports (including tabular exports) are documented; full product data is large. Multilingual community-submitted branded/barcoded products can help a later separate product module, but are not generic ingredient identities. Coverage and completeness vary, errors/duplicate brands occur, and missing allergens/nutrition are unknown. The public data/terms pages returned indexing restrictions during this review; no scraping or bypass was attempted. No dump downloaded/version pinned. Keep excluded until a documented compliant distribution strategy is approved.

## TheCocktailDB — excluded until applicable offline rights are resolved

Primary sources: [API documentation](https://www.thecocktaildb.com/api.php), [FAQ](https://www.thecocktaildb.com/faq.php), [linked terms](https://www.thecocktaildb.com/terms_of_use.php).

JSON drink/ingredient endpoints, development key and supporter/production access conditions are documented. Useful English beverage, spirit, mixer and garnish labels; incomplete quantities/ABV and inconsistent generic/product distinctions need review. API access conditions do not establish unrestricted full-database offline redistribution or image reuse.

The linked terms page identifies **TheMealDB** and displays “01/07/2025”; applicability to TheCocktailDB and an offline redistributed desktop catalog is unclear. Terms discuss API use and restrictions, not a clear blanket database dedication. FAQ/API production conditions include supporter access. Do not scrape. No dataset/images retrieved or pinned. Obtain explicit applicable database redistribution/commercial rights and separate image terms before enabling; this milestone does not contact the operator.

## Curation and release gates

Project-authored validation factual labels/classifications are CC0 as stated in `catalog/validation/NOTICE.md`; source mappings remain separately attributed. Translation choices are project curation, never fabricated upstream metadata. Stable identities and culinary forms are reviewed independently of source IDs. Missing density, allergens, ABV and nutrients remain absent.

M3B must set a representative food/beverage/Turkish coverage matrix, pin every approved artifact, preserve source evidence and generate actual counts. USDA alone does not provide complete spirit/liqueur/bitters/regional coverage; find lawful documented alternatives or author verified factual curation with evidence. Do not silently remove these V1 families because one source is excluded. Signoff on rights and attribution is required before redistribution; these technical findings do not substitute for resolving uncertain license applications.
