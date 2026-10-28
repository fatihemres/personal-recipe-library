## Current agent handoff (2026-10-27)

Read [PROJECT_STATUS.md](PROJECT_STATUS.md), [DECISIONS.md](DECISIONS.md), [ROADMAP.md](ROADMAP.md), [VERIFICATION.md](VERIFICATION.md), [IDEA_REGISTER.md](IDEA_REGISTER.md) and [docs/DEVELOPMENT_AND_RELEASE_PROCESS.md](docs/DEVELOPMENT_AND_RELEASE_PROCESS.md) **before** proposing a new milestone or PR. Live protected `main` at 2026-10-27: `fb944f87c3c898bcf69b3e889bfa3cc8a39748c4`. M4A, M4B-1 and M4B-2 are **PO ACCEPTED + INTEGRATED** (M4B-2 [PR #11](https://github.com/fatihemres/personal-recipe-library/pull/11), schema 8, exact-head CI 5/5, isolated Mac functional and 5/5 responsive PO visual PASS). Catalog Release 11 remains 854 canonical ingredients, app semver 0.1.0, further catalog expansion paused. M4B-1 annotated tag exists; **M4B-2 checkpoint tag NOT created**. [M5 Issue #14](https://github.com/fatihemres/personal-recipe-library/issues/14) was planning-only at that checkpoint. The PO subsequently explicitly authorized **M5A only** on 2026-10-27: simple-by-default optional advanced steps, own-recipe ingredient-line links and safe legacy drafts (DEC-019). The active `feature/m5a-advanced-recipe-steps` candidate adds migration 009/schema 9 from verified main `1edfec3...`; PO reported 5/5 native Mac M5A acceptance PASS (DEC-020), and requested review/update/merge of open PRs (2026-10-28). This checkpoint precedes integration: verify live main and exact final-head five-job CI before a merge commit. M5B/M5C/M5D/M6/M8 remain outside scope. A separate open [proprietary rights PR #13](https://github.com/fatihemres/personal-recipe-library/pull/13) exists; do not merge or overwrite it. Distinguish PLANNED/IMPLEMENTED/VERIFIED/INTEGRATED/PO-ACCEPTED, inspect actual Git and dirty worktrees, protect root untracked workbook `INGREDIENT_TERMINOLOGY_REVIEW.xlsx` and daily Mac SQLite profile. No automatic merge/tag/release; protected main requires five CI jobs, merge commits and separate explicit PO merge approval.

---

# Project guidance

## Objective
Build a local-first food and beverage recipe management desktop application for macOS and Windows using Tauri 2, React, TypeScript, and SQLite.

## Required planning context
- Before coding or making architectural decisions, read `docs/MASTER_SPEC.md`, `docs/ARCHITECTURE.md`, `docs/IMPLEMENTATION_PLAN.md`, and `PROJECT_PROGRESS.md`. Consult schema, data dictionary, measurement, licensing, testing, and release documents as they are introduced.
- MASTER_SPEC.md is the scope authority. All requested functional modules remain V1; optional use does not imply deferral. Record any user-approved scope change explicitly.
- Work on the requested milestone only. Each increment must leave the application runnable and meet its acceptance/test gates before being marked complete.
- Use real storage/data and verified catalog sources. Mocks in isolated presentation tests never establish native feature completion. Do not fabricate seed coverage, nutrition, ABV, provenance, or dashboard values.
- Record architectural decisions and migration/backup compatibility changes in the relevant documents; preserve personal edits during seed updates. Review source licenses before redistribution.

## Working rules
- Inspect the existing project and working tree before changing anything. Extend the existing starter; do not regenerate or overwrite it.
- Keep each task small and within its requested scope. Do not implement the whole roadmap in one task.
- Preserve unrelated user changes, existing configuration, and lockfiles. Use npm for the existing npm-managed frontend.
- Read PROJECT_PROGRESS.md before starting and update it after completing a milestone with changes, validation, limitations, and the next step.
- Keep recipe data local by default. Do not add cloud accounts, synchronization, or network dependencies to core recipe workflows without a scoped requirement.
- Do not store user databases, backups, or secrets in the repository. Resolve application data paths through Tauri rather than hardcoding OS paths.

## PR-based development and release safety (DEVOPS-1)
- Read `docs/DEVELOPMENT_AND_RELEASE_PROCESS.md` before branch, CI, tag, merge, or release work. `origin/main` is the integration branch; never push a milestone directly to it.
- Start future milestones from a verified, fast-forwarded local `main` on a dedicated feature branch. Keep pre-existing local/untracked work, especially the root `INGREDIENT_TERMINOLOGY_REVIEW.xlsx`, untouched unless separately authorized.
- Run scoped local checks, commit the milestone, push only its feature branch, open a PR to `main`, inspect all CI jobs and artifacts, fix failures, and obtain explicit Product Owner review before merging. Do not auto-merge.
- Use a merge commit for the one-time release-8/CI transition PR to preserve M3D-2A and M3D-2B ancestry. Do not reset, force-push, rewrite history, or delete unmerged branches. Add unique annotated milestone tags only after a verified merge; catalog release numbers and application semver are separate.
- The repository became public after the earlier private-plan limitation; `main` now has mandatory PRs and five required Actions checks. Never bypass protection, push directly to main, merge without Product Owner approval, or expose personal data in public commits.

## Proposed module boundaries
- React: application shell, feature UI, and reusable components. Keep domain types and Tauri calls outside presentation components.
- Rust: thin Tauri commands, domain validation/services, and a separate SQLite persistence layer with versioned migrations.
- SQLite: authoritative durable storage. Use parameterized queries and transactions for related writes; protect existing data during migrations.
- Keep IPC contracts typed and errors useful to the UI. Grant Tauri capabilities only as required by implemented features.
- Introduce modules when a scoped feature needs them; do not create empty scaffolding for every planned feature.

## Validation
- Frontend: `npm run typecheck`, `npm run lint`, `npm test`, `npm run build`; `npm run test:ui` for applicable browser interactions. Browser tests do not prove native persistence.
- Backend: `cargo check --locked --manifest-path src-tauri/Cargo.toml`; add `--offline` when cached dependencies are sufficient.
- For Rust changes, use `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `cargo test --locked --manifest-path src-tauri/Cargo.toml`, and Clippy with warnings denied. Never edit an applied migration; add a new version. Add meaningful persistence tests when introducing SQLite.
- Desktop smoke test: `npm run tauri -- dev` when runtime behavior changes.
- Validate macOS and Windows on their respective hosts before claiming cross-platform readiness. A Rust check is not a desktop runtime or installer test.
- Do not install dependencies, upgrade versions, or change packaging merely to complete a documentation task.

## Recipe foundation invariants (M2A)
- Preserve migrations 001–004 after M3A; future schema work adds version 005 or later.
- Measurements are decimal text, never JS Number/SQLite REAL; preserve selected units and unknown quantity NULL. Consult docs/MEASUREMENT_ENGINE.md before conversion/scaling changes.
- Recipe saves use the Rust worker/repository transaction and expected revisions. Do not bypass referential integrity or introduce renderer persistence as an alternative source of truth.
- Personal ingredient identity is independent of recipes; catalog upgrades must preserve it.
- M2B drafts are separate SQLite records with session revisions and immutable recipe base revisions. Serialize autosave/Save/Discard; commit the aggregate and close its draft in one transaction. Retain closed-session tombstones to reject delayed writes.
- Archive, trash and active states are distinct. Preserve drafts across state changes and purge; orphan edit drafts require explicit save-as-new. Never delete shared ingredient definitions through recipe purge.
- Ingredient references from saved recipes and active drafts restrict deletion. Renames preserve IDs; conflicts never silently merge or reassign references. Search uses NFC/Turkish normalization on both query and stored key, not SQLite NOCASE.
- Native close/Cmd-Q must await the current recipe draft flush and keep the window open on failure. Last changes before an unacknowledged abrupt crash are not guaranteed durable.
- M3 catalog work remains a separate task; never seed invented ingredients to satisfy search verification.

## Catalog foundation invariants (M3A)
- Consult docs/INGREDIENT_CATALOG_SCHEMA.md, docs/CATALOG_IMPORT_PIPELINE.md, docs/DATA_SOURCES_AND_LICENSES.md and catalog/sources.json before catalog changes.
- Validation version 1 contains eight real USDA-backed ingredients; it is not the production catalog and never auto-installs into personal app data. Extensive population/bundled installation is M3B; management/discovery UI is M3C.
- Canonical keys/UUIDs and source mappings are stable. Preserve recipe-facing personal IDs, references and catalog overrides. Name collisions require review; never merge implicitly. Changed system names that collide with personal records reject the package transaction for review.
- Sources require reviewed license/version clearance, checksummed evidence and accurate attribution. Unknown observations stay absent. Checksums verify integrity, not authenticity or translation quality.
- Catalog data and success audit commit together; interrupted running audits are not success. Re-import unchanged packages is idempotent; changed packages need a greater version. Never mutate an applied migration or existing source/version snapshot.

## Production catalog invariants (M3B-1)
- Consult docs/CATALOG_COVERAGE_PLAN.md and catalog/production/coverage.json before expanding coverage. Production version 2 is a first batch, not complete M3B.
- Keep validation fixtures separate and unchanged. Embedded production seed installs via the Rust worker; never bypass the shared checked transactional importer.
- Production extracts have explicit immutable source snapshot versions. Preserve canonical IDs, personal collisions and overrides; do not infer observations from names or proof labels.
- Reproduce tools/prepare_production_catalog.py with the pinned archive and curated descriptor assertions. New content requires a higher dataset version and reviewed source clearance.

## Expanded catalog invariants (M3B-2)
- Current production version 4 has 508 identities; historical versions 2 and 3 live in catalog/releases/2 and catalog/releases/3. Never mutate committed source snapshots or reassign canonical IDs.
- Consult docs/CATALOG_COVERAGE_REPORT.md, docs/INGREDIENT_LOCALIZATION_REVIEW.md and catalog/reference-sources.json. Project-only factual entries are not USDA records and reference-only websites are not approved bulk datasets.
- Reproduce grouped curation with tools/prepare_production_catalog.py; run tools/audit_catalog.py --check and tools/test_catalog_audit.py. Human review remains distinct from structural/semantic tooling.

## Ingredient library invariants (M3C-3)
- Schema 5 and production catalog version 4 are current; no M3C-3 migration or seed change occurred. Existing personal category links are owned by `personal_category_memberships` and edited atomically with expected-revision personal changes. Canonical category memberships remain source-owned.
- Candidate reasons express name/alias similarity, not proven ingredient equivalence. Never auto-merge or reassign recipe IDs. A strong name/alias match blocks misleading personal creation; ambiguous variants need explicit user judgment.
- The M3C-3 audit recorded 18 then-pending terminology items. Its `docs/CATALOG_M3C3_QUALITY_AUDIT.md` and `catalog/production/m3c3-quality-report.json` are immutable historical outputs, not linguistic approval; `tools/report_m3c3_quality.py --check` reads archived version 4 inputs.

## Terminology release invariants (M3C-4)
- Production catalog version 5 retains 508 canonical IDs, 260 aliases and 50 categories. Only three explicit user-selected Turkish names changed; 15 terminology decisions remain pending. See `docs/CATALOG_M3C4_QUALITY_AUDIT.md` and the review workbook.
- Version 4 is archived in `catalog/releases/4`. Generate version 5 only with `tools/apply_terminology_decisions.py`; never edit version 4, previously applied migrations, or historical source snapshots. The new terminology provenance is separate from USDA and old curation evidence.
- Old Turkish labels were not promoted to aliases; almond butter stays distinct from almond paste/marzipan. Catalog import preserves user name/unit overrides and personal records. An upgrade colliding with an unlinked personal name fails transactionally for explicit review.

## Final terminology release invariants (M3C-5)
- Production catalog version 6 retains 508 canonical IDs, 260 aliases and 50 categories. Version 5 is archived at `catalog/releases/5`; generate version 6 only with `tools/apply_final_terminology_decisions.py` from its pinned checksum.
- Exactly two canonical name fields changed: `usda-sr-174842` Turkish and `project-siyez-grain` English (including its matching `canonicalName`). Ten reviewed names were expressly retained and three remain unresolved; no independent expert linguistic verification is claimed. See `docs/CATALOG_M3C5_QUALITY_AUDIT.md` and the review workbook.
- Preserve historical evidence, IDs, aliases, user overrides, personal records, favorites, drafts and recipe quantities. Catalog update collisions still reject the whole transaction. Migrations 001–005 remain unchanged.

## Expansion release invariants (M3D-1)
- Historical production release 7 has 568 identities, 269 aliases and 50 categories; schema stays 5. Earlier version/count sections are historical. Release 6 is archived unchanged at `catalog/releases/6`.
- Reproduce with `tools/prepare_m3d1_catalog.py` using the pinned USDA archive or `--evidence` for checked offline extracts. Run `tools/report_m3d1_quality.py --check`, catalog audits and SQLite upgrade tests. Preserve all 508 complete baseline ingredient objects.
- The 61 proposed candidates yielded 60 additions. `usda-sr-169406` is deferred for possible overlap with existing raw tahini; never silently install or merge it. Three new naming questions plus the deferred identity are in `docs/INGREDIENT_M3D1_REVIEW.xlsx`. The three historical unresolved terms remain pending.
- Leave the pre-existing root `INGREDIENT_TERMINOLOGY_REVIEW.xlsx` untouched and untracked unless the user explicitly authorizes work on it.

## Curated expansion invariants (M3D-2B)
- Historical production release 8 has 647 identities, 269 aliases and 50 categories; schema remains 5. Release 7 and the M3D-2A protected baseline are archived unchanged at `catalog/releases/7`.
- Generate/check release 8 only through `tools/prepare_m3d2b_catalog.py`. `catalog/production/m3d2b-decisions.json` records all 194 candidate dispositions: 79 USDA accepted, 27 USDA deferred, 84 FoodOn source deferrals, three probable existing FoodOn concepts, one brand-specific rejection. These are autonomous editorial decisions, not human linguistic approvals.
- New source IDs are `usda-sr-legacy-m3d2b/2018-04-selection-6` and `recipeatlas-curation-m3d2b/production-5`. FoodOn remains outside the runtime source allowlist until term-level lineage/attribution/redistribution clearance. No new aliases, categories, factual observations or migrations were added.
- Preserve all 568 complete release-7 ingredient objects, stable recipe references and user state. Before later expansion, read `docs/CATALOG_M3D2B_QUALITY_AUDIT.md`, `docs/CATALOG_M3D2B_COVERAGE.md`, `docs/CATALOG_M3D2B_SOURCE_PLAN.md` and the seven prior pending review items. Future additions require a higher release and the same isolated fresh/upgrade/rollback/native gates.

## Global and regional expansion invariants (M3D-3/M3D-4)
- Historical Release 9 has 745 identities (624 food/121 beverage), 269 aliases and 50 categories, and is archived unchanged at `catalog/releases/9`; Release 8 is at `catalog/releases/8`.
- Current Release 10 has 795 identities (666 food/129 beverage), 269 aliases and 50 categories. Its 50 additions come from a screened 65-candidate ledger: 35 selected USDA Foundation April 2026 descriptors and 15 pinned Wikidata CC0 entity revisions. See `docs/CATALOG_M3D4_SOURCE_EVALUATION.md`, `docs/CATALOG_M3D4_QUALITY_AUDIT.md` and `catalog/preparation/m3d4` before any later source change.
- Generate/check only through `tools/prepare_m3d4_candidates.py` and `tools/prepare_m3d4_catalog.py`. Source/version/checksum pairs are immutable. Keep 9 new deferrals, 29 earlier deferrals and seven prior review questions pending; do not turn autonomous editorial labels into claims of independent human linguistic approval.
- FoodOn, Open Food Facts, TürKomp and TheCocktailDB remain outside the runtime seed. The protected GI registry is reference-only; a generic catalog label does not certify product origin. No prior migration, ingredient ID, user-owned record, alias, category node or factual observation was changed.
