# RFC 016 — Amendments from the review of RFC 001 to 015

**Status.** Implemented (main, 2026-10-31): the Amended-by lines and pointer notes are in place and the consolidated specification (`docs/src/specification/`) restates every amendment.
**Tracks.** The architect's review of every accepted RFC against the owner's philosophy
(2026-10-31, on the owner's instruction), findings F-1 to F-16.
**Touches.** RFC 001, 002, 004, 005, 007, 010, 011, 012, 013, 015. Identifiers keep their
numbers; new requirements continue each area's numbering.

Terms are defined in [RFC 001](../accepted/001-scope-people-vocabulary.md) as amended by
[RFC 009](../accepted/009-vocabulary-amendment-operator-words.md) and by this RFC.

## Summary

Sixteen findings, read across the fifteen RFCs as one specification. Two close security gaps
(an old signed release index could be replayed as an update; the operator is never told whether
their own management session is the readable kind). Two make requirements implementable as
written (the setup network's platform sections were owned by nobody; the setup name's suffix
depends on which address the label prints). The rest give one word to one thing and one number
to one act. Each amendment names the RFC and the text it changes; on acceptance, each amended RFC
receives an **Amended by** line and the pointer notes listed in §11.

## 1. RFC 001 — Vocabulary, areas, amendment rule

### 1.1 Vocabulary (§4.3) — finding F-1

| Term | Meaning | Replaces |
|---|---|---|
| **Settings** | What the operator asked the device to be: the country, the guest network (name, mode, on or off), the management network's name. Stored as the *desired document* of RFC 011; carried by the `settings` shape of RFC 015. | Wi-Fi configuration |
| **Change** | One saved alteration of the settings, from save to settled or undone (RFC 011 §4). | (new; replaces "transaction" in operator-facing text, already forbidden by RFC 009) |
| **Status** | What the device is doing now: the effective state of RFC 011 §1, shown on the first screen. | (new) |
| **Details** | The screen for facts that no decision needs: model, versions, identifier, hardening report. | (new) |

Wherever an accepted RFC says "Wi-Fi configuration" (RFC 004 ENK-OWN-018, RFC 005 ENK-BASE-004,
RFC 001 §4.3), read "settings". Japanese terms: Operator UI RFC.

### 1.2 Areas (§5.4) — finding F-2

The area table gains:

| Area | Subject | RFC |
|---|---|---|
| `WORD` | Operator-facing words | 009 |
| `FOOT` | Footprint budgets | 010 |
| `CFG` | Configuration and transactions | 011, 014 |
| `OWE`, `GUEST` | Guest modes, counts, notice | 012 |
| `RLS`, `UPD`, `BUILD`, `REPO` | Release, update, build, repository | 013 |
| `API` | Management API | 015 |

**Reserved: areas of the draft requirement set, never to be reused** (RFC 001 §5.4 continuity):
`RTR`, `NET` (001–007 and 101–104), `UI`, `SEC` (001–008), `FUT`, `DEV`, `BUILD` (001–006), `TEST`,
`SETUP`, `PWA`, `OWE` (001–006), `HW`, `DIAG`, `OBS`, `LIFE`, `REL`, `PERF`, `DIST`, `MAINT`.
A new area is checked against this list and against `.git-exclude/specs/enkrnett-requirements.md`.

### 1.3 Amendment rule (§5.1) — finding F-3

| ID | Requirement | Verification |
|---|---|---|
| ENK-SCOPE-006 | When a later RFC amends an accepted RFC, the amended RFC shall receive, in the commit that accepts the amending RFC, an **Amended by** line in its header naming the amending RFC, and a one-line pointer at each amended passage. The amended text itself is rewritten only in the consolidated specification. | Inspection at acceptance |

## 2. RFC 002 — Renumbering; hardening report

### 2.1 Finding F-4

| ID | Requirement | Verification |
|---|---|---|
| ENK-TOPO-006 | When the uplink's subnet collides with one of the device's Wi-Fi subnets, the device shall renumber only the colliding network. The management subnet shall be chosen from a range that the common site routers of the target markets do not use by default, so that it is the colliding one only by exception. When the management address does change, the status screen shall show the new address on the next visit, and the guide shall name the one fallback: Erase and set up again. | Test with an uplink that collides with the guest subnet (management address unchanged) and with the management subnet |

### 2.2 Finding F-5

ENK-HARD-006 reads: "The device shall report, for each of ENK-HARD-001 to 005 and 008, whether it
is met."

## 3. RFC 004 — One number; the label address; the guest mode

### 3.1 Finding F-6

The time the operator has to join the new management network after a change that needs
confirmation is **600 seconds**, the same as ENK-OWN-008. RFC 011 ENK-CFG-008 reads "within
600 s of a passed health check"; RFC 011 §8 and RFC 015 §7 read "within 10 minutes".

### 3.2 Finding F-7 (with RFC 005)

ENK-TIER-002 gains: "… and which hardware address the device's label prints and how the device
obtains that address at run time." ENK-OWN-004 is verified per device profile against the label
of a physical unit (M-7).

### 3.3 Finding F-8

RFC 004 §3 step 5 reads: "Chooses the country, the guest network's name and mode (RFC 012
ENK-OWE-008), and the owner password."

