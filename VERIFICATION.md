# Tarif Atlası — Verification and acceptance evidence

**Last checked 2026-10-23.** This index supplements detailed [TESTING.md](docs/TESTING.md) and milestone reports; do not copy complete logs here. **PASS by a coding agent, independent review, GitHub Actions success, native GUI testing and PO acceptance are distinct claims.** Each new milestone should link its authoritative CI, exact commit, report and PO acceptance decision.

## Last integrated milestone: M4A
- Candidate commit: `f6441fe2c5b503a8bb95e0ce81e00412697ae1ad`.
- Integration: [PR #5](https://github.com/fatihemres/personal-recipe-library/pull/5) merge commit `49955722e8359e19eee206c577faa7ba600c96f8`, two verified parents (previous main `21c930fad356197a36c9073e061ee83a9829083c` and candidate `f6441fe...`).
- **Independently checked GitHub CI:** [Actions run 39851790479](https://github.com/fatihemres/personal-recipe-library/actions/runs/39851790479) completed successfully on exact head. Jobs: Frontend and browser; Rust, SQLite and catalog; macOS Apple Silicon package; macOS Intel package; Windows x64 installer — all 5 PASS.
- **Coding-agent local report, not re-executed by ChatGPT:** 85 Rust/SQLite, 57 frontend and two Playwright browser tests PASS; typecheck, lint, fmt, Clippy, static catalog checks and arm64 production build. [Measured report](docs/M4A_ACCEPTANCE_AND_PERFORMANCE.md) notes 500 synthetic recipes / 854 ingredients, 100 paginated searches, median 39.551 ms and p95 45.387 ms in one debug host run; not a native UI SLA.
- **Native test distinction:** isolated arm64 app successfully launched/relaunched on schema 6, catalog Release 11 and clean integrity; automated GUI attachment did not succeed. **PO reported 8/8 real Mac manual acceptance tests PASS on 2026-10-23 in conversation.** This is not an independently recorded screen capture or a Windows/Intel native GUI test.
- **Open:** actual Windows and Intel Mac GUI/installer execution; signing/notarization, public distribution. Schema-6 upgrade means an old binary cannot safely be assumed compatible with an upgraded profile.

## M4B-1 engineering candidate — 2026-10-23
- Branch `feature/m4b1-recipe-organization` starts at `af094688...` in an isolated worktree; unrelated root workbook untouched. [PR #8](https://github.com/fatihemres/personal-recipe-library/pull/8) is **not integrated or PO-accepted**. Its initial head `15beee53ea35da550f46e8520184822cab1c78fa` passed all five jobs in [Actions run 39926175959](https://github.com/fatihemres/personal-recipe-library/actions/runs/39926175959), with arm64/Intel/Windows artifacts and matching SHA-256 manifests. Any later head requires a new exact-head five-job check; the latest result is on the PR.
- Local checks: 90 Rust/SQLite, 61 Vitest and three Playwright browser tests; typecheck, ESLint, Cargo fmt/check/Clippy, frontend build, catalog audit, five Python audit tests and app version check passed. Production arm64 bundle with temporary identifier launched and restarted on isolated schema-7/Release-11 profile (`integrity_check=ok`, zero FK rows). See [candidate record](docs/M4B1_ACCEPTANCE.md).
- Native accessibility confirmed the shell and organization navigation/filters, but no native create/edit collection or rating GUI click flow was exercised. Those remain for Product Owner Mac acceptance. The downloadable artifact uses the normal `com.recipeatlas.desktop` identifier; use a disposable Standard macOS account as specified in the [safe acceptance guide](docs/M4B1_ACCEPTANCE.md), not the PO's normal profile. Windows/Intel GUI and signed releases remain unverified.

## Recent catalog milestone CI pointers
| Work | Head evidence | GitHub CI | PO native acceptance | Integration |
| --- | --- | --- | --- | --- |
| M3D-4 Release 10 / PR #3 | `c839a7d9306b352b1bf6e7625e5fd6f72aa1a29e` | [39236466544](https://github.com/fatihemres/personal-recipe-library/actions/runs/39236466544): five PASS | PO reported 8/8 Mac | merge `e85c26043637da5daef0e2f7d8e8eacfc634944f` |
| M3D-5 Release 11 / PR #4 | `51b2c44b232fdf8e83b91399b62ba3255e41d6b7` | [39762969533](https://github.com/fatihemres/personal-recipe-library/actions/runs/39762969533): five PASS | PO reported 8/8 Mac | merge `21c930fad356197a36c9073e061ee83a9829083c` |
| M4A / PR #5 | `f6441fe2c5b503a8bb95e0ce81e00412697ae1ad` | [39851790479](https://github.com/fatihemres/personal-recipe-library/actions/runs/39851790479): five PASS | PO reported 8/8 Mac | merge `49955722e8359e19eee206c577faa7ba600c96f8` |

## Verification contract for future milestones
1. Identify exact PR head SHA, base SHA and parent ancestry. No stale Actions result.
2. Record local commands/environments/results in owning milestone report; label unrun or tool-inaccessible checks **UNVERIFIED**.
3. CI five checks must pass on current head; inspect artifact list and checksums when testing packages.
4. Separate isolated database/native app verification from user's normal profile; preserve personal SQLite backup strategy, drafts, quantity/units and source snapshots.
5. PO conducts or explicitly accepts documented Mac manual tests and authorizes merge **separately**. Intel/Windows GUI, installer, signing and notarization remain distinct verification gates.
6. After integration, inspect actual GitHub main/merge parent commits and update the dated checkpoint. Never claim a GitHub PR artifact is a signed stable release.

## Security and governance gaps to track
The PR CI enforces type/lint/tests/build across five jobs, but **no dedicated CodeQL/SAST/dependency-audit workflow was observed at this checkpoint**. Before public V1 release, review security scanning, dependencies and release secrets/signing as a scoped decision, rather than silently adding costly workflows. Root README/plan/checklist previously contained historical counts/status; refer to the new status pointer and exact milestone reports.

See [engineering/release policy](docs/DEVELOPMENT_AND_RELEASE_PROCESS.md) and [progress record](PROJECT_PROGRESS.md).
