# M3D-3 Release 9 quality and decision audit

**98 new canonical identities; 745 total (624 food / 121 beverage), 269 aliases, 50 categories.** All 647 Release-8 ingredient objects and every category object are preserved exactly. No prior source snapshot is rewritten. The 98 additions carry pinned USDA descriptors plus separate project-authored bilingual labels. No factual nutrition, density, allergen or ABV metadata was inferred; no human linguistic approval is claimed.

The reproducible candidate ledger has **131 decisions**: 98 ACCEPT, 29 DEFER (27 carried from M3D-2B), 3 EXISTING and 1 REJECT. The static audit reports zero structural errors and zero exact alias collisions; it lists possible English-base concept groups for review, which are not proven duplicates. Seven prior terminology/identity items remain unresolved. No aliases or category nodes were added.

## Source decision

- **USDA FoodData Central SR Legacy, April 2018:** pinned CSV archive SHA-256 `b80817294b8850530aaedf2e515c02593b1824f763a0ff356e5c2081643e6fd0`; official CC0/public-domain terms permit offline redistribution and commercial reuse. USDA descriptors and FDC IDs are preserved in `catalog/preparation/m3d3/source-records.json`. Project label curation is separate.
- **FoodOn:** term-level lineage/attribution and downstream redistribution have not been cleared; no term imported. **Wikidata:** CC0 identity direction considered, but no pinned QID/revision extraction prepared. **Open Food Facts, TürKomp, TheCocktailDB:** excluded under the standing source restrictions. These exclusions are scope/rights decisions, not assertions that every possible record is unsuitable.

## Accepted source-backed identities

