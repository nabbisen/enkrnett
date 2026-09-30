# RFC 001 — Scope, people, and vocabulary

**Status.** Accepted (2026-09-29)
**Tracks.** What Enkrnett is, whom it serves, the words it uses, and how its specifications are
written. Startup decisions D-01 and D-02.
**Touches.** Every document; the structure of `docs/src/`; requirement identifiers.

## Summary

Enkrnett lets a person without IT training give visitors a Wi-Fi network that is as easy to join
as open Wi-Fi and encrypts each visitor's radio link individually.

This RFC fixes the scope, the people, one vocabulary, and the rules for writing specifications.
Every later RFC uses these words and rules.

## Motivation

The drafts that preceded this RFC described one system with three sets of state names, nine
meanings of the word "profile", and three separate lists of open decisions. Many requirements
used words such as "small" or "reasonable" and could not be tested. A project whose main promise
is that operators are not confused cannot start from confused documents.

## 1. Purpose and scope

| Enkrnett is | Enkrnett is not |
|---|---|
| A deployment and management layer on OpenWrt | A router operating system |
| A small daemon, a small UI, firmware definitions, and guides | A Wi-Fi stack, a kernel, or a driver |
| Local: it works without any service outside the venue | A cloud-managed product |
| Specialised: guest Wi-Fi with OWE | A general router administration suite |

### What Enkrnett adds

OpenWrt already runs OWE. Its default Wi-Fi package has supported OWE since release 21.02
([source][hostapd-makefile]). Enkrnett therefore does not exist to make OWE possible. It exists to
provide what stock OpenWrt does not give a non-expert:

| Enkrnett adds | Stock OpenWrt |
|---|---|
| Installation and setup that a non-expert can complete | Assumes networking knowledge |
| Safe defaults for a public venue | Guests share the local network; `root` has no password |
| Status that says what is protected and what is not | Shows configuration |
| A narrow management surface | A general-purpose administration interface |
| Guidance when something fails | Logs |

### Requirements

| ID | Requirement | Verification |
|---|---|---|
| ENK-SCOPE-001 | Enkrnett shall use OpenWrt for all wireless execution, and shall reach hostapd/wpad only through OpenWrt. | Design review |
| ENK-SCOPE-002 | Setup, configuration, and status shall work without any service outside the venue. | Test without Internet |
| ENK-SCOPE-003 | An operator shall be able to install, set up, and run Enkrnett without knowledge of networking, OpenWrt, or Wi-Fi security. | Usability test with non-experts |
| ENK-SCOPE-004 | Whatever Enkrnett tells an operator or a guest about protection shall be accurate and bounded. It shall state what is protected and what is not. | Review of every status and message against the security RFCs |
| ENK-SCOPE-005 | No supported function shall require the operator to use a command line. | Test |

## 2. Design principles

The owner's philosophy governs every decision:

> "Finally clean, safe and secure, robust and sophisticated design."
>
> APIs and UI/UX must not leave users confused or misunderstanding.

| # | Principle | Meaning in practice |
|---|---|---|
| 1 | **One meaning** | One word for one thing. One way to do one thing. A screen or an API response can be understood in one way only. |
| 2 | **Less is more** | The operator sees only what a decision needs. Anything else must justify its presence. |
| 3 | **Safety over functionality** | When they conflict, the safe option wins and the function waits. |
| 4 | **Bounded claims** | Enkrnett never says "secure". It says what is encrypted, what is separated, and what is not. |
| 5 | **Smallest trusted surface** | Every service, package, and stored item must earn its place. |
| 6 | **Use the platform** | Enkrnett configures OpenWrt through OpenWrt's own interfaces and never works around them. |
| 7 | **Hardware is described, never assumed** | A capability is detected at run time or declared in a tested device profile. Unsupported stays explicit. |
| 8 | **A physical way out** | No configuration, lost credential, or failed update leaves a device that its holder cannot recover. |
| 9 | **Evidence before assertion** | A technical claim is a fact with a source, a decision, or an assumption with a verification task. |

## 3. People

| Person | Description |
|---|---|
| **Operator** | Owns or works at a shop, café, restaurant, hotel, or community space. Not an IT specialist. **Installs, sets up, and runs** Enkrnett. |
| **Guest** | A visitor who uses the guest network. Never sees the management UI. Is entitled to accurate information about the network. |
| **Expert** | A person with OpenWrt knowledge who may use facilities outside the management UI. Enkrnett does not depend on experts. |
| **Contributor** | A developer, tester, translator, or writer who works on Enkrnett. |

There is no separate "installer". The operator installs (decision D-02).

## 4. Vocabulary

A term has one meaning. The UI, the API, the documentation, and the code use the same term. Each
term has one fixed translation per language, kept in `docs/src/`.

### 4.1 Networks

| Term | Meaning |
|---|---|
| **Uplink** | The device's wired connection to the venue's existing router. |
| **Site network** | The operator's existing network, reached through the uplink. |
| **Guest network** | The Wi-Fi network for guests. |
| **Management network** | The Wi-Fi network through which the operator manages the device. |
| **Setup network** | The temporary Wi-Fi network of an unclaimed device. |

### 4.2 Actions

| Term | Meaning |
|---|---|
| **Install** | Put Enkrnett on a device for the first time. |
| **Set up** | Claim a device and give it its first configuration. |
| **Claim** | Become the owner of an unclaimed device. |
| **Update** | Install a newer Enkrnett release on the same OpenWrt baseline. |
| **Upgrade** | Move a device to a newer OpenWrt baseline. |
| **Reset** | Erase everything the operator configured. The device becomes unclaimed. |

### 4.3 Things

