# Tarif Atlası — Project status checkpoint

**Verified against GitHub 2026-10-26.** This is a dated checkpoint, **not a live dashboard**. Re-check branches, PRs, CI and repo content before continuing. Product Owner (PO) acceptance is different from CI and implementation.

## Product and technical baseline
- Repository: https://github.com/fatihemres/personal-recipe-library; protected integration branch `main`.
- Verified `main` HEAD and [PR #11](https://github.com/fatihemres/personal-recipe-library/pull/11) base: `8663787db4b68c8f4043f4a3405ae55eaf1dddd6`. M4B-1 [PR #8](https://github.com/fatihemres/personal-recipe-library/pull/8) merged at `77825776e1cfcf57eb81e547cb3407e5af9d05c2`, after governance PR #6 at `af094688377d1ec94bc793a381640aedae872c98` and M4A PR #5 at `49955722e8359e19eee206c577faa7ba600c96f8`.
- Application version **0.1.0** (development, not public v1.0.0); protected `main` has SQLite schema **7** (M4B-1 migration 007). Older binaries cannot safely be assumed compatible after upgrade. Bundled catalog **Release 11** remains 854 canonical ingredients (666 food/188 beverage), 269 aliases and 50 category nodes.
- Stack: Tauri 2 + React/TypeScript/Vite + Rust worker/repositories + SQLite/FTS5. Offline-first, local user data. Turkish-first UI. macOS/Windows are targets.
- Annotated milestone tags include `milestone/m3d-2b-release8`, `milestone/m3d-3-release9`, `milestone/m3d-4-release10`, `milestone/m3d-5-release11`, `milestone/m4a-recipe-search-library` (peels to M4A merge `49955722e8359e19eee206c577faa7ba600c96f8`) and `milestone/m4b1-recipe-organization` (tag object `bc65b6a98597e96f536a6675488c9aade211cc72`, peels to M4B-1 merge `77825776e1cfcf57eb81e547cb3407e5af9d05c2`).

## Last verified integration
- **M4B-2 implementation candidate — VERIFIED PR, awaiting PO native acceptance (2026-10-26).** [Issue #10](https://github.com/fatihemres/personal-recipe-library/issues/10) authorizes private versioned Markdown notes, live named filters and a real active-recipe dashboard. [PR #11](https://github.com/fatihemres/personal-recipe-library/pull/11) is open from `feature/m4b2-notes-filters-dashboard` against `main` `8663787db4b68c8f4043f4a3405ae55eaf1dddd6`. Its previous candidate head `540574047310e8c0a1933c61f4c8a7588c26f610` passed [five-job CI run 40004327930](https://github.com/fatihemres/personal-recipe-library/actions/runs/40004327930); this documentation follow-up needs its own exact-head CI. Schema 008 is candidate only; no merge or PO native acceptance is implied. See [M4B-2 candidate report](docs/M4B2_ACCEPTANCE.md).
- **M4B-1 Recipe Organization — INTEGRATED + PO ACCEPTED (2026-10-23).** [PR #8](https://github.com/fatihemres/personal-recipe-library/pull/8) head `d62e87c56657df9ff13b0b3fa0ea45e385c16a02`, five-job [CI run 39935646009](https://github.com/fatihemres/personal-recipe-library/actions/runs/39935646009) PASS, PO-reported isolated Mac GUI 7/7 PASS, separately authorized merge `77825776e1cfcf57eb81e547cb3407e5af9d05c2`. Ordered parents: `af094688...` (prior main) and `d62e87c...` (PR head). Favorites, independent To Try, nullable ratings and many-to-many collections live in main with schema 7. The annotated M4B-1 checkpoint tag exists; no GitHub Release was created. M4B-2 is an unmerged candidate.
- **M4A Search-first Recipe Library — INTEGRATED and PO-ACCEPTED (conversation acceptance on 2026-10-23).** [PR #5](https://github.com/fatihemres/personal-recipe-library/pull/5) closed/merged at `49955722e8359e19eee206c577faa7ba600c96f8`. Parent #1 `21c930fad356197a36c9073e061ee83a9829083c`; parent #2 `f6441fe2c5b503a8bb95e0ce81e00412697ae1ad`.
- **Verification:** [Actions run 39851790479](https://github.com/fatihemres/personal-recipe-library/actions/runs/39851790479) passed five required jobs on PR head `f6441fe2`. Reported local 85 Rust/SQLite, 57 frontend, two browser tests plus audits/build; independent re-execution of those local commands was not performed by ChatGPT. The PO reported **8/8 manual real-Mac acceptance checks passed**; this human report is separate from CI and may not have a repository artifact. See [M4A report](docs/M4A_ACCEPTANCE_AND_PERFORMANCE.md) and [verification ledger](VERIFICATION.md).
- M4A added bounded SQLite recipe FTS search, filters, sorting, card/list presentation, pagination, and keyboard navigation. No claim of native GUI testing on Windows/Intel Mac or public signing/notarization.

## M4B-1 integration and local checkout checkpoint (2026-10-23)
- [Issue #7](https://github.com/fatihemres/personal-recipe-library/issues/7) authorized implementation. [PR #8](https://github.com/fatihemres/personal-recipe-library/pull/8) was accepted by PO after 7/7 manually reported isolated arm64 GUI checks, and merged separately after explicit PO approval. Native GUI interaction on Windows/Intel was not performed.
- Migration 007 now integrated. Catalog Release 11, recipe/draft data and decimal-text quantities are preserved by automated verification. The isolated Mac acceptance build used app identifier `com.recipeatlas.m4b1acceptance`; it did not open the normal app profile. Actual personal-profile migration remains unverified.
- PO-terminal evidence: ordinary macOS checkout `main` fast-forwarded `af094688` → `7782577`, with `HEAD` and `origin/main` aligned. The untracked `INGREDIENT_TERMINOLOGY_REVIEW.xlsx` remains untouched.
- M4B-2 work began in a separate feature worktree after this historical integration checkpoint. The M4B-1 annotated checkpoint tag was created later; no public release was created.

## Historical planning direction (before DEC-014; retained for context)
- **PO direction (2026-10-22): Pause further catalog expansion**; prioritize completion of remaining V1 app capabilities. Release 11 remains the stable catalog baseline.
- **At the earlier planning checkpoint**, M4B was proposed for recipe favorites, custom collections, ratings, private/rich notes, saved filters and correct database-backed dashboard links. That proposal did not authorize coding at the time; subsequent [DEC-014](DECISIONS.md) authorized two slices, and M4B-1 was subsequently integrated and PO accepted via PR #8. M4B-2 retains notes, saved filters and dashboard. Follow M5 advanced recipe media/history/Cooking Mode, M6 measurement+Beverage Studio, M7 pantry/shopping, and remaining modules per [implementation plan](docs/IMPLEMENTATION_PLAN.md) and [roadmap](ROADMAP.md).

## Open risks and PO decisions
- Native GUI behavior on Intel macOS and Windows remains unverified; downloaded PR artifacts are unsigned/ad-hoc packages, not distributable signed releases.
- The official unsigned PR arm64 artifact uses `com.recipeatlas.desktop` and may upgrade the normal macOS profile. The PO instead manually tested an isolated build with identifier `com.recipeatlas.m4b1acceptance`. Daily-profile migration has not been verified.
- M4B-1 nullable 1–5 rating and collection policy was integrated and PO accepted. M4B-2 candidate policies on note format/lifecycle and saved-filter persistence are recorded as proposed DEC-017, pending PO native acceptance.
- Full portable backup/restore, Windows clean-install/offline strategy, release signing/notarization, and V1 readiness remain future gates.
- Historical catalog terminology/review deferrals are retained in existing reports; do not re-open catalog growth just because they remain open.
- The PO's local `main` and `origin/main` were aligned at `7782577` at the 2026-10-23 terminal checkpoint. The 2026-10-26 verified PR base and primary checkout are `8663787`; the root `INGREDIENT_TERMINOLOGY_REVIEW.xlsx` remains untracked and untouched. Recheck before future work.
- Governance [PR #6](https://github.com/fatihemres/personal-recipe-library/pull/6) was integrated at `af094688...`; it is the first parent of PR #8's merge commit.

## Pointers
- [Product requirements](docs/MASTER_SPEC.md) · [Roadmap](ROADMAP.md) · [Decisions](DECISIONS.md) · [Ideas](IDEA_REGISTER.md)
- [Architecture](docs/ARCHITECTURE.md) · [Engineering workflow](docs/DEVELOPMENT_AND_RELEASE_PROCESS.md) · [Evidence](VERIFICATION.md)
- [Detailed historical log](PROJECT_PROGRESS.md) · [Tests](docs/TESTING.md) · [Change log](CHANGELOG.md)
