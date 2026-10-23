# Tarif Atlası — Project status checkpoint

**Verified against GitHub 2026-10-23.** This is a dated checkpoint, **not a live dashboard**. Re-check branches, PRs, CI and repo content before continuing. Product Owner (PO) acceptance is different from CI and implementation.

## Product and technical baseline
- Repository: https://github.com/fatihemres/personal-recipe-library; protected integration branch `main`.
- Verified `main` HEAD: `af094688377d1ec94bc793a381640aedae872c98` — merge of governance [PR #6](https://github.com/fatihemres/personal-recipe-library/pull/6), after M4A [PR #5](https://github.com/fatihemres/personal-recipe-library/pull/5) merged at `49955722e8359e19eee206c577faa7ba600c96f8`.
- Application version **0.1.0** (development, not public v1.0.0); protected `main` has SQLite schema **6** (M4A migration 006), while the unmerged M4B-1 candidate has schema **7** (migration 007). Bundled catalog **Release 11** remains 854 canonical ingredients (666 food/188 beverage), 269 aliases and 50 category nodes.
- Stack: Tauri 2 + React/TypeScript/Vite + Rust worker/repositories + SQLite/FTS5. Offline-first, local user data. Turkish-first UI. macOS/Windows are targets.
- Annotated milestone tags confirmed on GitHub: `milestone/m3d-2b-release8`, `milestone/m3d-3-release9`, `milestone/m3d-4-release10`, `milestone/m3d-5-release11`, and the existing `milestone/m4a-recipe-search-library` tag, which peels to M4A merge `49955722e8359e19eee206c577faa7ba600c96f8`. No M4B tag exists or is authorized.

## Last verified integration
- **M4A Search-first Recipe Library — INTEGRATED and PO-ACCEPTED (conversation acceptance on 2026-10-23).** [PR #5](https://github.com/fatihemres/personal-recipe-library/pull/5) closed/merged at `49955722e8359e19eee206c577faa7ba600c96f8`. Parent #1 `21c930fad356197a36c9073e061ee83a9829083c`; parent #2 `f6441fe2c5b503a8bb95e0ce81e00412697ae1ad`.
- **Verification:** [Actions run 39851790479](https://github.com/fatihemres/personal-recipe-library/actions/runs/39851790479) passed five required jobs on PR head `f6441fe2`. Reported local 85 Rust/SQLite, 57 frontend, two browser tests plus audits/build; independent re-execution of those local commands was not performed by ChatGPT. The PO reported **8/8 manual real-Mac acceptance checks passed**; this human report is separate from CI and may not have a repository artifact. See [M4A report](docs/M4A_ACCEPTANCE_AND_PERFORMANCE.md) and [verification ledger](VERIFICATION.md).
- M4A added bounded SQLite recipe FTS search, filters, sorting, card/list presentation, pagination, and keyboard navigation. No claim of native GUI testing on Windows/Intel Mac or public signing/notarization.

## Current M4B-1 candidate (2026-10-23)
- The PO authorized starting M4B in [Issue #7](https://github.com/fatihemres/personal-recipe-library/issues/7) and the scoped brief; [DEC-014](DECISIONS.md) supersedes the earlier proposal's authorization status. [PR #8](https://github.com/fatihemres/personal-recipe-library/pull/8) on `feature/m4b1-recipe-organization` implements the first slice only. It is **not integrated or PO-accepted**. The branch starts from `af094688...` in an isolated checkout; the unrelated root workbook remains untouched. The five required jobs passed on initial head `15beee53...` in [Actions run 39926175959](https://github.com/fatihemres/personal-recipe-library/actions/runs/39926175959); any later head requires its own five-job result and matching artifacts.
- Migration 007 adds recipe organization state and normalized custom collections/membership, preserving recipe content/draft revisions, decimal quantities and Catalog Release 11. Backend/renderer/native evidence belongs in [VERIFICATION.md](VERIFICATION.md) and [TESTING.md](docs/TESTING.md). M4B-2 still owns rich/private notes, saved filters and dashboard.

## Historical planning direction (before DEC-014; retained for context)
- **PO direction (2026-10-22): Pause further catalog expansion**; prioritize completion of remaining V1 app capabilities. Release 11 remains the stable catalog baseline.
- **At the earlier planning checkpoint**, M4B was proposed for recipe favorites, custom collections, ratings, private/rich notes, saved filters and correct database-backed dashboard links. That proposal did not authorize coding at the time; subsequent [DEC-014](DECISIONS.md) authorized two slices, and M4B-1 is now the PR #8 candidate. M4B-2 retains notes, saved filters and dashboard. Follow M5 advanced recipe media/history/Cooking Mode, M6 measurement+Beverage Studio, M7 pantry/shopping, and remaining modules per [implementation plan](docs/IMPLEMENTATION_PLAN.md) and [roadmap](ROADMAP.md).

## Open risks and PO decisions
- Native GUI behavior on Intel macOS and Windows remains unverified; downloaded PR artifacts are unsigned/ad-hoc packages, not distributable signed releases.
- The PR arm64 package uses `com.recipeatlas.desktop`; opening it in the PO's normal macOS account may upgrade that account's database. [M4B-1 acceptance instructions](docs/M4B1_ACCEPTANCE.md) require a disposable Standard macOS account for candidate GUI testing.
- M4B-1 implements the authorized nullable 1–5 rating and collection policy as a candidate pending PO native acceptance. M4B-2 still needs decisions on rich-note format/sanitization and saved-filter persistence.
- Full portable backup/restore, Windows clean-install/offline strategy, release signing/notarization, and V1 readiness remain future gates.
- Historical catalog terminology/review deferrals are retained in existing reports; do not re-open catalog growth just because they remain open.
- Local `main` and `origin/main` were checked at `af094688...`; the root `INGREDIENT_TERMINOLOGY_REVIEW.xlsx` remains untracked and untouched, and the M4B-1 branch resides in a separate worktree. Recheck live state before future work.
- Governance [PR #6](https://github.com/fatihemres/personal-recipe-library/pull/6) **is integrated** at `af094688...`; no inference of M4B-1 acceptance or merge follows from it.

## Pointers
- [Product requirements](docs/MASTER_SPEC.md) · [Roadmap](ROADMAP.md) · [Decisions](DECISIONS.md) · [Ideas](IDEA_REGISTER.md)
- [Architecture](docs/ARCHITECTURE.md) · [Engineering workflow](docs/DEVELOPMENT_AND_RELEASE_PROCESS.md) · [Evidence](VERIFICATION.md)
- [Detailed historical log](PROJECT_PROGRESS.md) · [Tests](docs/TESTING.md) · [Change log](CHANGELOG.md)
