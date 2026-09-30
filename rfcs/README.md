# Enkrnett RFCs

Decisions and their design. The lifecycle follows
[RFC 000](./done/000-rfc-lifecycle-policy.md), 5-folder variant.

## Folders

| Folder | State | Meaning |
|---|---|---|
| `draft/` | Draft | Being written. |
| `proposed/` | Proposed | Open for review. Do not implement. |
| `accepted/` | Accepted | Review complete. Implementation may start. |
| `done/` | Implemented | Shipped. Kept as the record. |
| `archive/` | Withdrawn or superseded | Kept as the record. |
| `handoffs/NNN-slug/` | — | How to carry out an RFC: implementation, test, and other tasks such as release preparation. Its state follows the RFC. |

The folder is the state. Moving a file changes its state. The `Status` field in the file and this
index change in the same commit. A folder is created when its first file arrives.

## Proposed

None.

## Accepted

| ID | Title | Accepted | Handoff |
|---|---|---|---|
| 001 | [Scope, people, and vocabulary](./accepted/001-scope-people-vocabulary.md) | 2026-09-29 | — |
| 002 | [Network topology and separation](./accepted/002-network-topology-and-separation.md) | 2026-09-29 | — |
| 003 | [Management channel and UI delivery](./accepted/003-management-channel-and-ui-delivery.md) | 2026-09-29 | — |
| 004 | [Ownership, credentials, and authorization](./accepted/004-ownership-credentials-authorization.md) | 2026-09-29 | — |
| 005 | [Devices, OpenWrt baseline, and installation](./accepted/005-devices-baseline-installation.md) | 2026-09-29 | — |
| 006 | [Feasibility measurements](./accepted/006-feasibility-measurements.md) | 2026-09-29 | [Handoff](./handoffs/006-feasibility-measurements/README.md) |
| 007 | [Threat model](./accepted/007-threat-model.md) | 2026-09-30 | — |
| 008 | [Where OpenBSD fits](./accepted/008-openbsd-position.md) | 2026-09-30 | — |

## Implemented

| ID | Title | Shipped in |
|---|---|---|
| 000 | [RFC lifecycle policy](./done/000-rfc-lifecycle-policy.md) | Adopted at project start |

## Archive

None.

## Planned

Not yet written. A number is assigned when the file is created.

| Subject | Depends on |
|---|---|
| Configuration model and transactions | Measurement M-5 |
| OWE behavior and security status | Measurement M-4 |
| Footprint budgets | Measurements M-2, M-3 |
| Build, release, update, and upgrade | RFC 005; measurement M-3 |
| Management API | The RFCs above |
| Operator UI | The RFCs above; measurements M-1, M-7 |
