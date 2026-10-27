# Tarif Atlası — Verification and acceptance evidence

**Last checked 2026-10-27.** This index supplements detailed [TESTING.md](docs/TESTING.md) and milestone reports; do not copy complete logs here. **PASS by a coding agent, independent review, GitHub Actions success, native GUI testing and PO acceptance are distinct claims.** Each new milestone should link its authoritative CI, exact commit, report and PO acceptance decision.

## Last integrated milestone: M4B-2 — 2026-10-27

- **Exact engineering/CI evidence:** [PR #11](https://github.com/fatihemres/personal-recipe-library/pull/11) final head `0b7cd9cb2891e8af40e6ab2274bd416ea20d825a`, protected-base `8663787db4b68c8f4043f4a3405ae55eaf1dddd6`. [Actions 40325150468](https://github.com/fatihemres/personal-recipe-library/actions/runs/40325150468) **5/5 PASS** on exact head (Frontend and browser; Rust/SQLite/catalog; macOS Apple Silicon; macOS Intel; Windows x64 installer). Three unsigned artifacts present. Agent reports checksum matching, not independently recalculated by ChatGPT.
- **Agent local checks (not independently rerun):** 94 Rust/SQLite, 69 frontend and 7 Playwright/browser tests in final UI/layout test report; isolated schema-8 Apple Silicon app startup/restart, SQLite integrity and Catalog Release 11, notes/filter/dashboard/Cmd-Q/Trash-restore smoke, actual-component layout harness; exact native narrow/docked measurements initially agent-UNVERIFIED.
- **PO manual Mac acceptance:** PO first reported functional M4B-2 checks completed, identified and had two responsive-layout defects corrected, then explicitly reported **5/5 visual tests PASS** in normal, narrow, side-by-side and fullscreen window arrangements plus theme/enlarged text. Test application used isolated `com.recipeatlas.m4b2acceptance`, not normal production user data. These are PO statements, not automatically recorded native GUI test logs.
- **Integration:** PO separately and expressly authorized a merge commit for exact candidate head. GitHub [PR #11](https://github.com/fatihemres/personal-recipe-library/pull/11) CLOSED/MERGED at `fb944f87c3c898bcf69b3e889bfa3cc8a39748c4`; independent GitHub verification found protected `main` and branch ref at the same SHA with exactly two ordered parents: `8663787db4b68c8f4043f4a3405ae55eaf1dddd6` (previous main), `0b7cd9cb2891e8af40e6ab2274bd416ea20d825a` (approved head).
- **Still unverified/not authorized:** Real Intel Mac/Windows GUI and installers, signed/notarized public distribution, normal personal SQLite profile schema-7→8 upgrade, annotated M4B-2 checkpoint tag and portable backup/restore release gates. Media/history/Cooking Mode begin in M5, which has only a proposed decomposition in [Issue #14](https://github.com/fatihemres/personal-recipe-library/issues/14). See [M4B-2 acceptance details](docs/M4B2_ACCEPTANCE.md) and [Issue #10](https://github.com/fatihemres/personal-recipe-library/issues/10).

## Historical integrated milestone: M4A

- Candidate commit: `f6441fe2c5b503a8bb95e0ce81e00412697ae1ad`.
- Integration: [PR #5](https://github.com/fatihemres/personal-recipe-library/pull/5) merge commit `49955722e8359e19eee206c577faa7ba600c96f8`, two verified parents (previous main `21c930fad356197a36c9073e061ee83a9829083c` and candidate `f6441fe...`).
- **Independently checked GitHub CI:** [Actions run 39851790479](https://github.com/fatihemres/personal-recipe-library/actions/runs/39851790479) completed successfully on exact head. Jobs: Frontend and browser; Rust, SQLite and catalog; macOS Apple Silicon package; macOS Intel package; Windows x64 installer — all 5 PASS.
- **Coding-agent local report, not re-executed by ChatGPT:** 85 Rust/SQLite, 57 frontend and two Playwright browser tests PASS; typecheck, lint, fmt, Clippy, static catalog checks and arm64 production build. [Measured report](docs/M4A_ACCEPTANCE_AND_PERFORMANCE.md) notes 500 synthetic recipes / 854 ingredients, 100 paginated searches, median 39.551 ms and p95 45.387 ms in one debug host run; not a native UI SLA.
- **Native test distinction:** isolated arm64 app successfully launched/relaunched on schema 6, catalog Release 11 and clean integrity; automated GUI attachment did not succeed. **PO reported 8/8 real Mac manual acceptance tests PASS on 2026-10-23 in conversation.** This is not an independently recorded screen capture or a Windows/Intel native GUI test.
- **Open:** actual Windows and Intel Mac GUI/installer execution; signing/notarization, public distribution. Schema-6 upgrade means an old binary cannot safely be assumed compatible with an upgraded profile.

## Last integrated milestone: M4B-1 — 2026-10-23
- Exact candidate head `d62e87c56657df9ff13b0b3fa0ea45e385c16a02` in [PR #8](https://github.com/fatihemres/personal-recipe-library/pull/8). Previous main `af094688377d1ec94bc793a381640aedae872c98`.
- **Independently checked GitHub CI:** [Actions run 39935646009](https://github.com/fatihemres/personal-recipe-library/actions/runs/39935646009) completed SUCCESS on the exact head. Required frontend/browser, Rust/SQLite/catalog, macOS arm64 package, macOS Intel package and Windows x64 installer jobs **all 5 PASS**. Protected branch rules, mergeability and no unresolved review threads checked before merge.
- **Coding-agent test report, not rerun by ChatGPT:** 90 Rust/SQLite, 61 frontend Vitest, 3 Playwright browser tests plus typecheck, lint, Cargo fmt/check/Clippy, catalog audit and isolated arm64 packaged app launch/restart passed. Isolated schema 7, catalog Release 11 with 854 canonical ingredients, integrity OK, zero FK violations; normal user profile untouched.
- **PO native Mac acceptance:** PO explicitly reported **7/7 checks PASS** on a temporary isolated arm64 app using unique identifier `com.recipeatlas.m4b1acceptance`, built from the exact candidate head. This was user-reported manual GUI acceptance, not independently automated native click testing; it did not exercise the daily `com.recipeatlas.desktop` profile. PO gave **separate explicit authorization** for merge-commit integration of the exact PR head.
- **Integration verified:** [PR #8](https://github.com/fatihemres/personal-recipe-library/pull/8) merged as `77825776e1cfcf57eb81e547cb3407e5af9d05c2`. GitHub protected `main` and branch ref matched merge SHA; parent #1 `af094688377d1ec94bc793a381640aedae872c98`, parent #2 `d62e87c56657df9ff13b0b3fa0ea45e385c16a02`. PO terminal output confirmed a safe local fast-forward to `7782577`, with `origin/main` aligned and an untouched untracked review workbook.
- **Remaining unverified:** Intel/Windows native GUI/installer execution, signing/notarization, and actual migration of the user's normal profile. Official unsigned PR artifacts retain the production identifier, so they must not be casually launched against a daily profile. The annotated `milestone/m4b1-recipe-organization` tag was created later and peels to merge `77825776e1cfcf57eb81e547cb3407e5af9d05c2`; no GitHub Release was created. M4B-2 is an unmerged candidate.
- Detailed acceptance checklist: [docs/M4B1_ACCEPTANCE.md](docs/M4B1_ACCEPTANCE.md); recorded PO/merge checkpoint: [Issue #7](https://github.com/fatihemres/personal-recipe-library/issues/7).

## Recent catalog milestone CI pointers
| Work | Head evidence | GitHub CI | PO native acceptance | Integration |
| --- | --- | --- | --- | --- |
| M3D-4 Release 10 / PR #3 | `c839a7d9306b352b1bf6e7625e5fd6f72aa1a29e` | [39236466544](https://github.com/fatihemres/personal-recipe-library/actions/runs/39236466544): five PASS | PO reported 8/8 Mac | merge `e85c26043637da5daef0e2f7d8e8eacfc634944f` |
| M3D-5 Release 11 / PR #4 | `51b2c44b232fdf8e83b91399b62ba3255e41d6b7` | [39762969533](https://github.com/fatihemres/personal-recipe-library/actions/runs/39762969533): five PASS | PO reported 8/8 Mac | merge `21c930fad356197a36c9073e061ee83a9829083c` |
| M4A / PR #5 | `f6441fe2c5b503a8bb95e0ce81e00412697ae1ad` | [39851790479](https://github.com/fatihemres/personal-recipe-library/actions/runs/39851790479): five PASS | PO reported 8/8 Mac | merge `49955722e8359e19eee206c577faa7ba600c96f8` |
| M4B-1 / PR #8 | `d62e87c56657df9ff13b0b3fa0ea45e385c16a02` | [39935646009](https://github.com/fatihemres/personal-recipe-library/actions/runs/39935646009): five PASS | PO reported 7/7 isolated Mac | merge `77825776e1cfcf57eb81e547cb3407e5af9d05c2` |

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
