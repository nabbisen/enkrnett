# Handoff — RFC 006 Feasibility measurements

**Entry point.** Start here.
**RFC.** [`006-feasibility-measurements`](../../accepted/006-feasibility-measurements.md)
**State.** Follows the RFC. The RFC was accepted on 2026-09-29. Work may start.

## What this is

Eight measurements. Each answers one question with numbers and observations. None produces
product code.

## Read first

| Order | Document | Why |
|---|---|---|
| 1 | [RFC 006](../../accepted/006-feasibility-measurements.md) | The questions, the rules, the thresholds |
| 2 | [RFC 001](../../accepted/001-scope-people-vocabulary.md) §4 | The vocabulary. Use these words in reports. |
| 3 | [RFC 002](../../accepted/002-network-topology-and-separation.md) | What M-3, M-4, M-8 configure and test |
| 4 | [RFC 003](../../accepted/003-management-channel-and-ui-delivery.md) | What M-1 tests |
| 5 | [RFC 005](../../accepted/005-devices-baseline-installation.md) §7 | Candidate devices |

## Files in this handoff

| File | Content |
|---|---|
| [implementation-handoff.md](./implementation-handoff.md) | Setup, procedure, and data to record, per measurement |
| [acceptance-qa-checklist.md](./acceptance-qa-checklist.md) | When a report is complete |
| [report-template.md](./report-template.md) | The form of a report |

## Who does what

| Measurement | Needs | Prepared by | Carried out by | Can start |
|---|---|---|---|---|
| M-6 Simulated radios | Virtual machine | Development team | Development team | **Now** |
| M-5 Confirm-or-revert | Virtual machine | Development team | Development team | **Now**, after M-6 |
| M-2 Daemon footprint | Build machine; virtual machine | Development team | Development team | **Now** |
| M-1 Phone browsers | Phone and tablet; a computer that serves the test page | Development team | Owner | When the preparation is delivered |
| M-4 OWE on candidates | Candidate devices; client devices | Development team | Holder of the devices | Blocked: see below |
| M-8 Separation | Candidate device; two client machines | Development team | Holder of the devices | Blocked: see below |
| M-3 Complete image | A `constrained` device | Development team | Holder of the devices | Blocked: see below |
| M-7 Installation by non-experts | Candidate devices; participants | Architect (guide) | Owner | Blocked: see below |

## Hardware at hand

| Item | State |
|---|---|
| Android phone | Google Pixel |
| Apple device | iPad. It stands in for an iPhone in M-1. What an iPad cannot show is listed in the procedure. |
| Computer for M-1 | The owner's workstation. Its Wi-Fi adapter supports access-point mode with one network at a time. |
| Candidate routers | **None yet.** |

## What blocks M-3, M-4, M-7, and M-8

| # | Condition |
|---|---|
| 1 | A candidate router of RFC 005 §7 is at hand. |
| 2 | Operating that router with replaced firmware is lawful at the place of measurement (rule 8). |

Both must be met. Until then these four measurements do not start.

## Rules

| # | Rule |
|---|---|
| 1 | Use OpenWrt **25.12.5**. Record the version of every package in every report: OpenWrt's package feeds change without a new release number. |
| 2 | Keep code and images written for a measurement outside the repository, or under `.git-exclude/tmp/`. Do not commit them. |
| 3 | Do not change an RFC. If a result contradicts an RFC, or an RFC cannot be followed, stop that measurement and report it. |
| 4 | Report what was observed, including what failed. Do not judge against the thresholds. |
| 5 | Use the vocabulary of RFC 001. |
| 6 | Do not connect a device under test to a network that others depend on. |
| 7 | Set the country of every radio to the country where the measurement takes place, before any transmission. |
| 8 | Do not switch on the radio of a device whose firmware was replaced unless the owner has confirmed that this is lawful at the place of measurement. M-6, M-5, and M-2 use no radio. M-1 uses the workstation's adapter with its own driver. |
| 9 | For M-1 the development team delivers the test page, the steps to start and stop the two test networks, and an observation sheet. The owner only observes and fills in the sheet. |

## Where results go

| Item | Path |
|---|---|
| One report per measurement | `docs/src/measurements/m-<n>-<slug>.md` |
| Index with the state of each measurement | `docs/src/measurements/index.md` |
| Raw data (logs, captures, screenshots) | `docs/src/measurements/data/m-<n>/` when small; otherwise described in the report and kept outside the repository |

Add every new page to `docs/src/SUMMARY.md`.

## Asking for review

When a report is ready, tell the owner the path of `docs/src/measurements/index.md` and which
entries changed. The report must name the RFC and this handoff. The architect reviews against
[acceptance-qa-checklist.md](./acceptance-qa-checklist.md).

## Questions

A question about what an RFC means goes to the architect as a review request. Do not answer it
by changing the design.