| Term | Meaning |
|---|---|
| **Device** | A router that runs Enkrnett. |
| **Owner** | The person who claimed the device. |
| **Owner password** | What the owner enters to change anything. |
| **Management key** | The Wi-Fi key of the management network. |
| **Management UI** | The pages the operator uses. |
| **Management API** | The interface the management UI uses. |
| **OpenWrt baseline** | The OpenWrt release series that an Enkrnett release is built on. |
| **Topology** | How the device is connected between the site network and its Wi-Fi networks. |
| **Device profile** | The description of one device model and hardware revision: how to build for it, what it can do, how to install and recover it. |
| **Device class** | `constrained` (about 8 MB flash, 64 MB RAM) or `standard` (16 MB flash and 128 MB RAM, or more). |
| **Device tier** | The level of support a device profile receives. Defined in RFC 005. |
| **Wi-Fi configuration** | What the operator asked the guest network to be. |

### 4.4 Words that are replaced

| Do not use | Use | Reason |
|---|---|---|
| admin SSID, admin network | management network | One stem: management network, management UI, management API |
| pairing, pairing credential | claim, owner password | There is one owner, not pairs of devices |
| PIN | — | No printed PIN exists on the target hardware |
| deployment profile | topology | "Profile" had nine meanings |
| AP profile | Wi-Fi configuration | Same |
| resource profile | device class | Same |
| secure, secured | the specific property | Principle 4 |

## 5. Specification rules

### 5.1 Where documents live

| Place | Holds | Normative |
|---|---|---|
| `rfcs/` | Decisions and their design. Lifecycle per [RFC 000](../done/000-rfc-lifecycle-policy.md), 5-folder variant. | Yes, once accepted |
| `rfcs/handoffs/NNN-slug/` | How to carry out an RFC: implementation, test, and other tasks such as release preparation. | No. A handoff never overrides its RFC. |
| `docs/src/` | The current documentation, including the consolidated specification. mdBook. | The specification chapters, yes. They restate accepted RFCs. |
| `ROADMAP.md` | Milestones and order of work. | No |
| `CHANGELOG.md` | History of releases. | No |

**Precedence on conflict:** the owner's project rules, then accepted RFCs (the newer wins), then
`docs/src/`.

Working drafts written before this RFC are inputs. They are not normative and are not part of
the repository.

### 5.2 Keywords

| Keyword | Meaning |
|---|---|
| **shall** | Required. A release that does not meet it is not complete. |
| **should** | Expected. A deviation is allowed only with a written reason. |
| **may** | Permitted. |

"Must", "will", and capitalised variants are not used in requirements.

### 5.3 Attributes of a requirement

| Attribute | Content |
|---|---|
| ID | `ENK-<AREA>-<NNN>` |
| Statement | One sentence with one keyword |
| Rationale | Why it exists |
| Source | An RFC, an owner decision, or a cited fact |
| Release | The first release that shall meet it |
| Verification | Test, measurement, inspection, or usability test |

A statement is accepted only if two readers would verify it the same way. Words such as "small",
"reasonable", "sufficient", "minimal", and "clear" are replaced by a number or an observable
behavior.

### 5.4 Identifiers

| Rule | |
|---|---|
| Form | `ENK-<AREA>-<NNN>`, three digits |
| Stability | An identifier is never reused and never renumbered |
| Withdrawal | A withdrawn requirement keeps its identifier and is marked withdrawn |
| Continuity | Numbering continues from the draft requirement set. Where an area starts above 001 in an RFC, the lower numbers belong to draft requirements that are carried over or withdrawn when the specification is consolidated. |

Areas used by the baseline RFCs:

| Area | Subject | RFC |
|---|---|---|
| `SCOPE` | Purpose and product promises | 001 |
| `TOPO` | Topology | 002 |
| `ISO` | Isolation | 002 |
| `NET` | Network exposure | 002 |
| `HARD` | Hardening | 002 |
| `MGMT` | Management channel and UI delivery | 003 |
| `SEC` | Request security | 003 |
| `OWN` | Ownership and credentials | 004 |
| `AUTH` | Sessions and authorization | 004 |
| `TIER` | Device tiers and profiles | 005 |
| `BASE` | OpenWrt baseline, update, upgrade | 005 |
| `INST` | Installation | 005 |
| `RF` | Radio regulation | 005 |

### 5.5 Traceability

| Rule | |
|---|---|
| Design to requirement | Every design section names the requirements it satisfies. |
| Requirement to test | Every requirement names how it is verified. Tests are derived from the specification, not from the code. |
| Decision | Every decision is recorded in an RFC. There is no other list of decisions. |

## Consequences

| Affected | Change |
|---|---|
| Draft requirement set | Is consolidated into `docs/src/` after the baseline RFCs are accepted. Until then the RFCs carry their own requirements. |
| Draft persona "secondary operator" | Becomes **Expert**. |
| Draft use case "first-time setup" | Splits into **install** and **set up**. |
| All later RFCs | Use section 4 and section 5. |

## Alternatives considered

| Alternative | Why not |
|---|---|
| Keep a separate installer persona | The owner requires that a non-expert can install (D-02). |
| Keep the drafts as the living specification | They are outside version control and contradict each other. |
| A glossary only in `docs/src/` without an RFC | The vocabulary is a decision. It needs a record and an acceptance. |

## Open questions

| # | Question |
|---|---|
| 1 | Japanese translations of the terms in section 4 are fixed when the UI RFC is written. |

## References

- OpenWrt, hostapd package build options, branch `openwrt-25.12`: [Makefile][hostapd-makefile]

[hostapd-makefile]: https://github.com/openwrt/openwrt/blob/openwrt-25.12/package/network/services/hostapd/Makefile
