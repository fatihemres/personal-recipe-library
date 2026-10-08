# Changelog

## Unreleased — Milestone 1 foundation

- Turkish responsive desktop shell, keyboard navigation, accessible light/dark/system theme choices, honest upcoming-module states and localized errors.
- Real app-data SQLite, bundled FTS5 probe, foreign keys, WAL, checksummed transactional migrations and pre-upgrade online snapshots.
- Dedicated Rust database worker, typed/validated IPC, durable theme preferences and initialization retry without data reset.
- Rust storage/command tests, renderer/contract tests, Playwright browser checks, lint/typecheck tooling and native macOS debug-bundle verification.
- Architecture/schema/data dictionary/testing/checklist/handoff documentation. No application features beyond foundation are enabled; no public release/tag is published.

## M2A — Persistent Recipe Management (2026-10-08, unreleased)

- Real SQLite recipe create/read/edit/list, independent custom ingredients with Turkish Unicode search and normalized duplicate reuse.
- Decimal-safe quantity strings and six standard unit references; stable ordered ingredient lines and preparation steps.
- Turkish editor/detail/library, kind/title filters, confirmed soft deletion, trash and restore, validation/error states and in-memory edit preservation across Settings navigation.
- Additive checksummed migration 002, revision conflicts, transactional rollback and FK safeguards; M1 preferences/migration/FTS infrastructure retained.
- Persistence/IPC/UI regression tests and packaged native macOS restart verification. M2B and later modules remain pending.
