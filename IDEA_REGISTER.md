# Tarif Atlası — Product idea register

This is the **canonical long-term IDEA REGISTER**. Ideas are *not approvals to build*. The V1 implementation obligations already mandated by [MASTER_SPEC.md](docs/MASTER_SPEC.md) belong primarily in [ROADMAP.md](ROADMAP.md); avoid duplicating every required feature as an idea. Dates reflect known sources, not invented origin dates.

## States and ownership
- `INBOX`: new PO suggestion not yet screened; `RESEARCH`: hypothesis/source discovery; `PROPOSED`: AI suggestion or option not PO-approved; `ROADMAP`: PO-approved scoped planning (not automatic implementation permission); `DEFERRED`: paused; `REJECTED`: declined; `DELIVERED`: implemented and accepted (linked PR/evidence).
- Prefix every idea with immutable `IDEA-###`. Keep source `PO` vs `AI` vs `SPEC` vs `RESEARCH` explicit and attach milestone/decision/PR only when real.
- If conversation tools cannot write GitHub immediately, label new items in ChatGPT **temporary idea inbox, not yet canonical**; bring them to this file via a reviewable documentation PR. Do not maintain a second canonical register.

## Registered ideas / future hypotheses
| ID | Origin and date | Idea | State (as of 2026-10-23) | Roadmap/decision | Important constraint |
| --- | --- | --- | --- | --- | --- |
| IDEA-001 | PO conversation, date 2026-10-22 | Resume broader international/Turkish and specialty drink **catalog expansion** at some later stage, only where evidence and licensing allow. | **DEFERRED** | [DEC-010](DECISIONS.md), Release 11 current | PO specifically paused growth to prioritize finishing app. This idea does **not** authorize M3D-6. |
| IDEA-002 | Original product specification, 2026-10-08; explicitly future-facing | Optional **AI assistance** after core V1 workflows are sound (e.g., ideas/organization). | **PROPOSED (V2 concept, not PO-approved to build)** | Product baseline future proposals | No mandatory online AI, no telemetry/accounts; requires explicit PO authorization and privacy review. |
| IDEA-003 | Original product specification, 2026-10-08; explicitly future-facing | Optional **cloud synchronization/accounts** for multi-device use. | **PROPOSED (V2 concept, not PO-approved to build)** | R18 local-first constraints | Cannot weaken offline-first or create mandatory accounts. |
| IDEA-004 | Original product specification, 2026-10-08; explicitly future-facing | Optional **online catalog refresh** with reviewed sources and safe deltas. | **PROPOSED (V2 concept, not PO-approved to build)** | DEC-006, DEC-010 | Must never silently change immutable snapshots or personal edits; current catalog expansion paused. |
| IDEA-005 | Original product specification, 2026-10-08; explicitly future-facing | Optional **barcode-scanning hardware workflow**. | **PROPOSED (V2 concept, not PO-approved to build)** | R08 | Keep product/barcode identities distinct from canonical ingredients; no automatically licensed sources. |
| IDEA-006 | Original product specification, 2026-10-08; explicitly future-facing | Optional **hosted video** experience after local media support. | **PROPOSED (V2 concept, not PO-approved to build)** | R17 | No mandatory hosting/network dependency; V1 need only optional video-reference architecture. |

## Intake protocol for future chats / agents
For a newly expressed idea:
1. Check this table and [ROADMAP.md](ROADMAP.md) for duplicates or an existing V1 requirement.
2. Capture concise title, origin (quoted PO conversation/source if available), date if known, benefit, constraints, suggested milestone and temporary state. **Do not implement.**
3. If not yet in the repo, say so and put it into a temporary working-context inbox until a permitted docs PR. Do not falsely claim memory persistence through model state alone.
4. Once PO decides, add a `DEC-###` record only for a material binding decision, and update idea status/roadmap link; preserve superseded/rejected history.

No one is authorized to green-light a feature purely by changing its idea-register state.
