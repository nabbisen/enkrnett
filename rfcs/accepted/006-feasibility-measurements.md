# RFC 006 — Feasibility measurements

**Status.** Accepted (2026-09-29)
**Tracks.** The assumptions under the baseline RFCs that no source could settle. Prepares the
Round 2 decisions.
**Touches.** `docs/src/measurements/`. No product code.

Handoff: [`../handoffs/006-feasibility-measurements/`](../handoffs/006-feasibility-measurements/README.md)

Terms are defined in [RFC 001](./001-scope-people-vocabulary.md).

## Summary

Eight measurements turn assumptions into numbers and observations. Their results decide the
remaining design questions and confirm or correct RFC 002 to RFC 005.

## Motivation

The baseline rests on facts that were checked against sources. Some questions cannot be answered
from sources: how a phone behaves, how much memory a device has left, whether a person without
training can install a device. Deciding them by assumption would repeat the mistake of the
earlier study, several of whose technical premises proved wrong when checked.

## Design

### 1. Rules

| # | Rule |
|---|---|
| 1 | A measurement produces numbers and observations. It produces no product code. Code written for a measurement is not kept in the repository. |
| 2 | A result that contradicts an RFC is reported as plainly as one that confirms it. |
| 3 | Every report states the exact versions of everything involved and how to repeat the measurement. |
| 4 | Where a threshold is given, the report states the measured value beside it. It does not judge. The architect and the owner judge. |
| 5 | A measurement that cannot be carried out is reported with the reason. |

### 2. Measurements

| ID | Question | Informs | Needs |
|---|---|---|---|
| **M-1** | What do phone browsers do with a UI served over HTTP from a private address? | RFC 003 (name, wording of guides); RFC 004 (saving the password) | Phones; any device that serves a page |
| **M-2** | How large is a daemon of Enkrnett's kind on `mipsel_24kc`, and how much memory does it use? | Footprint budgets; RFC 004 open question 1 | Build machine; virtual machine or device |
| **M-3** | Does a complete image fit and run on a `constrained` device? | Promotion of 8 MB / 64 MB devices; footprint budgets | A `constrained` device |
| **M-4** | How do OWE and Transition Mode behave on the candidate devices with real clients? | OWE policy; RFC 002; RFC 005 criteria 4 and 5 | Candidate devices; client devices |
| **M-5** | Can configuration be applied with confirm-or-revert through OpenWrt's own interfaces, by a process that is not `root`? | Configuration transactions | Virtual machine |
| **M-6** | Does OpenWrt in a virtual machine with simulated radios run OWE from association to traffic? | Test strategy | Virtual machine |
| **M-7** | Can non-experts install a candidate device, and recover it, with a written guide alone? | RFC 005 criteria 2 and 3 | Candidate devices; participants |
| **M-8** | Does the separation of RFC 002 hold against a client that tries to break it? | RFC 002 | A candidate device; two client machines |

### 3. Provisional thresholds

These are the budgets proposed for decision D-10. They are shown so that results can be compared
with them. They are not yet requirements.

| Measurement | Quantity | Provisional budget |
|---|---|---:|
| M-2 | Daemon inside squashfs | ≤ 350 KB |
| M-2 | Daemon resident memory, idle | ≤ 3 MB |
| M-2 | Time to check the owner password on the slowest device | ≤ 500 ms |
| M-3 | Complete image | Not larger than the stock release image for the same device |
| M-3 | Free overlay after installation | ≥ 1,200 KB |
| M-3 | Free memory with 20 associated clients | ≥ 12 MB |

### 4. Who measures

| Measurement | Carried out by | Reason |
|---|---|---|
| M-2, M-5, M-6 | Development team | Need no hardware |
| M-1, M-3, M-4, M-8 | Whoever holds the devices and phones | Need hardware |
| M-7 | The owner, with participants who are not IT specialists | Needs people |

### 5. Order

```text
M-6 ─► M-5 ─► M-2            no hardware needed; start at once
M-1                           as soon as two phones are at hand
M-4 ─► M-8 ─► M-3 ─► M-7      as devices arrive
```

### 6. Reports

| Item | Rule |
|---|---|
| Place | `docs/src/measurements/`, one file per measurement |
| Index | `docs/src/measurements/index.md` lists every measurement with its state |
| Form | The template in the handoff |
| Review | The architect reviews each report and records what it changes |

### 7. Completion

This RFC is implemented when all eight reports are delivered and reviewed, or when the owner
decides that a measurement is not needed.

## Consequences

| Affected | Change |
|---|---|
| Round 2 decisions | Taken after the measurements they depend on |
| RFC 002 to RFC 005 | May be amended by a result. An amendment is made in the RFC, before acceptance or by a following RFC. |
| Draft requirement "at least one real OWE-capable AP/client combination" | Replaced by the client matrix of M-4. |

## Alternatives considered

| Alternative | Why not |
|---|---|
| Decide by estimate and correct during implementation | Footprint and phone behavior can invalidate the architecture. Finding out late is the most expensive way. |
| Build a prototype of the product | A prototype invites keeping its code. Measurements are cheaper and their code is discarded. |

## Open questions

| # | Question | For |
|---|---|---|
| 1 | Which devices and phones are available. | Owner |
| 2 | Who takes part in M-7. | Owner |
