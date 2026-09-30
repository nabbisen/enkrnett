# Roadmap

**State.** Draft, waiting for the owner's authorization.
**Updated.** 2026-09-30

Order of work, not a calendar. Dates are set when milestone M0 ends, because its measurements
decide how much work the later milestones contain.

## Milestones

| Milestone | Version | Goal | Ends when |
|---|---|---|---|
| **M0 Foundations** | — | Decisions, measurements, and the specification baseline | All RFCs listed under M0 are accepted |
| **M1 Skeleton** | 0.1.0 | Daemon and management UI run together on the virtual machine and on the Tier 1 device. A device can be claimed and reset. | The tests derived from RFC 003 and RFC 004 pass |
| **M2 Guest Wi-Fi** | 0.2.0 | The operator configures the guest network. Separation and hardening are in place. The status says what is protected and what is not. | The tests derived from RFC 002 and the OWE and configuration RFCs pass |
| **M3 Installable** | 0.3.0 | A non-expert installs, sets up, updates, and resets a Tier 1 device with the guides. English and Japanese. | The usability test passes; update and upgrade are tested |
| **M4 Constrained device** | 0.4.0 | An 8 MB / 64 MB device meets the budgets. | Measured on the device; the owner decides on promotion |
| **First release** | 1.0.0 | Release for operators | Release checklist below |

M3 cannot publish an image for a device sold in Japan until the legal basis is documented
(RFC 005, ENK-RF-005).

## M0 in detail

| Step | Content | Who | State |
|---|---|---|---|
| 1 | Gap analysis of the drafts | Architect | Done |
| 2 | Round 1 decisions | Owner | Done |
| 3 | RFC 001 to 006 proposed | Architect | Done |
| 4 | RFC 001 to 006 accepted | Owner | Done |
| 5 | Measurements M-6, M-5, M-2 | Development team | **Ready to start** |
| 6 | Measurement M-1, with the Pixel phone and the iPad | Development team prepares; owner observes | **Ready to start** |
| 7 | RFC 007 Threat model accepted | Owner | Done |
| 8 | Inquiry on radio regulation | Owner | The owner will start it. It may take time. |
| 9 | Candidate routers obtained, and a lawful basis for operating them with replaced firmware | Owner | **Waiting** |
| 10 | Measurements M-4, M-8, M-3, M-7 | Holder of the hardware; owner | Blocked by 9 |
| 11 | Round 2 decisions | Owner, prepared by the architect | Blocked by 5, 6, 10 |
| 12 | Remaining baseline RFCs proposed and accepted | Architect; owner | Blocked by 11 |
| 13 | Consolidated specification in `docs/src/` | Architect | Blocked by 12 |
| 14 | Repository layout; handoffs for M1 | Architect | Blocked by 12 |

## RFC order

| Order | RFC | State | Needed for |
|---|---|---|---|
| 1 | 001 Scope, people, and vocabulary | Accepted | Everything |
| 2 | 002 Network topology and separation | Accepted | M2 |
| 3 | 003 Management channel and UI delivery | Accepted | M1 |
| 4 | 004 Ownership, credentials, and authorization | Accepted | M1 |
| 5 | 005 Devices, OpenWrt baseline, and installation | Accepted | M1, M3 |
| 6 | 006 Feasibility measurements | Accepted | Round 2 |
| 7 | 007 Threat model | Accepted | M1 |
| 8 | Footprint budgets | Planned | M1 |
| 9 | Configuration model and transactions | Planned | M2 |
| 10 | OWE behavior and security status | Planned | M2 |
| 11 | Management API | Planned | M1 |
| 12 | Operator UI | Planned | M1 |
| 13 | Build, release, update, and upgrade | Planned | M3 |

## Release cycle

| Subject | Rule |
|---|---|
| Version numbers | Semantic versioning. Tags carry no "v" prefix. |
| Before 1.0.0 | Each milestone ends with one release. These are for development and testing, not for operators. |
| OpenWrt baseline | One per Enkrnett release. The first is 25.12. |
| Every release | Security review. Threat model updated when data flows, integrations, or authorization changed. Documentation checked against the accepted RFCs. `CHANGELOG.md` updated. |
| After 1.0.0 | Cadence is decided with the update strategy in Round 2. |

## Release checklist for 1.0.0

| # | Item |
|---|---|
| 1 | Every requirement allocated to the first release is verified by the method its RFC names. |
| 2 | At least one Tier 1 device meets RFC 005 §2. |
| 3 | The threat model is current. |
| 4 | Install, setup, update, and reset guides exist in English and Japanese, and non-experts completed them. |
| 5 | Every published file can be rebuilt from the repository and its pinned inputs. |
| 6 | The legal basis for each market in which images are published is documented. |
