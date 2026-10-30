# Roadmap

**State.** Draft, waiting for the owner's authorization.
**Updated.** 2026-10-31

Order of work, not a calendar. Dates are set when milestone M0 ends, because its measurements
decide how much work the later milestones contain.

## Milestones

| Milestone | Version | Goal | Ends when |
|---|---|---|---|
| **M0 Foundations** | — | Decisions, measurements, and the specification baseline | All RFCs listed under M0 are accepted |
| **M1 Skeleton** | 0.1.0 | Daemon and management UI run together on the virtual machine and on the Tier 1 device. A device can be claimed and reset. First task: a browser harness in GitHub Actions (iOS Simulator, Android emulator) that loads the management UI from a private address against a mock daemon (owner decision 2026-10-29). | The tests derived from RFC 003 and RFC 004 pass |
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
| 6 | Measurement M-1, with the Pixel phone and the iPad: the seven network rows by hand; the page rows move to the M1 browser harness | Kit prepared by the architect; owner observes | **Owner, now** |
| 7 | RFC 007 Threat model accepted | Owner | Done |
| 8 | Inquiry on radio regulation | Owner | The owner will start it. It may take time. |
| 9 | Candidate routers obtained (one `standard`, one `constrained`, RFC 005 §7); a lawful basis for switching their radios on | Owner | **Owner buys the routers now; the radios wait for the legal answer** |
| 10a | M-3 part 1 (image fits) and M-8 part 1 (separation) on VMs | Architect | Done 2026-10-29 |
| 10c | M-4a: OWE with the Pixel and the iPad against the workstation's Wi-Fi card | Architect prepared the kit; owner runs it | **Owner, when convenient** (`.git-exclude/tmp/m/m4a/README.md`) |
| 10b | Measurements M-4, M-8, M-7 and the rest of M-3 | Holder of the hardware; owner | Blocked by the legal answer |
| 11 | Round 2 decisions | Owner | Done (D-08/D-10/D-12 by RFC 010–012; D-11/D-13/D-14/D-15 accepted 2026-10-29) |
| 12 | RFC 013 (build, release, update) and 014 (platform findings) | Architect; owner | Accepted 2026-10-31 |
| 12a | RFC 015 Management API | Architect; owner | Accepted 2026-10-31 |
| 12c | Review of RFC 001–015 against the owner's philosophy | Architect | **In progress** (owner's instruction 2026-10-31) |
| 12b | RFC Operator UI | Architect; owner | After 12c |
| 13 | Consolidated specification in `docs/src/` | Architect | After 12a and 12b |
| 14 | Repository layout (RFC 013 §5); handoffs for M1 | Architect | Layout may start now; handoffs after 12a |

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
| 8 | 009 Vocabulary amendment | Accepted | Everything |
| 9 | 010 Footprint budgets | Accepted | M1 |
| 10 | 011 Configuration model and transactions | Accepted | M2 |
| 11 | 012 Guest network modes and protection status | Accepted | M2 |
| 12 | 013 Build, release, update, and upgrade | Accepted | M3 |
| 13 | 014 Platform findings that Enkrnett must handle itself | Accepted | M2 |
| 14 | 015 Management API | Accepted | M1 |
| 15 | Operator UI | Planned (after the review of 001–015) | M1 |

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
