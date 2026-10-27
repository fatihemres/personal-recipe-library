# Tarif Atlası — Product delivery roadmap

**Checkpoint 2026-10-27.** The full V1 boundary comes from [MASTER_SPEC.md](docs/MASTER_SPEC.md); [IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) contains detailed scope/acceptance. This file records *progress*, not automatic development authorization. Verify current work remotely and inspect local changes before starting.

## Status vocabulary
- **PLANNED**: in approved V1 scope, but no verified work begun.
- **IN PROGRESS**: active verified branch/work.
- **IMPLEMENTED/CANDIDATE**: code or artifact created; not necessarily tested.
- **VERIFIED**: specified tests/evidence passed; not necessarily integrated.
- **INTEGRATED**: merged into protected main.
- **PO ACCEPTED/CLOSED**: Product Owner explicitly accepted scope. A milestone may be integrated but lack broader platform/public-release acceptance.

## Milestone map
| Milestone | Outcome/dependency | Verified state at checkpoint | Next gate |
| --- | --- | --- | --- |
| M0–M2A/M2B | Native shell, recipe CRUD, exact units, local SQLite drafts and recovery | Implemented and integrated historically; see progress/tests | Regression preservation for each next migration |
| M3A–M3C | Offline catalog pipeline, ingredient library, category hierarchy, aliases, favorites for ingredients and terminology curation | Implemented/integrated historically, with documented unresolved terminology | Retain immutable evidence and personal overrides |
| M3D-2B | Catalog Release 8, 647 identities | INTEGRATED / checkpoint tagged | Historical |
| M3D-3 | Release 9, 745 identities | INTEGRATED / checkpoint tagged | Historical |
| M3D-4 | Release 10, 795 identities | INTEGRATED / PO Mac-accepted / checkpoint tagged | Historical |
| M3D-5 | Release 11, 854 identities (666 food/188 beverage) | INTEGRATED / PO Mac-accepted / checkpoint tagged | **Catalog expansion paused by PO** |
| M4A | Search-first recipe library, FTS5, filters, sorting, pagination, keyboard | **INTEGRATED + PO ACCEPTED** via [PR #5](https://github.com/fatihemres/personal-recipe-library/pull/5), 2026-10-23; schema 6 | Verify local FF/tag separately |
| **M4B overall** | Recipe favorites, collections, ratings, private/rich notes, saved filters, database-backed dashboard navigation | **INTEGRATED + PO ACCEPTED: M4B-1 and M4B-2** | M4B delivered to main; optional checkpoint tag only by separate PO authorization |
| **M4B-1** | Recipe favorite/To Try, nullable 1–5 rating, custom collections and membership | **INTEGRATED + PO ACCEPTED** via [PR #8](https://github.com/fatihemres/personal-recipe-library/pull/8), schema 7, 2026-10-23; annotated `milestone/m4b1-recipe-organization` peels to merge `77825776e1cfcf57eb81e547cb3407e5af9d05c2` | Cross-platform GUI remains unverified; no release implied |
| **M4B-2** | Private versioned Markdown notes, live named saved filters and real active dashboard | **INTEGRATED + PO ACCEPTED** via [PR #11](https://github.com/fatihemres/personal-recipe-library/pull/11), schema 8, 2026-10-27; final head `0b7cd9cb2891e8af40e6ab2274bd416ea20d825a`, [CI 40325150468](https://github.com/fatihemres/personal-recipe-library/actions/runs/40325150468) 5/5 and merge `fb944f87c3c898bcf69b3e889bfa3cc8a39748c4` | Cross-platform GUI, ordinary-profile migration and any tag/release remain distinct gates |
| M5 | Advanced recipe editor, variations/version history, images/media, Cooking Mode | **PLANNED**; [Issue #14](https://github.com/fatihemres/personal-recipe-library/issues/14) has **proposed** M5A–M5D breakdown only | PO confirms scope/policies before coding; native/media/history safety acceptance |
| M6 | Full measurement and conversion engine; specialized Beverage Studio and honest ABV estimates | PLANNED; depends on M3/M5 | Precision/source-backed calculation tests |
| M7 | Pantry lots and shopping lists | PLANNED; depends on M6 | Safe stock deduction, aggregation, offline native tests |
| M8–M11 | Meal planning, stats/integration, data portability, QA/cross-platform/release | PLANNED; use exact [implementation plan](docs/IMPLEMENTATION_PLAN.md) milestone ownership | Complete R01–R20 before v1.0.0 |

## M4B overall acceptance outline (M4B-1 and M4B-2 integrated/PO accepted; remaining V1 gates apply)
1. Recipe favorite + to-try and persistent ratings with selected policy; many-to-many custom collections.
2. Private/rich notes with stored format/version and safe rendering; save/edit/reopen/restart without loss.
3. Saved library filters with exact typed query model and explicit archive/trash handling; no SQL-string persistence.
4. Dashboard counts, favorites, recent and collection links derive from real SQLite queries and open matching filtered views.
5. Turkish sort/filter/search and keyboard/a11y UI; no mock records, no hardcoded metrics.
6. Forward-only migration only if needed, genuine SQLite/renderer/native verification, PR five CI jobs, Mac PO acceptance, separate merge authorization.

M4B-2 was integrated after separately reported PO Mac acceptance and explicit merge approval (DEC-018). This outline does not establish any wider V1/platform/release acceptance. M4B does not absorb M5 history/media or M6 beverage-specific editing.

## Ongoing safety gates
- Always protect installed catalog Release 11, previous release archives, recipe/draft data, exact decimal quantity/unit state and user overrides.
- Build/CI package success is not native GUI verification on Intel/Windows. A public release requires installation, signing/notarization and a full V1 acceptance process.
- Consult [PROJECT_STATUS.md](PROJECT_STATUS.md), [DECISIONS.md](DECISIONS.md), [VERIFICATION.md](VERIFICATION.md), [IDEA_REGISTER.md](IDEA_REGISTER.md), and [engineering process](docs/DEVELOPMENT_AND_RELEASE_PROCESS.md) before claiming a milestone closed.