## 4. RFC 005 — see §3.2 and §11

## 5. RFC 007 — Controls updated; two threats added; one risk added

### 5.1 Finding F-9: controls that exist now

| Threat | Control | Verification | Remains |
|---|---|---|---|
| 23 | ENK-SEC-017 to 022 | Test | None known beyond R-10 |
| 37 | Memory-safe language; ENK-SEC-014; ENK-CFG-019 to 022 | Review; inspection | R-12 on targets with reduced hardening |
| 38 | ENK-RLS-004, 005, 007; ENK-UPD-003, 009 | Test | R-14 |
| 39 | ENK-SCOPE-004; RFC 012 ENK-OWE-013 to 018; ENK-GUEST-002 | Review of every status and message | None known |
| 41 | D-15: ENK-OWE-014; ENK-API-028 to 030 | Inspection | None known |

### 5.2 Finding F-10: threats added (numbers continue)

| # | Threat | Attacker | Asset | Control | Verification | Remains |
|---|---|---|---|---|---|---|
| 43 | Replays an old, validly signed release index so that the device offers an older release as an update | AC-5, AC-7 | AS-10 | **ENK-UPD-009** | Test with a replayed index | None known |
| 44 | Delivers, or the project publishes, an image that passes every check and does not boot on a single-slot device | AC-10; project error | AS-8, AS-10 | ENK-UPD-005 for images that boot; release checklist item 2 (every Tier 1 device updated from the previous release before publication) | Release checklist | R-14 |

| ID | Risk | Who | Effect | How it is noticed or reduced | What would close it |
|---|---|---|---|---|---|
| R-14 | An update image does not boot on a single-slot device. | AC-10; project error | No service; the device needs the recovery procedure of its device profile (RFC 005 ENK-INST-004). | Every release is installed on every Tier 1 device before publication; the UI says before the update that it cannot be undone from the device. | Devices with two firmware slots |

### 5.3 Open questions 1 to 4 of RFC 007: closed by RFC 015, RFC 011, RFC 013, RFC 012.

## 6. RFC 010 — TLS wording (finding F-11)

ENK-FOOT-013 reads: "The daemon shall carry no TLS implementation, and the management channel
shall use none. The platform's downloader may use the platform's TLS for the release index and
images (RFC 013)."

## 7. RFC 011 — Dead text; setup network; the number

| Finding | Amendment |
|---|---|
| F-12 | ENK-CFG-008: "or the country" removed. The country changes only through Erase (RFC 011 §2). |
| F-13 | §3: `network` row owns `enk_guest`, `enk_mgmt`, `enk_setup`; `dhcp` row owns `enk_guest`, `enk_mgmt`, `enk_setup`. |
| F-6 | ENK-CFG-008: 180 s → 600 s; §8 row "waiting for confirmation": "within 10 minutes". |

## 8. RFC 012 — The notice (finding F-14)

RFC 012 §4 gains, as its last paragraph:

> This access point keeps no record of the devices that connect to it.

ENK-GUEST-002 covers it. RFC 015 ENK-API-030 and RFC 012 ENK-OWE-014 make it true.

## 9. RFC 013 — Downgrade protection; transport (findings F-10, F-11)

