# M3D-5 beverage coverage and gap matrix

**Release 11: 854 canonical identities, 666 food / 188 beverage, 269 aliases and 50 categories.** Fifty-nine new beverage-oriented identities were accepted after screening 85 source candidates. All 795 Release-10 identity objects and all category objects remain unchanged.

Direct category memberships overlap and must not be summed into the 188 unique beverage identities.

| Beverage category | Release 10 direct | Release 11 direct | Change |
| --- | ---: | ---: | ---: |
| `coffee` | 7 | 13 | +6 |
| `tea` | 9 | 14 | +5 |
| `infusions` | 2 | 6 | +4 |
| `syrups` | 12 | 19 | +7 |
| `mixers` | 5 | 10 | +5 |
| `bitters` | 1 | 2 | +1 |
| `liqueurs` | 8 | 10 | +2 |
| `spirits` | 25 | 26 | +1 |
| `traditional-drinks` | 4 | 5 | +1 |
| `garnishes` | 6 | 10 | +4 |
| `juices` | 28 | 39 | +11 |
| `wines` | 5 | 17 | +12 |
| `soft-drinks` | 4 | 5 | +1 |
| `other-beverages` | 9 | 14 | +5 |
| `water` | 2 | 2 | +0 |
| `beers` | 7 | 7 | +0 |

The release adds bean and ground coffee forms; matcha, white, pu-erh, jasmine and genmaicha teas; orange bitters, ginger beer, falernum, cordials, syrups and garnishes; several fruit juices and wine varieties; and salep flour as a traditional beverage input. English source identities and original Turkish editorial labels are separate.

**Remaining high-value gaps:** independently verified dry rooibos and herbal materials, more coffee extracts and concentrates, vermouth identities, culturally specific şerbet bases, additional alcohol-free mixers, and non-branded liqueur types. The 16 deferred candidates have explicit reasons in `catalog/preparation/m3d5/candidates.json`; none is in the runtime seed.

This is a real SQLite-backed count. `catalog/production/m3d5-sqlite-report.json` records isolated fresh/upgrade integrity checks and local debug-CLI durations, which include process startup and are not cross-device benchmarks.
