## Latest M5B delivery status — M5B-4 integrated

**INTEGRATED + scoped PO_ACCEPTED — 2026-11-04.** [PR #29](https://github.com/fatihemres/personal-recipe-library/pull/29) CLOSED/MERGED via **merge commit** `e21801103a65d2df67b11ed927faf0eea9750d32`, with independently verified protected `main` at the same SHA and two ordered parents `6bf160d2eccd374cdc44031f64fcd71918d73630` (previous main) and `3d99eb5aae73b271fa86ac6dcfc5ad82654719f4` (unchanged approved feature head). [Exact-head CI #41642014893](https://github.com/fatihemres/personal-recipe-library/actions/runs/41642014893): **five required jobs SUCCESS**; the CI job logs directly confirm 145 Rust, 95 frontend and 13 browser tests PASS. [Independent source review](https://github.com/fatihemres/personal-recipe-library/pull/29#pullrequestreview-5763976826) had no blocking finding within the inspected high-risk diff but was COMMENT-only, not GitHub APPROVE. The PO [reported all five isolated native Mac checks PASS](https://github.com/fatihemres/personal-recipe-library/issues/28#issuecomment-6624804477), then gave [separate explicit scoped `PO_ACCEPTED`](https://github.com/fatihemres/personal-recipe-library/issues/28#issuecomment-6624857129) and later separately authorized PR #29 merge.

Delivered M5B-4/P7: multiple ordered alternatives for each recipe ingredient line, optional user-authored exact decimal quantity/unit (no guessed conversions), explicit draft-only Use Alternative until separate Save, reviewed owner-line removal, transactional persistence and rollback, immutable format-2 snapshots while old format-1 bytes/checksums stay untouched, mixed-format preview/comparison/restore, and independent Duplicate/Variation. **SQLite schema 12** (forward-only migration 012) is now integrated; application `0.1.0` and frozen Catalog Release **11 (854 canonical, 269 aliases, 50 categories)** remain unchanged. M5A and all four M5B slices are INTEGRATED + scoped PO_ACCEPTED; this is **not full M5/V1 release acceptance**.

Evidence limits: PO's five native Mac GUI cases are **PO-reported**; the reviewer did not rerun the GUI and received no associated screenshots/SQL receipts. Agent-reported 35 Python tests, 29 isolated catalog installations and native package/checksum details are not independently reexecuted here. Mac Intel/Windows native GUI, ordinary personal-profile schema 11→12 upgrade, large-library profiling, signing/notarization, portable backup/recovery and full V1 release remain **UNVERIFIED**. Older M5B-1 Test 4 discrepancy and M5B-2 follow-ups remain recorded. M5C offline media, M5D Cooking Mode, M6 conversions/scaling and any new feature coding require separate PO authorization. **No tag, GitHub Release, normal user profile operation or other PR merge** was performed by this milestone merge.

**Chronology:** Candidate/authorization-only sections below are historical checkpoint records, not the current product status. Do not overwrite or reinterpret them as current decisions.

---

## M5B-4 engineering candidate — 2026-11-03

[Issue #28 / explicit PO coding authorization](https://github.com/fatihemres/personal-recipe-library/issues/28#issuecomment-6603250800), recorded by DEC-029, supersedes earlier planning-only wording **for M5B-4 coding only**. Verified protected base `6bf160d2eccd374cdc44031f64fcd71918d73630`; branch `feature/m5b4-ingredient-alternatives`, isolated worktree `/private/tmp/tarif-atlasi-m5b4`. M5B-1/2/3 remain integrated/scoped PO accepted. M5B-4 adds candidate schema **12** and format-2 content snapshots; app **0.1.0**, Catalog Release **11 (854/269/50)** unchanged. IMPLEMENTED candidate; executed verification and remaining gates: [M5B4_ACCEPTANCE.md](docs/M5B4_ACCEPTANCE.md). Not INTEGRATED or PO_ACCEPTED; no normal/retained-profile access, merge, tag, release or M5C/M5D/M6 authority. Historical DEC/status/evidence below remain dated records, not current authorization blockers.

## Latest delivery status — M5B-3 (2026-11-03)

**INTEGRATED + scoped PO_ACCEPTED.** [PR #26](https://github.com/fatihemres/personal-recipe-library/pull/26) integrated M5B-3 by **merge commit** `f0e472d6f7f99335ac10a5c528bc363e785f0a9d` (parents: previous main `9298744c29f70f92ea3c1e60bae9bb31b481a2db` and approved head `5f7dfc76994efc8c9504d144164c85a0d8f8d884`). Independently verified protected `main` at merge SHA, PR closed/merged, [five exact-head CI checks PASS](https://github.com/fatihemres/personal-recipe-library/actions/runs/41601102074). The PO explicitly [accepted the scoped milestone](https://github.com/fatihemres/personal-recipe-library/issues/25#issuecomment-6600774200) following **5/5 PO-reported isolated Mac GUI acceptance PASS**, including actual confirmed final purge on disposable fixtures, then separately authorized merge. Source-level [independent review](https://github.com/fatihemres/personal-recipe-library/pull/26#pullrequestreview-5749350807) found no blocking issue (COMMENT, not formal GitHub APPROVE). SQLite schema **11** (forward-only 011) is integrated; app `0.1.0` and Catalog Release **11 (854 canonical/269 aliases/50 categories)** are unchanged.

**M5 overall:** M5A, M5B-1, M5B-2 and M5B-3 are integrated/scoped accepted; **M5B-4 substitutions** and M5C/D remain planned and separately gated. Evidence limits: the native 5/5 GUI outcome is **PO-reported**, not independently rerun by this GitHub reviewer. Intel/Windows native GUI, ordinary personal-profile schema 10→11 upgrade, large graph profiling, signed installers, portable backup and full V1 acceptance remain **UNVERIFIED**. M5B-1 Test 4 discrepancy is still open; M5B-2 Retry/large history follow-ups remain open. M5B-4 substitutions, M5C media and M5D Cooking Mode are **PLANNED, not code-authorized**. No tag, release, normal-profile access or other feature code permission.

**Historical status note:** Later-dated summary supersedes candidate-only claims in the older roadmap rows below. Future authors must not interpret those rows as current authority.

---

## M5B-3 candidate checkpoint — 2026-11-03

Issue #25 / DEC-027 explicitly authorize variations, symmetric related links and safe reviewed purge only. Verified base main `9298744c29f70f92ea3c1e60bae9bb31b481a2db`; isolated branch `feature/m5b3-recipe-variations-relations`, worktree `/private/tmp/tarif-atlasi-m5b3`. Candidate schema **11**, app **0.1.0**, Catalog Release **11 (854/269/50)**. Prior migrations/history, normal and retained verification profiles, workbook and other worktrees are protected. Earlier “M5B-3 not authorized” checkpoint statements below are superseded only for coding by DEC-027. IMPLEMENTED; local/native/final-head CI evidence is recorded in docs/M5B3_ACCEPTANCE.md and the feature PR. Not INTEGRATED or PO_ACCEPTED; no merge/tag/release or later-slice authority. M5B-1 Test4 discrepancy and M5B-2 historical follow-ups remain recorded, not retroactively passed.

## Current integrated milestone — M5B-2 (2026-11-02)

**INTEGRATED + scoped PO_ACCEPTED** via [PR #23](https://github.com/fatihemres/personal-recipe-library/pull/23), merge commit `e8fe359fba1a4514ebd887ef5e177d66cbe5544c` from final head `af16dc0f4e96e4ca719df4eaf885f4f9ed2f44d8`; [exact-head 5/5 CI PASS](https://github.com/fatihemres/personal-recipe-library/actions/runs/41420109075). Version list/history, read-only previews/comparison and transactional restore-as-new-version now exist on schema 10. PO reported isolated Mac acceptance successful and explicitly authorized integration separately. App `0.1.0`/Catalog Release 11 unchanged. M5B-3/4 remain PLANNED / not authorized; M5C/D likewise. Known UX/performance/Intel/Windows and earlier M5B-1 verification limitations remain open.

## Historical M5B-2 engineering candidate — 2026-11-02

Issue #22 / DEC-025 explicitly authorize implementation of history, comparison and safe restore-as-new-version. Base main `eb2cd73f6b1cbbc7b9687add3631d1a0894e7882`; branch `feature/m5b2-recipe-history-restore`. Schema **10**, Catalog **11 (854/269/50)**, app **0.1.0** unchanged. Not INTEGRATED or PO_ACCEPTED; local/native/CI evidence is maintained in [M5B2_ACCEPTANCE.md](docs/M5B2_ACCEPTANCE.md). Prior “M5B-2 not authorized” statements below describe their historical checkpoints, superseded by DEC-025 only for this slice. M5B-3/4 and later remain outside authorization. M5B-1 Test 4 remains a documented verification gap, not a proven defect or retroactive PASS.

# Tarif Atlası — Product delivery roadmap

**Checkpoint 2026-11-02 (M5B-1 integrated + scoped PO accepted).** The full V1 boundary comes from [MASTER_SPEC.md](docs/MASTER_SPEC.md); [IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) contains detailed scope/acceptance. This file records *progress*, not automatic development authorization. Verify current work remotely and inspect local changes before starting.

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
| M5 overall | Advanced editor, history/relations, media and Cooking Mode | **Partially INTEGRATED** (M5A, M5B-1/2/3; M5B-4 engineering candidate); parent [Issue #14](https://github.com/fatihemres/personal-recipe-library/issues/14) | M5B-1/2/3 integrated by PR #20/#23/#26; Issue #28 / DEC-029 authorize M5B-4 coding only; M5C/D remain separately gated |
| M5A | Simple-by-default advanced steps and own-recipe line associations | **INTEGRATED + PO_ACCEPTED** via [PR #16](https://github.com/fatihemres/personal-recipe-library/pull/16), merge `f80b41cf4e66fc3ef7a1dc8273cc98c829a27e83`; schema 9, [CI 40645726359](https://github.com/fatihemres/personal-recipe-library/actions/runs/40645726359) 5/5; isolated Mac PO 5/5 | Native Intel/Windows GUI, normal-profile upgrade and release gates unverified; no tag |
| **M5B** | Immutable committed-content versions/restore; independent variations, related recipes, explicit ingredient-line substitutions | **POLICY ACCEPTED (DEC-022); M5B-1/2/3 INTEGRATED + scoped PO_ACCEPTED; M5B-4 IMPLEMENTED candidate** | [P1–P10 policy contract](docs/M5B_POLICY.md). M5B-1 persisted history schema 10 via [PR #20](https://github.com/fatihemres/personal-recipe-library/pull/20), merge `c528014838ae1ce571032928cc8c81239bb234aa`, [CI 40845080044](https://github.com/fatihemres/personal-recipe-library/actions/runs/40845080044) 5/5; scoped PO acceptance records unresolved private-note/favorite evidence gap and skipped manual structured-step check (DEC-024). M5B-2 integrated via [PR #23](https://github.com/fatihemres/personal-recipe-library/pull/23), merge `e8fe359fba1a4514ebd887ef5e177d66cbe5544c`, CI 41420109075 5/5, scoped PO acceptance (DEC-026). M5B-3 integrated via PR #26; M5B-4 candidate under Issue #28 / DEC-029. All slices retain independent review/CI/acceptance/merge gates |
| M5C / M5D | Safe offline media ownership; Cooking Mode | **PLANNED; NOT AUTHORIZED TO CODE** | Physical media lifecycle/Cooking Mode policies and separate implementation gates remain open |
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
