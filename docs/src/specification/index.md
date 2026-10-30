# Specification

The consolidated specification of Enkrnett. It restates the accepted RFCs with every amendment
applied, so that a reader needs no pointer notes. Where this book and an RFC differ, the newer
accepted RFC wins (RFC 001 §5.1); the difference is a defect of this book and is corrected.

**Baseline.** RFC 001 to 017, all accepted by 2026-10-31. Decisions and their reasons stay in
`rfcs/`; this book holds what is built and how it is verified.

## How to read a requirement

| Column | Meaning |
|---|---|
| ID | `ENK-<AREA>-<NNN>`. Never reused, never renumbered. Areas and the reserved draft areas: chapter 1 §4. |
| Requirement | One sentence with one keyword: **shall** (required), **should** (expected; deviation needs a written reason), **may** (permitted). |
| Verification | Test, measurement, inspection, usability test, or review. Tests are derived from this text, not from the code. |
| RFC | The RFC that introduced the requirement, and the RFCs that amended it. |
| Release | The first release that shall meet it. `0.1.0` M1 Skeleton, `0.2.0` M2 Guest Wi-Fi, `0.3.0` M3 Installable, `0.4.0` M4 Constrained device, `1.0.0` first release. A requirement about a function that arrives later is verified when the function arrives. |

## Chapters

| # | Chapter | Areas | From RFC |
|---|---|---|---|
| 1 | [Scope, people, vocabulary](./01-scope-vocabulary.md) | SCOPE, WORD | 001, 009, 016, 017 |
| 2 | [Networks and separation](./02-networks-separation.md) | TOPO, ISO, NET, HARD | 002, 007, 014, 016 |
| 3 | [Management channel](./03-management-channel.md) | MGMT, SEC 009–016 | 003, 007 |
| 4 | [Ownership and sessions](./04-ownership-sessions.md) | OWN, AUTH | 004, 007, 009, 015, 016 |
| 5 | [Devices, baseline, installation](./05-devices-installation.md) | TIER, INST, BASE, RF | 005, 016 |
| 6 | [Settings and changes](./06-settings-changes.md) | CFG | 011, 014, 015, 016 |
| 7 | [Guest modes and status](./07-guest-modes-status.md) | OWE, GUEST | 012, 016 |
| 8 | [Footprint budgets](./08-footprint-budgets.md) | FOOT | 010, 016 |
| 9 | [Management API](./09-management-api.md) | API, SEC 017–022 | 015, 016 |
| 10 | [Operator UI](./10-operator-ui.md) | UI | 017 |
| 11 | [Build, release, update](./11-build-release-update.md) | RLS, UPD, BUILD, REPO | 013, 016 |
| 12 | [Threat model](./12-threat-model.md) | — | 007, 016 |
| 13 | [Requirements index](./13-requirements-index.md) | all | — |
| 14 | [Verification plan](./14-verification-plan.md) | all | project rules; RFC 001 §5.5 |

## Counts

| Requirements | Threats | Remaining risks |
|---:|---:|---:|
| 241 | 44 | 14 |

## Maintenance

When an RFC that amends an accepted RFC is accepted, the amended passage is rewritten here in
the same commit (RFC 016 ENK-SCOPE-006). The requirement tables under `requirements/` are plain
Markdown, included by the chapters; edit them in place.
