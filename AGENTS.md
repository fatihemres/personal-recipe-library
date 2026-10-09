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
