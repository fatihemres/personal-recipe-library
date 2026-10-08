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
- Preserve migration 001 and 002 after this milestone; future schema work adds version 003 or later.
- Measurements are decimal text, never JS Number/SQLite REAL; preserve selected units and unknown quantity NULL. Consult docs/MEASUREMENT_ENGINE.md before conversion/scaling changes.
- Recipe saves use the Rust worker/repository transaction and expected revisions. Do not bypass referential integrity or introduce renderer persistence as an alternative source of truth.
- Personal ingredient identity is independent of recipes; catalog upgrades must preserve it.
- Durable drafts/duplication/archive remain M2B. Basic trash/restore already exists; do not silently drop it.
