# Tarif Atlası — Decision register

This is the **canonical register of material PO/product/engineering decisions**, not a change log or an automated assertion that every implementation is correct. Do not confuse historical architecture proposals with shipped code. Dates and authority below reflect explicit conversation decisions, product specification, or existing repo rules; no unknown dates have been invented.

Statuses: **ACCEPTED (PO)**, **ESTABLISHED (repo policy)**, **PROPOSED**, **OPEN**, **SUPERSEDED**. An idea or suggestion is not authorization to implement.

| ID | Date/source | Authority/status | Decision and consequences |
| --- | --- | --- | --- |
| DEC-001 | Product baseline 2026-10-08; `docs/MASTER_SPEC.md` | ACCEPTED (PO specification) | Offline/local-first macOS and Windows personal food/beverage library; no mandatory cloud, accounts, server, sync or telemetry. |
| DEC-002 | M1 architecture/production repo | ESTABLISHED (implemented architecture) | Tauri 2 + React/strict TypeScript/Vite UI; typed IPC; Rust services/repositories; SQLite/FTS5 as source of truth. Respect real code when documentation drifts. |
| DEC-003 | PO chat + `docs/MASTER_SPEC.md` | ACCEPTED (PO) | PO owns scope, architectural trade-offs, milestone acceptance, merge and releases. Coding-agent PASS, CI success, independent review and PO native acceptance are distinct. |
| DEC-004 | `docs/MASTER_SPEC.md` | ACCEPTED (PO specification) | R01–R20 are V1 unless expressly identified as V2. Do not silently drop pantry, shopping, planning, media, variations, backup, beverage editor or other V1 features. |
| DEC-005 | M2/M3 implementation and `docs/MEASUREMENT_ENGINE.md` | ESTABLISHED (data invariant) | Persist authoritative quantities as exact decimal text; keep units and unknown-null states. No invented g/mL equivalence, density or alcohol percentage. |
| DEC-006 | M3 source policy + [Release-8 incident](PROJECT_PROGRESS.md) | ESTABLISHED (data invariant) | Catalog source/version/checksum snapshots immutable; stable canonical IDs; upgrades transactional; user recipes, overrides, drafts and links preserved. Never repeat `CATALOG_SOURCE_CHANGED` by reusing altered source versions. |
| DEC-007 | `docs/DEVELOPMENT_AND_RELEASE_PROCESS.md`; enforced main ruleset | ESTABLISHED (repo policy) | One milestone per feature branch/PR, five mandatory CI jobs, protected `main`, no direct push, no unauthorized merge. Use **merge commits**, not squash/rebase, and verify resulting parent ancestry. |
| DEC-008 | Release 8–11 workflow and PO approvals | ACCEPTED (PO) | Stage feature → isolated tests/CI packages → real Mac acceptance → explicit exact-commit merge approval → remote verification → local FF sync → unique annotated checkpoint tag (not moved). |
| DEC-009 | App/release policy in `docs/DEVELOPMENT_AND_RELEASE_PROCESS.md` | ESTABLISHED (repo policy) | Application semver `0.1.0` differs from catalog Release 11 and `milestone/*` checkpoint tags. Do not publish `v1.0.0` or a GitHub Release without separate PO approval and complete V1 gates. |
| DEC-010 | **2026-10-22 PO conversation** | **ACCEPTED (PO)** | **Pause ingredient catalog expansion after Release 11** to focus on finishing application functionality. Historical pending candidates remain documented, not an instruction to start a new catalog release. |
| DEC-011 | **2026-10-23 PO conversation** and [PR #5](https://github.com/fatihemres/personal-recipe-library/pull/5) | **ACCEPTED (PO), INTEGRATED** | M4A real Mac 8/8 acceptance and explicit merge-commit approval for exact `f6441fe2...`; merged `49955722...`. |
| DEC-012 | Discussion following M4A; `docs/IMPLEMENTATION_PLAN.md` | **PROPOSED (not approved to code)** | Next delivery slice M4B: personal recipe organization and real dashboard, separate from M5/M6. This is planning, not evidence of work started. |
| DEC-013 | **2026-10-23 PO governance request** | **ACCEPTED (PO)** | GitHub is single auditable canonical engineering memory. ChatGPT Project is the decision/working area. Maintain a persistent idea register, status/decisions/roadmap/evidence without duplicated canonical copies or per-chat handoff churn. |

## Open decisions (not implicit approvals)
| ID | Owning scope | Question/gate |
| --- | --- | --- |
| OPEN-001 | M4B | Recipe rating scale, rich-note representation/sanitization, saved-filter persistence rules and collection/trash semantics. Decide during scoped M4B planning; preserve current data. |
| OPEN-002 | M5 | Version-history retention/restoration and media lifecycle policies. |
| OPEN-003 | M6 | Household/bar units, density evidence and conversion/default profiles; never invent universal shot/cup factors. |
| OPEN-004 | M7 | Pantry stock deduction confirmation and expired stock policy. |
| OPEN-005 | M9–M11 | Complete portable backup/restore compatibility; minimum OS support, native Windows/Intel GUI acceptance, installer trust, signing/notarization and V1 release gates. |
| OPEN-006 | Git/PO desktop | Verify local post-M4A fast-forward, untracked review workbook and checkpoint tag before creating M4B branch. |

## Rules for changing this register
- On a material PO decision, append a new dated unique `DEC-###` or explicitly mark the old entry **SUPERSEDED** with a link; do not silently rewrite history.
- State source (PO message, issue, PR or doc). Record unknown date as unknown.
- Project proposals, AI suggestions and research hypotheses belong in [IDEA_REGISTER.md](IDEA_REGISTER.md) until a binding decision is made.
- Keep detailed validation in [VERIFICATION.md](VERIFICATION.md), milestones in [ROADMAP.md](ROADMAP.md), and historical narrative in [PROJECT_PROGRESS.md](PROJECT_PROGRESS.md).