| ID | Requirement | Verification |
|---|---|---|
| ENK-UPD-009 | The release index shall carry a sequence number that increases with every publication. The device shall keep the highest sequence it has accepted, shall refuse an index with a lower sequence, and shall offer and install only a release whose version is greater than the installed one. An expert may install any release by other means (ENK-UPD-008). | Test with a replayed index and with an index that offers the installed or an older version |

ENK-UPD-001 gains: "The device fetches the index and the image through the platform's downloader
over HTTPS to the project's host. Authenticity rests on the signatures of ENK-UPD-003, not on the
transport."

ENK-RLS-006 (the index) gains the field *sequence*. The `update` shape of RFC 015 is unchanged:
an older or equal release is simply not `available`, and `last_check.result` gains the value
`stale_index`.

## 10. RFC 015 — `connection.security` (finding F-16)

The `connection` shape reads: `network` (`setup` | `management`), `security` (`none` | `owe` |
`wpa2` | `wpa3` | `null`). `encrypted` is removed before any client exists. ENK-API-011 names
`security`. The UI RFC maps the values to words without naming the standards; on the management
network it says something only for `wpa2` ("someone who knows the management key could read this
session; a newer phone would not have this limit") and `null`.

`last_check.result` gains `stale_index` (§9).

## 11. Pointer notes written at acceptance (ENK-SCOPE-006)

| RFC | Header line | Pointer notes at |
|---|---|---|
| 001 | Amended by RFC 009, 016 | §4.2 Reset; §4.3 Wi-Fi configuration; §5.4 areas |
| 002 | Amended by RFC 007, 016 | ENK-HARD-006; §5 "reset"; open question 1 (closed by RFC 011) |
| 003 | Amended by RFC 007 | Open question 2 (closed by RFC 015) |
| 004 | Amended by RFC 007, 009, 015, 016 | §3 step 5; ENK-OWN-016; §7; §8 "Reset"; open questions 1, 2 (closed by RFC 010) |
| 005 | Amended by RFC 016 | ENK-TIER-002; summary row "Update and upgrade" (RFC 013); open question 3 |
| 007 | Amended by RFC 016 | Threats 23, 37, 38, 39, 41; §5.7 and §7 (43, 44, R-14); open questions |
| 010 | Amended by RFC 016 | ENK-FOOT-013 |
| 011 | Amended by RFC 015, 016 | ENK-CFG-008; §3; §8 |
| 012 | Amended by RFC 016 | §4 |
| 013 | Amended by RFC 016 | ENK-UPD-001; ENK-UPD-003 (009 beside it); ENK-RLS-006 |
| 015 | Amended by RFC 016 | `connection`; §7 "3 minutes"; `last_check.result` |

## Consequences

| Affected | Change |
|---|---|
| Gap analysis traceability | F-1 closes the last use of "Wi-Fi configuration". |
| Operator UI RFC | Inherits §1.1 (four terms), S-1 to S-4 of review 13, and the words for `connection.security`. |
| Consolidated specification | Rewrites every amended passage; until then the pointer notes carry the amendment. |
| M1 handoffs | ENK-UPD-009 and ENK-TOPO-006 enter the daemon's design; ENK-TIER-002's address field enters the device profile schema. |

## Alternatives considered

| Alternative | Why not |
|---|---|
| Edit the accepted RFCs in place | The folder is the state (RFC 000); a silent edit of an accepted text is a change nobody accepted. |
| Keep 180 s for a change and 600 s for the claim | Two numbers for the same act of joining a network and proving it; the shorter one is too short for typing a generated key. |
| Keep `connection.encrypted` as a boolean | Hides the one fact (WPA2 versus WPA3) that decides whether R-5 applies to this operator today. |
| Protect against downgrade by rejecting any version lower than a compile-time floor | Does not stop a replay of the previous release's index; a sequence number does. |

## Open questions

| # | Question | Closed by |
|---|---|---|
| 1 | The default range for the management subnet (ENK-TOPO-006): candidates are inside `10.0.0.0/8` away from `10.0.0.0/24` and `10.1.1.0/24`, or inside `172.16.0.0/12` | M1 internal design, with a survey of the default subnets of routers sold in Japan |
