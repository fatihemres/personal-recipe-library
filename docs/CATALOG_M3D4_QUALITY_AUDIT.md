# M3D-4 Release 10 quality and decision audit

**50 new canonical identities; 795 total (666 food / 129 beverage), 269 aliases, 50 categories.** All 745 Release-9 ingredient objects and every category object are byte-equivalent as parsed data. No previous source/version evidence is rewritten.

The 65 newly screened candidates resolve to 50 ACCEPT, 9 DEFER, 4 EXISTING and 2 REJECT. The 29 earlier Release-9 deferrals and seven historical terminology/identity questions remain separate and unresolved. Automated editorial decisions are not independent human linguistic approval.

The structural audit reports zero errors, zero exact alias collisions and zero new capitalization/translation warnings. Its 37 same-English-base concept groups are review hints, not proven duplicates. New additions have no aliases, nutrition, density, allergen or ABV observations.

## Source and identity boundaries

- USDA Foundation Foods April 2026: 35 selected CC0 descriptors with official FDC IDs, original ZIP SHA-256 `186e988ec542e913f51ef62b86a47758e8cdd0d1dc3889e7b055581f3c09c77a`. This is a different upstream data type and immutable source/version from prior SR Legacy selections.
- Wikidata: 15 individually screened CC0 structured QID revisions. The pinned 24-entity candidate evidence stores exact revision URLs and original response checksums; no external photographs or linked copyrighted text are included.
- Tarif Atlası: bilingual labels, category choices and unit defaults are separately authored curation, not claimed as USDA translations or independently approved terminology.
- FoodOn, Open Food Facts, TürKomp and TheCocktailDB contributed zero bundled records. TÜRKPATENT and Turkish ministry pages were identity references, not redistributed datasets; protected-origin status is not inferred for a user-selected generic ingredient.

## Accepted identities

