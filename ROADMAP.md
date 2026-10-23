# Tarif Atlası — Product delivery roadmap

**Checkpoint 2026-10-23.** The full V1 boundary comes from [MASTER_SPEC.md](docs/MASTER_SPEC.md); [IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) contains detailed scope/acceptance. This file records *progress*, not automatic development authorization. Verify current work remotely and ask about unpushed local changes before starting.

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
| **M4B overall** | Recipe favorites, collections, ratings, private/rich notes, saved filters, database-backed dashboard navigation | **PARTIALLY INTEGRATED: M4B-1 PO ACCEPTED; M4B-2 CANDIDATE** | M4B-2 PR, CI, native PO acceptance and explicit merge gates |
| **M4B-1** | Recipe favorite/To Try, nullable 1–5 rating, custom collections and membership | **INTEGRATED + PO ACCEPTED** via [PR #8](https://github.com/fatihemres/personal-recipe-library/pull/8), schema 7, 2026-10-23 | Optional annotated checkpoint tag only with separate PO authorization; cross-platform GUI remains unverified |
| **M4B-2** | Private versioned Markdown notes, live named saved filters and real active dashboard | **IN PROGRESS / CANDIDATE** on `feature/m4b2-notes-filters-dashboard`, authorized by [Issue #10](https://github.com/fatihemres/personal-recipe-library/issues/10) | Complete local/native verification, five-job PR CI, independent review and separate PO acceptance/merge approval |
| M5 | Advanced recipe editor, variations/version history, images/media, Cooking Mode | PLANNED; depends on M4 | Real media, history and native acceptance |
| M6 | Full measurement and conversion engine; specialized Beverage Studio and honest ABV estimates | PLANNED; depends on M3/M5 | Precision/source-backed calculation tests |
| M7 | Pantry lots and shopping lists | PLANNED; depends on M6 | Safe stock deduction, aggregation, offline native tests |
| M8–M11 | Meal planning, stats/integration, data portability, QA/cross-platform/release | PLANNED; use exact [implementation plan](docs/IMPLEMENTATION_PLAN.md) milestone ownership | Complete R01–R20 before v1.0.0 |

## M4B overall acceptance outline (M4B-1 completed; M4B-2 requires scoped PO review)
1. Recipe favorite + to-try and persistent ratings with selected policy; many-to-many custom collections.
2. Private/rich notes with stored format/version and safe rendering; save/edit/reopen/restart without loss.
3. Saved library filters with exact typed query model and explicit archive/trash handling; no SQL-string persistence.
4. Dashboard counts, favorites, recent and collection links derive from real SQLite queries and open matching filtered views.
5. Turkish sort/filter/search and keyboard/a11y UI; no mock records, no hardcoded metrics.
6. Forward-only migration only if needed, genuine SQLite/renderer/native verification, PR five CI jobs, Mac PO acceptance, separate merge authorization.

No M4B-2 code is approved solely by this outline. M4B should not silently absorb M5 history/media or M6 beverage-specific editing.

## Ongoing safety gates
- Always protect installed catalog Release 11, previous release archives, recipe/draft data, exact decimal quantity/unit state and user overrides.
- Build/CI package success is not native GUI verification on Intel/Windows. A public release requires installation, signing/notarization and a full V1 acceptance process.
- Consult [PROJECT_STATUS.md](PROJECT_STATUS.md), [DECISIONS.md](DECISIONS.md), [VERIFICATION.md](VERIFICATION.md), [IDEA_REGISTER.md](IDEA_REGISTER.md), and [engineering process](docs/DEVELOPMENT_AND_RELEASE_PROCESS.md) before claiming a milestone closed.
