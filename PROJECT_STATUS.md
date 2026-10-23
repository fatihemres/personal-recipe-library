# Tarif Atlası — Project status checkpoint

**Verified against GitHub 2026-10-23.** This is a dated checkpoint, **not a live dashboard**. Re-check branches, PRs, CI and repo content before continuing. Product Owner (PO) acceptance is different from CI and implementation.

## Product and technical baseline
- Repository: https://github.com/fatihemres/personal-recipe-library; protected integration branch `main`.
- Verified `main` HEAD: `49955722e8359e19eee206c577faa7ba600c96f8` — merge of [PR #5](https://github.com/fatihemres/personal-recipe-library/pull/5).
- Application version **0.1.0** (development, not public v1.0.0); SQLite schema **6** (M4A migration 006); bundled catalog **Release 11**: 854 canonical ingredients (666 food/188 beverage), 269 aliases and 50 category nodes.
- Stack: Tauri 2 + React/TypeScript/Vite + Rust worker/repositories + SQLite/FTS5. Offline-first, local user data. Turkish-first UI. macOS/Windows are targets.
- Released checkpoints confirmed on GitHub: `milestone/m3d-2b-release8`, `milestone/m3d-3-release9`, `milestone/m3d-4-release10`, `milestone/m3d-5-release11`. **M4A annotated tag was not present on GitHub at this snapshot**; do not create or move it without the usual post-merge verification/PO workflow.

## Last verified integration
- **M4A Search-first Recipe Library — INTEGRATED and PO-ACCEPTED (conversation acceptance on 2026-10-23).** [PR #5](https://github.com/fatihemres/personal-recipe-library/pull/5) closed/merged at `49955722e8359e19eee206c577faa7ba600c96f8`. Parent #1 `21c930fad356197a36c9073e061ee83a9829083c`; parent #2 `f6441fe2c5b503a8bb95e0ce81e00412697ae1ad`.
- **Verification:** [Actions run 39851790479](https://github.com/fatihemres/personal-recipe-library/actions/runs/39851790479) passed five required jobs on PR head `f6441fe2`. Reported local 85 Rust/SQLite, 57 frontend, two browser tests plus audits/build; independent re-execution of those local commands was not performed by ChatGPT. The PO reported **8/8 manual real-Mac acceptance checks passed**; this human report is separate from CI and may not have a repository artifact. See [M4A report](docs/M4A_ACCEPTANCE_AND_PERFORMANCE.md) and [verification ledger](VERIFICATION.md).
- M4A added bounded SQLite recipe FTS search, filters, sorting, card/list presentation, pagination, and keyboard navigation. No claim of native GUI testing on Windows/Intel Mac or public signing/notarization.

## Current M4B-1 candidate (2026-10-23)
- The PO authorized starting M4B in [Issue #7](https://github.com/fatihemres/personal-recipe-library/issues/7) and this brief. `feature/m4b1-recipe-organization` implements the first slice only; it is **not integrated or PO-accepted**. The branch starts from `af094688...` in an isolated checkout; the unrelated root workbook remains untouched.
- Migration 007 adds recipe organization state and normalized custom collections/membership, preserving recipe content/draft revisions, decimal quantities and Catalog Release 11. Backend/renderer/native evidence belongs in [VERIFICATION.md](VERIFICATION.md) and [TESTING.md](docs/TESTING.md). M4B-2 still owns rich/private notes, saved filters and dashboard.

## Prior direction
- **PO direction (2026-10-22): Pause further catalog expansion**; prioritize completion of remaining V1 app capabilities. Release 11 remains the stable catalog baseline.
- **Proposed next scope: M4B** — recipe favorites, custom collections, ratings, private/rich notes, saved filters, correct database-backed dashboard links. Scope/acceptance should be reviewed before implementation. No M4B branch, PR or local development is verified here. Follow M5 advanced recipe media/history/Cooking Mode, M6 measurement+Beverage Studio, M7 pantry/shopping, and remaining modules per [implementation plan](docs/IMPLEMENTATION_PLAN.md) and [roadmap](ROADMAP.md).

## Open risks and PO decisions
- Native GUI behavior on Intel macOS and Windows remains unverified; downloaded PR artifacts are unsigned/ad-hoc packages, not distributable signed releases.
- M4B must decide rating scale, rich-note format/sanitization and exact collection/saved-filter policy without inventing earlier PO approvals.
- Full portable backup/restore, Windows clean-install/offline strategy, release signing/notarization, and V1 readiness remain future gates.
- Historical catalog terminology/review deferrals are retained in existing reports; do not re-open catalog growth just because they remain open.
- Mac local `main` fast-forward following PR #5 merge, root untracked workbook status, other local worktrees/branches and an M4A milestone tag **cannot be verified remotely**. Inspect local state before new branch work.
- Governance documentation PR itself is **not integrated** until separately approved and merged.

## Pointers
- [Product requirements](docs/MASTER_SPEC.md) · [Roadmap](ROADMAP.md) · [Decisions](DECISIONS.md) · [Ideas](IDEA_REGISTER.md)
- [Architecture](docs/ARCHITECTURE.md) · [Engineering workflow](docs/DEVELOPMENT_AND_RELEASE_PROCESS.md) · [Evidence](VERIFICATION.md)
- [Detailed historical log](PROJECT_PROGRESS.md) · [Tests](docs/TESTING.md) · [Change log](CHANGELOG.md)