| Stable key | Turkish editorial label | English editorial label | Categories | Primary source |
| --- | --- | --- | --- | --- |
| `usda-foundation-1104867` | Yapışkan pirinç unu | Glutinous rice flour | flours, international-specialty | [1104867](https://fdc.nal.usda.gov/food-details/1104867/nutrients) |
| `usda-foundation-1999626` | Aslan yelesi mantarı | Lion's mane mushroom | mushrooms, international-specialty | [1999626](https://fdc.nal.usda.gov/food-details/1999626/nutrients) |
| `usda-foundation-1999630` | Şekersiz soya içeceği (raf ömrü uzun) | Soy milk (unsweetened, shelf-stable) | dairy-alternatives, other-beverages | [1999630](https://fdc.nal.usda.gov/food-details/1999630/nutrients) |
| `usda-foundation-2003586` | Tipo 00 buğday unu | Tipo 00 wheat flour | flours, international-specialty | [2003586](https://fdc.nal.usda.gov/food-details/2003586/nutrients) |
| `usda-foundation-2003588` | İri taneli irmik | Coarse semolina | flours, grains | [2003588](https://fdc.nal.usda.gov/food-details/2003588/nutrients) |
| `usda-foundation-2003589` | İnce taneli irmik | Fine semolina | flours, grains | [2003589](https://fdc.nal.usda.gov/food-details/2003589/nutrients) |
| `usda-foundation-2003593` | Beyaz üzüm suyu (konsantreden, raf ömrü uzun) | White grape juice (from concentrate, shelf-stable) | juices | [2003593](https://fdc.nal.usda.gov/food-details/2003593/nutrients) |
| `usda-foundation-2003594` | Turna yemişi suyu (konsantreden, raf ömrü uzun) | Cranberry juice (from concentrate, shelf-stable) | juices | [2003594](https://fdc.nal.usda.gov/food-details/2003594/nutrients) |
| `usda-foundation-2003595` | Kırmızı greyfurt suyu (konsantre değil) | Red grapefruit juice (not from concentrate) | juices | [2003595](https://fdc.nal.usda.gov/food-details/2003595/nutrients) |
| `usda-foundation-2003599` | Kral istiridye mantarı | King oyster mushroom | mushrooms, international-specialty | [2003599](https://fdc.nal.usda.gov/food-details/2003599/nutrients) |
| `usda-foundation-2003601` | Kestane mantarı (crimini) | Crimini mushroom | mushrooms | [2003601](https://fdc.nal.usda.gov/food-details/2003601/nutrients) |
| `usda-foundation-2003604` | Pioppino mantarı | Pioppino mushroom | mushrooms, international-specialty | [2003604](https://fdc.nal.usda.gov/food-details/2003604/nutrients) |
| `usda-foundation-2257045` | Şekersiz badem içeceği (soğuk zincir) | Almond milk (unsweetened, refrigerated) | dairy-alternatives, other-beverages | [2257045](https://fdc.nal.usda.gov/food-details/2257045/nutrients) |
| `usda-foundation-2257046` | Şekersiz yulaf içeceği (soğuk zincir) | Oat milk (unsweetened, refrigerated) | dairy-alternatives, other-beverages | [2257046](https://fdc.nal.usda.gov/food-details/2257046/nutrients) |
| `usda-foundation-2261420` | Badem unu | Almond flour | flours, nuts-seeds | [2261420](https://fdc.nal.usda.gov/food-details/2261420/nutrients) |
| `usda-foundation-2512371` | Amarant unu | Amaranth flour | flours, international-specialty | [2512371](https://fdc.nal.usda.gov/food-details/2512371/nutrients) |
| `usda-foundation-2512372` | Kinoa unu | Quinoa flour | flours, international-specialty | [2512372](https://fdc.nal.usda.gov/food-details/2512372/nutrients) |
| `usda-foundation-2512377` | Manyok unu | Cassava flour | flours, international-specialty | [2512377](https://fdc.nal.usda.gov/food-details/2512377/nutrients) |
| `usda-foundation-2512378` | Karabuğday tanesi | Whole buckwheat grain | grains | [2512378](https://fdc.nal.usda.gov/food-details/2512378/nutrients) |
| `usda-foundation-2515377` | Kestane unu | Chestnut flour | flours, international-specialty | [2515377](https://fdc.nal.usda.gov/food-details/2515377/nutrients) |
| `usda-foundation-2515382` | Hindistan cevizi unu | Coconut flour | flours, international-specialty | [2515382](https://fdc.nal.usda.gov/food-details/2515382/nutrients) |
| `usda-foundation-2644281` | Kuru cannellini fasulyesi | Cannellini beans (dry) | legumes, international-specialty | [2644281](https://fdc.nal.usda.gov/food-details/2644281/nutrients) |
| `usda-foundation-2644286` | Konserve navy fasulyesi (süzülmüş ve durulanmış) | Navy beans (canned, drained and rinsed) | legumes, international-specialty | [2644286](https://fdc.nal.usda.gov/food-details/2644286/nutrients) |
| `usda-foundation-2644287` | Konserve cannellini fasulyesi (süzülmüş ve durulanmış) | Cannellini beans (canned, drained and rinsed) | legumes, international-specialty | [2644287](https://fdc.nal.usda.gov/food-details/2644287/nutrients) |
| `usda-foundation-2710821` | Yabani pirinç (kuru) | Wild rice (dry) | grains, international-specialty | [2710821](https://fdc.nal.usda.gov/food-details/2710821/nutrients) |
| `usda-foundation-2710825` | Siyah pirinç (çiğ) | Black rice (raw) | grains, international-specialty | [2710825](https://fdc.nal.usda.gov/food-details/2710825/nutrients) |
| `usda-foundation-2710829` | Fonio tahılı (kuru) | Fonio grain (dry) | grains, international-specialty | [2710829](https://fdc.nal.usda.gov/food-details/2710829/nutrients) |
| `usda-foundation-2710830` | Horasan buğdayı tanesi (kuru) | Khorasan wheat grain (dry) | grains, international-specialty | [2710830](https://fdc.nal.usda.gov/food-details/2710830/nutrients) |
| `usda-foundation-2710838` | Kırmızı pirinç (çiğ) | Red rice (raw) | grains, international-specialty | [2710838](https://fdc.nal.usda.gov/food-details/2710838/nutrients) |
| `usda-foundation-2727582` | Tomatillo (kabuksuz, çiğ) | Tomatillo (dehusked, raw) | vegetables, international-specialty | [2727582](https://fdc.nal.usda.gov/food-details/2727582/nutrients) |
| `usda-foundation-2727588` | Nar suyu (konsantreden, raf ömrü uzun) | Pomegranate juice (from concentrate, shelf-stable) | juices | [2727588](https://fdc.nal.usda.gov/food-details/2727588/nutrients) |
| `usda-foundation-2727589` | Vişne suyu (konsantreden, raf ömrü uzun) | Tart cherry juice (from concentrate, shelf-stable) | juices | [2727589](https://fdc.nal.usda.gov/food-details/2727589/nutrients) |
| `usda-foundation-2747661` | Jalapeño biberi (çekirdeksiz, çiğ) | Jalapeño pepper (seeded, raw) | vegetables, international-specialty | [2747661](https://fdc.nal.usda.gov/food-details/2747661/nutrients) |
| `usda-foundation-2747662` | Poblano biberi (çekirdeksiz, çiğ) | Poblano pepper (seeded, raw) | vegetables, international-specialty | [2747662](https://fdc.nal.usda.gov/food-details/2747662/nutrients) |
| `usda-foundation-2747663` | Serrano biberi (çekirdeksiz, çiğ) | Serrano pepper (seeded, raw) | vegetables, international-specialty | [2747663](https://fdc.nal.usda.gov/food-details/2747663/nutrients) |
| `wikidata-q1196289` | Doenjang | Doenjang | fermented, sauces, international-specialty | [Q1196289](https://www.wikidata.org/wiki/Special:EntityData/Q1196289.json?revision=2552440448) |
| `wikidata-q12812990` | İsot biberi | Urfa biber | spices, turkish-regional | [Q12812990](https://www.wikidata.org/wiki/Special:EntityData/Q12812990.json?revision=2543281212) |
| `wikidata-q1432724` | Sucuk | Sucuk | meat, turkish-regional | [Q1432724](https://www.wikidata.org/wiki/Special:EntityData/Q1432724.json?revision=2539736104) |
| `wikidata-q16620197` | Kefir mayası taneleri | Kefir grains | fermented, dairy | [Q16620197](https://www.wikidata.org/wiki/Special:EntityData/Q16620197.json?revision=2067936241) |
| `wikidata-q2056399` | Pastırma | Pastırma | meat, turkish-regional | [Q2056399](https://www.wikidata.org/wiki/Special:EntityData/Q2056399.json?revision=2490072075) |
| `wikidata-q2118339` | Tarhana (kuru karışım) | Tarhana (dried mixture) | fermented, turkish-regional | [Q2118339](https://www.wikidata.org/wiki/Special:EntityData/Q2118339.json?revision=2500795983) |
| `wikidata-q3183584` | Tulum peyniri | Tulum cheese | cheeses, turkish-regional | [Q3183584](https://www.wikidata.org/wiki/Special:EntityData/Q3183584.json?revision=2556524328) |
| `wikidata-q318367` | Damla sakızı | Mastic resin | spices, turkish-regional | [Q318367](https://www.wikidata.org/wiki/Special:EntityData/Q318367.json?revision=2537480791) |
| `wikidata-q3668413` | Kesme erişte | Kesme noodles | noodles-pasta, turkish-regional | [Q3668413](https://www.wikidata.org/wiki/Special:EntityData/Q3668413.json?revision=2553117911) |
| `wikidata-q465554` | Mahlep | Mahleb | spices, turkish-regional | [Q465554](https://www.wikidata.org/wiki/Special:EntityData/Q465554.json?revision=2475402700) |
| `wikidata-q478654` | Kaymak | Kaymak | dairy, turkish-regional | [Q478654](https://www.wikidata.org/wiki/Special:EntityData/Q478654.json?revision=2537650278) |
| `wikidata-q6027754` | Çökelek | Çökelek cheese | cheeses, turkish-regional | [Q6027754](https://www.wikidata.org/wiki/Special:EntityData/Q6027754.json?revision=2439800899) |
| `wikidata-q6062076` | Van otlu peyniri | Van herbed cheese | cheeses, turkish-regional | [Q6062076](https://www.wikidata.org/wiki/Special:EntityData/Q6062076.json?revision=1352521981) |
| `wikidata-q67207722` | Demirhindi ezmesi (sulandırılmış) | Tamarind paste (diluted) | sauces, international-specialty | [Q67207722](https://www.wikidata.org/wiki/Special:EntityData/Q67207722.json?revision=2480222626) |
| `wikidata-q699637` | Gochujang | Gochujang | fermented, sauces, international-specialty | [Q699637](https://www.wikidata.org/wiki/Special:EntityData/Q699637.json?revision=2488883660) |

## Excluded or pending candidates

| Source ID | Decision | Rationale |
| --- | --- | --- |
| [1999627](https://fdc.nal.usda.gov/food-details/1999627/nutrients) | DEFER | Existing oyster-mushroom form may cover the descriptor; compare species and preparation before accepting. |
| [1999628](https://fdc.nal.usda.gov/food-details/1999628/nutrients) | EXISTING | Existing raw shiitake mushroom identity covers this form. |
| [2262073](https://fdc.nal.usda.gov/food-details/2262073/nutrients) | DEFER | Sesame butter overlaps historical deferred tahini identity; do not import without identity review. |
| [2647441](https://fdc.nal.usda.gov/food-details/2647441/nutrients) | DEFER | Oaxaca cheese may already be covered by a related Mexican cheese; verify culinary identity and category before adding. |
| [2710827](https://fdc.nal.usda.gov/food-details/2710827/nutrients) | EXISTING | Existing siyez grain identity already covers einkorn wheat. |
| [321358](https://fdc.nal.usda.gov/food-details/321358/nutrients) | REJECT | Commercial prepared hummus is a ready dish rather than a base ingredient for this catalog batch. |
| [Q108946643](https://www.wikidata.org/wiki/Special:EntityData/Q108946643.json?revision=2557959353) | EXISTING | Existing nar ekşisi covers pomegranate reduction; do not create a duplicate. |
| [Q148306](https://www.wikidata.org/wiki/Special:EntityData/Q148306.json?revision=2553152846) | DEFER | Zahter may denote herb or mixed seasoning; entity description is ambiguous. |
| [Q1817099](https://www.wikidata.org/wiki/Special:EntityData/Q1817099.json?revision=2302378824) | REJECT | QID is a music project, not the edible lemongrass ingredient. |
| [Q1866347](https://www.wikidata.org/wiki/Special:EntityData/Q1866347.json?revision=2557231067) | DEFER | Phyllo and regional yufka may have different thickness/use; do not conflate without independent evidence. |
| [Q235169](https://www.wikidata.org/wiki/Special:EntityData/Q235169.json?revision=2517552095) | EXISTING | Existing miso identity covers fermented soybean paste. |
| [Q27943](https://www.wikidata.org/wiki/Special:EntityData/Q27943.json?revision=2556895689) | DEFER | Salicornia species and marketed deniz börülcesi may not be coterminous. |
| [Q3337007](https://www.wikidata.org/wiki/Special:EntityData/Q3337007.json?revision=2535968927) | DEFER | Botanical madımak species does not alone establish edible harvested ingredient form. |
| [Q734727](https://www.wikidata.org/wiki/Special:EntityData/Q734727.json?revision=2558005692) | DEFER | Galangal spans several species; exact culinary rhizome identity requires a narrower source. |
| [Q966327](https://www.wikidata.org/wiki/Special:EntityData/Q966327.json?revision=2511176634) | DEFER | Harissa entity has an identity description but lacks stable English/Turkish labels in this revision. |

Potentially close forms such as canned versus dry beans, concentrated versus fresh juice, coarse versus fine semolina, and regional versus generic dairy remain separate only where source identity and culinary use justify it. Remaining source rights and terminology issues are detailed in `docs/CATALOG_M3D4_SOURCE_EVALUATION.md`.
