# M3C-5 catalog terminology quality report

Release 6 contains **508 stable identities, 260 aliases and 50 categories**. The package changes exactly two canonical display names; it retains all other release-5 ingredient records unchanged. No independent expert linguistic verification is claimed.

| Key | Field | Previous | User-authorized value |
| --- | --- | --- | --- |
| `usda-sr-174842` | tr | Soda (club soda) | Club soda (kokteyl sodası) |
| `project-siyez-grain` | en | Siyez einkorn wheat | Einkorn wheat (siyez) |

Three earlier M3C-4 name choices remain applied. The following ten records retain their current Turkish names under the user's limited implementation decision; their earlier AI suggestions are not independent expert verification:

| Key | Retained Turkish name |
| --- | --- |
| `usda-sr-171329` | Paprika |
| `usda-sr-170927` | Öğütülmüş muskat çiçeği |
| `usda-sr-170936` | Öğütülmüş sater otu |
| `usda-sr-171318` | Kuru Frenk maydanozu |
| `usda-sr-170160` | Badem ezmesi (pastacılık) |
| `usda-sr-170903` | Sade az yağlı Yunan tipi yoğurt |
| `usda-sr-175142` | Levrek (karışık türler, çiğ) |
| `project-salep-drink` | Salep (hazırlanmış içecek) |
| `project-aromatic-bitters` | Aromatik kokteyl bitteri |
| `project-creme-de-cassis` | Crème de cassis |

Three items remain unresolved and unchanged:

| Key | Current Turkish name |
| --- | --- |
| `usda-sr-171319` | Chili tozu |
| `usda-sr-170874` | Fermente az yağlı buttermilk |
| `usda-sr-168412` | Hindiba (endivyen, çiğ) |

The structural audit reports zero structural errors and zero exact alias collisions. Its 25 possible concept groups remain candidate comparisons only. Names, recipe references, user overrides and personal ingredients must be checked through SQLite upgrade tests; this artifact audit alone does not prove runtime behavior.