| Stable key | Turkish editorial label | English editorial label | Categories | Source |
| --- | --- | --- | --- | --- |
| `usda-sr-167722` | Tofu yoğurdu | Tofu yogurt | dairy-alternatives, fermented | [FDC](https://fdc.nal.usda.gov/food-details/167722/nutrients) |
| `usda-sr-167786` | Konserve demirhindi nektarı | Canned tamarind nectar | juices | [FDC](https://fdc.nal.usda.gov/food-details/167786/nutrients) |
| `usda-sr-167812` | Çiğ domuz göbeği | Pork belly (raw) | meat | [FDC](https://fdc.nal.usda.gov/food-details/167812/nutrients) |
| `usda-sr-168147` | Vital buğday gluteni | Vital wheat gluten | baking, flours | [FDC](https://fdc.nal.usda.gov/food-details/168147/nutrients) |
| `usda-sr-168190` | Şurupta konserve jackfruit | Canned jackfruit in syrup | fruit | [FDC](https://fdc.nal.usda.gov/food-details/168190/nutrients) |
| `usda-sr-168410` | Dondurulmuş edamame (hazırlanmamış) | Frozen edamame (unprepared) | legumes, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/168410/nutrients) |
| `usda-sr-168607` | Çiğ dana döşü (yağsız) | Beef brisket (lean, raw) | meat | [FDC](https://fdc.nal.usda.gov/food-details/168607/nutrients) |
| `usda-sr-168652` | Yağ oranı %30 çiğ dana kıyması | Ground beef (70% lean, raw) | meat | [FDC](https://fdc.nal.usda.gov/food-details/168652/nutrients) |
| `usda-sr-168751` | Vanilya aromalı şekerli badem sütü | Sweetened vanilla almond milk | dairy-alternatives | [FDC](https://fdc.nal.usda.gov/food-details/168751/nutrients) |
| `usda-sr-168886` | Orta öğütülmüş çavdar unu | Medium rye flour | flours | [FDC](https://fdc.nal.usda.gov/food-details/168886/nutrients) |
| `usda-sr-168894` | Ağartılmış çok amaçlı beyaz buğday unu | Bleached all-purpose wheat flour | flours | [FDC](https://fdc.nal.usda.gov/food-details/168894/nutrients) |
| `usda-sr-168895` | Kendiliğinden kabaran beyaz buğday unu | Self-rising white wheat flour | flours | [FDC](https://fdc.nal.usda.gov/food-details/168895/nutrients) |
| `usda-sr-168896` | Beyaz ekmeklik buğday unu | White bread wheat flour | flours | [FDC](https://fdc.nal.usda.gov/food-details/168896/nutrients) |
| `usda-sr-169207` | Süzülmüş konserve kuşkonmaz | Canned asparagus (drained) | vegetables | [FDC](https://fdc.nal.usda.gov/food-details/169207/nutrients) |
| `usda-sr-169212` | Süzülmüş konserve bambu filizi | Canned bamboo shoots (drained) | vegetables, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/169212/nutrients) |
| `usda-sr-169282` | Yeşil soya fasulyesi (çiğ) | Green soybeans (raw) | legumes | [FDC](https://fdc.nal.usda.gov/food-details/169282/nutrients) |
| `usda-sr-169284` | Soya filizi (çiğ) | Raw soybean sprouts | legumes | [FDC](https://fdc.nal.usda.gov/food-details/169284/nutrients) |
| `usda-sr-169695` | Rüşeymi alınmış sarı mısır unu | Degermed yellow corn flour | flours | [FDC](https://fdc.nal.usda.gov/food-details/169695/nutrients) |
| `usda-sr-169697` | Tam tahıllı sarı mısır irmiği | Whole-grain yellow cornmeal | grains | [FDC](https://fdc.nal.usda.gov/food-details/169697/nutrients) |
| `usda-sr-169701` | Konserve beyaz hominy mısırı | Canned white hominy | grains, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/169701/nutrients) |
| `usda-sr-169713` | Pirinç kepeği | Rice bran | grains | [FDC](https://fdc.nal.usda.gov/food-details/169713/nutrients) |
| `usda-sr-169717` | Kuru tapiyoka incileri | Dry tapioca pearls | baking, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/169717/nutrients) |
| `usda-sr-169723` | Beyaz pastalık buğday unu | White cake wheat flour | flours | [FDC](https://fdc.nal.usda.gov/food-details/169723/nutrients) |
| `usda-sr-169725` | Filizlendirilmiş buğday | Sprouted wheat | grains | [FDC](https://fdc.nal.usda.gov/food-details/169725/nutrients) |
| `usda-sr-169745` | Spelt buğdayı (pişmemiş) | Spelt wheat (uncooked) | grains | [FDC](https://fdc.nal.usda.gov/food-details/169745/nutrients) |
| `usda-sr-169747` | Teff (pişmemiş) | Teff (uncooked) | grains, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/169747/nutrients) |
| `usda-sr-169766` | Konserve acı biber turşusu | Canned pickled hot peppers | condiments | [FDC](https://fdc.nal.usda.gov/food-details/169766/nutrients) |
| `usda-sr-169890` | Japon usulü taze lahana turşusu | Japanese-style fresh pickled cabbage | condiments, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/169890/nutrients) |
| `usda-sr-169892` | Patlıcan turşusu | Pickled eggplant | condiments | [FDC](https://fdc.nal.usda.gov/food-details/169892/nutrients) |
| `usda-sr-169957` | Maş fasulyesi filizi (çiğ) | Raw mung bean sprouts | legumes, vegetables | [FDC](https://fdc.nal.usda.gov/food-details/169957/nutrients) |
| `usda-sr-169966` | Süzülmüş konserve pancar | Canned beets (drained) | vegetables | [FDC](https://fdc.nal.usda.gov/food-details/169966/nutrients) |
| `usda-sr-170071` | Yam kökü (çiğ) | Yam (raw) | vegetables, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/170071/nutrients) |
| `usda-sr-170073` | Jicama (çiğ) | Jicama (raw) | vegetables, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/170073/nutrients) |
| `usda-sr-170106` | Kırmızı acı chili biberi (çiğ) | Hot red chili pepper (raw) | vegetables | [FDC](https://fdc.nal.usda.gov/food-details/170106/nutrients) |
| `usda-sr-170152` | Tuzsuz kavrulmuş susam içi | Unsalted toasted sesame kernels | nuts-seeds | [FDC](https://fdc.nal.usda.gov/food-details/170152/nutrients) |
| `usda-sr-170155` | Tuzsuz ayçiçeği çekirdeği ezmesi | Unsalted sunflower seed butter | nuts-seeds | [FDC](https://fdc.nal.usda.gov/food-details/170155/nutrients) |
| `usda-sr-170171` | Konserve şekerli Hindistan cevizi kreması | Canned sweetened coconut cream | dairy-alternatives | [FDC](https://fdc.nal.usda.gov/food-details/170171/nutrients) |
| `usda-sr-170190` | Kavrulmuş Avrupa kestanesi | Roasted European chestnuts | nuts-seeds | [FDC](https://fdc.nal.usda.gov/food-details/170190/nutrients) |
| `usda-sr-170289` | Ham mısır kepeği | Crude corn bran | grains | [FDC](https://fdc.nal.usda.gov/food-details/170289/nutrients) |
| `usda-sr-170375` | Pancar yaprağı (çiğ) | Beet greens (raw) | vegetables | [FDC](https://fdc.nal.usda.gov/food-details/170375/nutrients) |
| `usda-sr-170392` | Kimchi lahana turşusu | Cabbage kimchi | fermented, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/170392/nutrients) |
| `usda-sr-170480` | Konserve pancar turşusu | Canned pickled beets | condiments | [FDC](https://fdc.nal.usda.gov/food-details/170480/nutrients) |
| `usda-sr-170497` | Yeşil acı chili biberi (çiğ) | Hot green chili pepper (raw) | vegetables | [FDC](https://fdc.nal.usda.gov/food-details/170497/nutrients) |
| `usda-sr-170500` | Kurutulmuş havuç | Dehydrated carrot | vegetables | [FDC](https://fdc.nal.usda.gov/food-details/170500/nutrients) |
| `usda-sr-170566` | Tam yağlı meşe palamudu unu | Full-fat acorn flour | flours, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/170566/nutrients) |
| `usda-sr-170579` | Kavrulmuş kuru Hindistan cevizi eti | Toasted desiccated coconut meat | nuts-seeds | [FDC](https://fdc.nal.usda.gov/food-details/170579/nutrients) |
| `usda-sr-170582` | Zarı soyulmuş fındık | Blanched hazelnuts | nuts-seeds | [FDC](https://fdc.nal.usda.gov/food-details/170582/nutrients) |
| `usda-sr-170590` | Kuru pili fıstığı | Dried pili nuts | nuts-seeds, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/170590/nutrients) |
| `usda-sr-170592` | Kuru pinyon çam fıstığı | Dried pinyon pine nuts | nuts-seeds | [FDC](https://fdc.nal.usda.gov/food-details/170592/nutrients) |
| `usda-sr-170846` | Düşük nemli tam yağlı mozzarella | Low-moisture whole-milk mozzarella | cheeses | [FDC](https://fdc.nal.usda.gov/food-details/170846/nutrients) |
| `usda-sr-170886` | Sade az yağlı yoğurt | Plain low-fat yogurt | dairy | [FDC](https://fdc.nal.usda.gov/food-details/170886/nutrients) |
| `usda-sr-170887` | Sade yağsız yoğurt | Plain skim-milk yogurt | dairy | [FDC](https://fdc.nal.usda.gov/food-details/170887/nutrients) |
| `usda-sr-171015` | Palm yağı | Palm oil | oils | [FDC](https://fdc.nal.usda.gov/food-details/171015/nutrients) |
| `usda-sr-171043` | Ballı hardal sosu | Honey mustard dressing | condiments | [FDC](https://fdc.nal.usda.gov/food-details/171043/nutrients) |
| `usda-sr-171248` | Yarım yağlı sütten ricotta peyniri | Part-skim ricotta cheese | cheeses | [FDC](https://fdc.nal.usda.gov/food-details/171248/nutrients) |
| `usda-sr-171425` | Çay tohumu yağı | Tea seed oil | oils | [FDC](https://fdc.nal.usda.gov/food-details/171425/nutrients) |
| `usda-sr-171580` | Raf ömürlü hazır pesto sosu | Shelf-stable pesto sauce | sauces | [FDC](https://fdc.nal.usda.gov/food-details/171580/nutrients) |
| `usda-sr-171604` | Konserve kırmızı acı biber sosu | Canned red hot chili sauce | sauces | [FDC](https://fdc.nal.usda.gov/food-details/171604/nutrients) |
| `usda-sr-171891` | Espresso kahve (hazırlanmış) | Prepared espresso coffee | coffee | [FDC](https://fdc.nal.usda.gov/food-details/171891/nutrients) |
| `usda-sr-171911` | Şekersiz hazır yeşil çay (içecek) | Unsweetened ready-to-drink green tea | tea | [FDC](https://fdc.nal.usda.gov/food-details/171911/nutrients) |
| `usda-sr-172201` | Queso añejo peyniri | Queso añejo cheese | cheeses, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/172201/nutrients) |
| `usda-sr-172222` | Queso seco peyniri | Queso seco cheese | cheeses, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/172222/nutrients) |
| `usda-sr-172234` | Sarı hardal (hazır) | Prepared yellow mustard | condiments | [FDC](https://fdc.nal.usda.gov/food-details/172234/nutrients) |
| `usda-sr-172239` | Kuru yeşil nane | Dried spearmint | herbs | [FDC](https://fdc.nal.usda.gov/food-details/172239/nutrients) |
| `usda-sr-172242` | Sazon baharat karışımı (kişniş ve annatto) | Sazon seasoning (coriander and annatto) | spices, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/172242/nutrients) |
| `usda-sr-172243` | Taco baharat karışımı (kuru) | Dry taco seasoning mix | spices, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/172243/nutrients) |
| `usda-sr-172337` | Hardal yağı | Mustard oil | oils | [FDC](https://fdc.nal.usda.gov/food-details/172337/nutrients) |
| `usda-sr-172410` | Çiğ ördek eti (derisiz) | Duck meat (skinless, raw) | poultry | [FDC](https://fdc.nal.usda.gov/food-details/172410/nutrients) |
| `usda-sr-172421` | Tuzsuz haşlanmış mercimek | Boiled lentils (without salt) | legumes | [FDC](https://fdc.nal.usda.gov/food-details/172421/nutrients) |
| `usda-sr-173045` | Çilek guavası (çiğ) | Strawberry guava (raw) | fruit | [FDC](https://fdc.nal.usda.gov/food-details/173045/nutrients) |
| `usda-sr-173187` | Raf ömürlü şekersiz çikolatalı badem sütü | Unsweetened shelf-stable chocolate almond milk | dairy-alternatives, chocolate-cocoa | [FDC](https://fdc.nal.usda.gov/food-details/173187/nutrients) |
| `usda-sr-173216` | Dondurulmuş limonata konsantresi | Frozen lemonade concentrate | mixers | [FDC](https://fdc.nal.usda.gov/food-details/173216/nutrients) |
| `usda-sr-173415` | Cheshire peyniri | Cheshire cheese | cheeses | [FDC](https://fdc.nal.usda.gov/food-details/173415/nutrients) |
| `usda-sr-173416` | Colby peyniri | Colby cheese | cheeses | [FDC](https://fdc.nal.usda.gov/food-details/173416/nutrients) |
| `usda-sr-173433` | Yarı yumuşak keçi peyniri | Semisoft goat cheese | cheeses | [FDC](https://fdc.nal.usda.gov/food-details/173433/nutrients) |
| `usda-sr-173437` | Queso asadero peyniri | Queso asadero cheese | cheeses, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/173437/nutrients) |
| `usda-sr-173438` | Queso Chihuahua peyniri | Queso Chihuahua cheese | cheeses, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/173438/nutrients) |
| `usda-sr-173470` | Taze kekik | Fresh thyme | herbs | [FDC](https://fdc.nal.usda.gov/food-details/173470/nutrients) |
| `usda-sr-173471` | Vanilya özütü | Vanilla extract | baking | [FDC](https://fdc.nal.usda.gov/food-details/173471/nutrients) |
| `usda-sr-173472` | Hazır yaban turpu | Prepared horseradish | condiments | [FDC](https://fdc.nal.usda.gov/food-details/173472/nutrients) |
| `usda-sr-173473` | Taze biberiye | Fresh rosemary | herbs | [FDC](https://fdc.nal.usda.gov/food-details/173473/nutrients) |
| `usda-sr-173474` | Taze biber nanesi | Fresh peppermint | herbs | [FDC](https://fdc.nal.usda.gov/food-details/173474/nutrients) |
| `usda-sr-173476` | Chili baharat karışımı (kuru) | Dry chili seasoning mix | spices | [FDC](https://fdc.nal.usda.gov/food-details/173476/nutrients) |
| `usda-sr-173754` | Konserve bakla | Canned fava beans | legumes | [FDC](https://fdc.nal.usda.gov/food-details/173754/nutrients) |
| `usda-sr-173801` | Süzülüp yıkanmış konserve nohut | Canned chickpeas (drained and rinsed) | legumes | [FDC](https://fdc.nal.usda.gov/food-details/173801/nutrients) |
| `usda-sr-174066` | Hazır tatlı-ekşi sos | Ready-to-serve sweet and sour sauce | sauces | [FDC](https://fdc.nal.usda.gov/food-details/174066/nutrients) |
| `usda-sr-174125` | Kafeinsiz espresso kahve (hazırlanmış) | Prepared decaffeinated espresso | coffee | [FDC](https://fdc.nal.usda.gov/food-details/174125/nutrients) |
| `usda-sr-174144` | Hazır siyah çay (içecek) | Ready-to-drink black tea | tea | [FDC](https://fdc.nal.usda.gov/food-details/174144/nutrients) |
| `usda-sr-174249` | Pişmiş ahtapot | Cooked octopus | seafood | [FDC](https://fdc.nal.usda.gov/food-details/174249/nutrients) |
| `usda-sr-174252` | İri lima fasulyesi (çiğ) | Large lima beans (raw) | legumes | [FDC](https://fdc.nal.usda.gov/food-details/174252/nutrients) |
| `usda-sr-174255` | Küçük lima fasulyesi (çiğ) | Baby lima beans (raw) | legumes | [FDC](https://fdc.nal.usda.gov/food-details/174255/nutrients) |
| `usda-sr-174285` | Süzülmüş konserve kırmızı kidney fasulyesi | Canned red kidney beans (drained) | legumes | [FDC](https://fdc.nal.usda.gov/food-details/174285/nutrients) |
| `usda-sr-174286` | Süzülmüş konserve pinto fasulyesi | Canned pinto beans (drained) | legumes | [FDC](https://fdc.nal.usda.gov/food-details/174286/nutrients) |
| `usda-sr-174687` | Jackfruit (çiğ) | Jackfruit (raw) | fruit, international-specialty | [FDC](https://fdc.nal.usda.gov/food-details/174687/nutrients) |
| `usda-sr-174820` | Çikolatalı badem sütü (hazır içecek) | Chocolate almond milk (ready to drink) | dairy-alternatives | [FDC](https://fdc.nal.usda.gov/food-details/174820/nutrients) |
| `usda-sr-174858` | Limonata tozu | Lemonade powder | mixers | [FDC](https://fdc.nal.usda.gov/food-details/174858/nutrients) |
| `usda-sr-175140` | Domates soslu konserve Pasifik sardalyası | Canned Pacific sardines in tomato sauce | seafood | [FDC](https://fdc.nal.usda.gov/food-details/175140/nutrients) |
| `usda-sr-175290` | Çiğ süt danası kıyması | Ground veal (raw) | meat | [FDC](https://fdc.nal.usda.gov/food-details/175290/nutrients) |

## Non-imported candidates

| Source ID | Decision | Reason |
| --- | --- | --- |
| [USDA 167749](https://fdc.nal.usda.gov/food-details/167749/nutrients) | DEFER | Kabuk ile mevcut limon zestesi ayrı mı incelenmeli. |
| [USDA 167759](https://fdc.nal.usda.gov/food-details/167759/nutrients) | DEFER | Yerleşik Türkçe ortak ad incelenmeli. |
| [USDA 167760](https://fdc.nal.usda.gov/food-details/167760/nutrients) | DEFER | Mamey elmasıyla aynı meyve olarak ele alınmamalı. |
| [USDA 167761](https://fdc.nal.usda.gov/food-details/167761/nutrients) | DEFER | Türkçe ad ve soursop karşılığı incelenmeli. |
| [USDA 168148](https://fdc.nal.usda.gov/food-details/168148/nutrients) | DEFER | Mevcut hiyerarşide deniz ürünleri yerine ayrı sınıflama gerekebilir. |
| [USDA 168150](https://fdc.nal.usda.gov/food-details/168150/nutrients) | DEFER | Türkçe ortak ad incelenmeli. |
| [USDA 168152](https://fdc.nal.usda.gov/food-details/168152/nutrients) | DEFER | Kaynak tanımındaki fresh/dried çelişkisi doğrulanmalı. |
| [USDA 168170](https://fdc.nal.usda.gov/food-details/168170/nutrients) | DEFER | Çiçek çanağı mı başka bitki kısmı mı doğrulanmalı. |
| [USDA 168192](https://fdc.nal.usda.gov/food-details/168192/nutrients) | DEFER | Tek kaynak kaydında iki saklama formu var. |
| [USDA 168419](https://fdc.nal.usda.gov/food-details/168419/nutrients) | DEFER | Molehiya ile otomatik eşleştirilmemeli; tür doğrulanmalı. |
| [USDA 168453](https://fdc.nal.usda.gov/food-details/168453/nutrients) | DEFER | Daikon ile form ilişkisi doğrulanmalı. |
| [USDA 168454](https://fdc.nal.usda.gov/food-details/168454/nutrients) | DEFER | Şalgam köküyle eşdeğerlik iddiası yok; Türkçe ad incelenmeli. |
| [USDA 168491](https://fdc.nal.usda.gov/food-details/168491/nutrients) | DEFER | Yenilebilir tür ve Türkçe ad incelenmeli. |
| [USDA 168571](https://fdc.nal.usda.gov/food-details/168571/nutrients) | DEFER | Botanik olarak gövde; Türkçe mutfak adı incelenmeli. |
| [USDA 168581](https://fdc.nal.usda.gov/food-details/168581/nutrients) | DEFER | Mevcut odun kulağı mantarıyla tür/form karşılaştırması gerekli. |
| [USDA 168585](https://fdc.nal.usda.gov/food-details/168585/nutrients) | DEFER | Sıvı coconut cream ile aynı form değil; creamed coconut ayrımı. |
| [USDA 169103](https://fdc.nal.usda.gov/food-details/169103/nutrients) | DEFER | Mevcut portakal zestesiyle form ilişkisi incelenmeli. |
| [USDA 169244](https://fdc.nal.usda.gov/food-details/169244/nutrients) | DEFER | Lambsquarters tür/ad eşleşmesi incelenmeli. |
| [USDA 169277](https://fdc.nal.usda.gov/food-details/169277/nutrients) | DEFER | Scorzonera ile otomatik eşleştirilmemeli. |
| [USDA 169765](https://fdc.nal.usda.gov/food-details/169765/nutrients) | DEFER | Artificially sweetened pickled ginger is narrow and its naming is not yet reviewed. |
| [USDA 169981](https://fdc.nal.usda.gov/food-details/169981/nutrients) | DEFER | Türkçe şevketibostan adıyla otomatik eşleştirilmemeli. |
| [USDA 169990](https://fdc.nal.usda.gov/food-details/169990/nutrients) | DEFER | Türkçe mutfak terimi incelenmeli. |
| [USDA 169993](https://fdc.nal.usda.gov/food-details/169993/nutrients) | DEFER | Mevcut endivyen kararını çözmez; kök kimliği ayrıca incelenmeli. |
| [USDA 170172](https://fdc.nal.usda.gov/food-details/170172/nutrients) | EXISTING | Raw coconut milk is already represented by this exact USDA source ID in release 8. |
| [USDA 170481](https://fdc.nal.usda.gov/food-details/170481/nutrients) | DEFER | Kaynakta yenilen bitki kısmı net değil. |
| [USDA 170483](https://fdc.nal.usda.gov/food-details/170483/nutrients) | DEFER | Drumstick pod için doğal Türkçe terim incelenmeli. |
| [USDA 170688](https://fdc.nal.usda.gov/food-details/170688/nutrients) | EXISTING | Dry bulgur is already represented by this exact USDA source ID in release 8. |
| [USDA 171823](https://fdc.nal.usda.gov/food-details/171823/nutrients) | DEFER | Ördek eti içerdiği anlamına gelmez; doğal Türkçe ad incelenmeli. |
| [USDA 171882](https://fdc.nal.usda.gov/food-details/171882/nutrients) | DEFER | Clam için midye/genel çift kabuklu ayrımı incelenmeli. |
| [USDA 171893](https://fdc.nal.usda.gov/food-details/171893/nutrients) | EXISTING | Instant coffee powder is already represented by this exact USDA source ID in release 8. |
| [USDA 171976](https://fdc.nal.usda.gov/food-details/171976/nutrients) | DEFER | Clam species and natural Turkish culinary name need review; do not label it scallop. |
| [USDA 172026](https://fdc.nal.usda.gov/food-details/172026/nutrients) | REJECT | Branded cooked pasta product is not a generic canonical ingredient. |
| [USDA 174811](https://fdc.nal.usda.gov/food-details/174811/nutrients) | DEFER | Hazır kokteyl değil; karışımın alkol durumu bilinmiyor. |

The seven earlier unresolved decisions remain in the historical review artifacts. Automated decisions here are auditable editorial screening, not expert or individual human linguistic verification. The direct category counts, isolated SQLite imports and upgrade measurements are in `docs/CATALOG_M3D3_COVERAGE.md` and `catalog/production/m3d3-sqlite-report.json`.
