# Production catalog coverage — M3B-2

Version 3 contains **482 identities**, up **247** from version 2 (235). **396 food / 86 beverage**, **252 aliases**, **50 category nodes**. Counts are canonical identities, not unique brands or nutrient records.

443 identities have unchanged USDA IDs/descriptors; all 482 have project-authored labels/classification, including 39 project-only reference-backed identities. No fabricated empirical metadata. Eight validation identities are shared by stable ID and counted once.

Zero missing bilingual labels, zero structural errors, zero exact normalized alias collision candidates, zero rejected accepted-package records. Two selection exclusions are recorded in quality-audit.json (not importer rejections). **17 human terminology review items**, zero independent human reviews. “Production-ready” means validated identity/provenance/import compatibility, not certified language quality.

Direct memberships overlap and do not sum to 482. Root categories can have zero direct members. Food/beverage type is separate from alcohol classification; no alcohol percentages assigned.

| Category | Parent | Turkish name | Direct members |
| --- | --- | --- | ---: |
| baking | food | Un ve fırıncılık | 37 |
| beers | beverage | Biralar | 2 |
| beverage | — | İçecek malzemeleri | 3 |
| bitters | beverage | Kokteyl bitterleri | 1 |
| brandy | spirits | Brandi ve meyve distilatları | 6 |
| cheeses | dairy | Peynirler | 29 |
| chocolate-cocoa | baking | Çikolata ve kakao | 5 |
| coffee | coffee-tea | Kahve | 4 |
| coffee-tea | beverage | Kahve ve çay | 2 |
| condiments | food | Çeşniler | 12 |
| dairy | food | Süt ürünleri | 41 |
| dairy-alternatives | food | Süt alternatifleri | 3 |
| dried-fruit | fruit | Kuru meyveler | 9 |
| eggs | food | Yumurtalar | 7 |
| fermented | food | Fermente gıdalar | 4 |
| flours | baking | Unlar | 22 |
| food | — | Gıda malzemeleri | 0 |
| fruit | food | Meyveler | 47 |
| garnishes | beverage | Süsleme malzemeleri | 6 |
| gin | spirits | Cin | 2 |
| grains | food | Tahıllar | 28 |
| herbs | food | Otlar | 20 |
| infusions | beverage | Bitki demlemeleri | 2 |
| international-specialty | food | Uluslararası özel malzemeler | 7 |
| juices | beverage | Meyve ve sebze suları | 12 |
| legumes | food | Baklagiller | 19 |
| liqueurs | beverage | Likörler | 8 |
| meat | food | Et | 7 |
| milk-powders | dairy | Süt tozları | 2 |
| mixers | beverage | Karışım içecekleri | 3 |
| mushrooms | vegetables | Mantarlar | 8 |
| noodles-pasta | grains | Makarna ve erişte | 9 |
| nuts-seeds | food | Kuruyemiş ve tohumlar | 23 |
| oils | food | Yağlar | 17 |
| other-beverages | beverage | Diğer içecek malzemeleri | 5 |
| poultry | food | Kümes hayvanları | 7 |
| sauces | food | Soslar | 16 |
| seafood | food | Deniz ürünleri | 21 |
| soft-drinks | beverage | Gazlı içecekler | 3 |
| spices | food | Baharatlar | 28 |
| spirits | beverage | Distile alkollü içkiler | 23 |
| sweeteners | food | Tatlandırıcılar | 7 |
| syrups | beverage | Şuruplar | 8 |
| tea | coffee-tea | Çay | 6 |
| traditional-drinks | other-beverages | Geleneksel Türk içecekleri | 4 |
| turkish-regional | food | Yöresel Türk malzemeleri | 10 |
| vegetables | food | Sebzeler | 52 |
| water | beverage | Su ve maden suyu | 2 |
| whisky | spirits | Viski | 8 |
| wines | beverage | Şaraplar | 5 |

Remaining gaps: regional cheeses/ferments, fine/coarse regional bulgur, Turkish pulses and wild culinary plants, rum variants, vodka variants, tequila aging categories, vermouth, beer styles, raw coffee beans/tea leaves, tonic and bar syrup varieties, Mediterranean/Middle Eastern specialty breadth. These are future source/terminology curation tasks, not invented coverage. No new broad catalog search UI (M3C).

Actual database counts are recorded separately in installation-report.json and upgrade-report.json; pending personal collisions can reduce available materialized system rows without deleting identities. See INGREDIENT_LOCALIZATION_REVIEW.md for review questions.
