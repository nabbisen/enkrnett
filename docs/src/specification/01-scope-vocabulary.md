# 1. Scope, people, vocabulary

From RFC 001, as amended by RFC 009, 016, and 017.

## 1.1 Purpose and scope

Enkrnett lets a person without IT training give visitors a Wi-Fi network that is as easy to join
as open Wi-Fi and encrypts each visitor's radio link individually.

| Enkrnett is | Enkrnett is not |
|---|---|
| A deployment and management layer on OpenWrt | A router operating system |
| A small daemon, a small UI, firmware definitions, and guides | A Wi-Fi stack, a kernel, or a driver |
| Local: it works without any service outside the venue | A cloud-managed product |
| Specialised: guest Wi-Fi with OWE | A general router administration suite |

OpenWrt already runs OWE. Enkrnett adds what stock OpenWrt does not give a non-expert:
installation and setup they can complete, safe defaults for a public venue, a status that says
what is protected and what is not, a narrow management surface, and guidance when something
fails.

{{#include requirements/scope.md}}

## 1.2 Design principles

The owner's philosophy governs every decision:

> "Finally clean, safe and secure, robust and sophisticated design."
> APIs and UI/UX must not leave users confused or misunderstanding.

| # | Principle | In practice |
|---|---|---|
| 1 | One meaning | One word for one thing. One way to do one thing. A screen or an API response can be understood in one way only. |
| 2 | Less is more | The operator sees only what a decision needs. |
| 3 | Safety over functionality | When they conflict, the safe option wins and the function waits. |
| 4 | Bounded claims | Enkrnett never says "secure". It says what is encrypted, what is separated, and what is not. |
| 5 | Smallest trusted surface | Every service, package, and stored item must earn its place. |
| 6 | Use the platform | Enkrnett configures OpenWrt through OpenWrt's own interfaces and never works around them. |
| 7 | Hardware is described, never assumed | A capability is detected at run time or declared in a tested device profile. |
| 8 | A physical way out | No configuration, lost credential, or failed update leaves a device that its holder cannot recover. |
| 9 | Evidence before assertion | A technical claim is a fact with a source, a decision, or an assumption with a verification task. |

## 1.3 People

| Person | Description |
|---|---|
| **Operator** | Owns or works at a shop, café, restaurant, hotel, or community space. Not an IT specialist. Installs, sets up, and runs Enkrnett. There is no separate installer. |
| **Guest** | A visitor who uses the guest network. Never sees the management UI. Is entitled to accurate information about the network. |
| **Expert** | A person with OpenWrt knowledge who may use facilities outside the management UI. Enkrnett does not depend on experts. |
| **Contributor** | A developer, tester, translator, or writer who works on Enkrnett. |

## 1.4 Vocabulary

A term has one meaning. The API, the documentation, and the code use the term; the UI and the
guides use the **display form** where one is given, because the operator also reads the phone's
own Wi-Fi screen. Japanese display forms are the fixed translations (RFC 017 §2).

### Networks

| Term | Meaning | On screen (EN) | On screen (JA) |
|---|---|---|---|
| **Uplink** | The device's wired connection to the venue's existing router. | Internet connection | インターネット接続 |
| **Site network** | The operator's existing network, reached through the uplink. | the shop's or venue's own network | お店や施設のネットワーク |
| **Guest network** | The Wi-Fi network for guests. | guest Wi-Fi | ゲストWi-Fi |
| **Management network** | The Wi-Fi network through which the operator manages the device. | management Wi-Fi | 設定用Wi-Fi |
| **Setup network** | The temporary Wi-Fi network of an unclaimed device. | setup Wi-Fi | セットアップ用Wi-Fi |

### Actions

| Term | Meaning | On screen (EN) | On screen (JA) |
|---|---|---|---|
| **Install** | Put Enkrnett on a device for the first time. | install | インストール |
| **Set up** | Claim a device and give it its first settings. | set up | セットアップ |
| **Claim** | Become the owner of an unclaimed device. | (never shown; "set up") | — |
| **Update** | Install a newer Enkrnett release on the same OpenWrt baseline. | Update | アップデート |
| **Upgrade** | Move a device to a newer OpenWrt baseline. | Update (the operator sees one word) | アップデート |
| **Erase** | Remove everything the operator configured. The device becomes unclaimed. | Erase | 初期化 |
| **Restart** | Power the device off and on again, or let it reboot. Nothing changes. | Restart | 再起動 |

### Things

| Term | Meaning | On screen (EN) | On screen (JA) |
|---|---|---|---|
| **Device** | A router that runs Enkrnett. | this device | 本体 |
| **Owner** | The person who claimed the device. | owner | 管理者 |
| **Owner password** | What the owner enters to change anything. | owner password | 管理者パスワード |
| **Management key** | The Wi-Fi key of the management network. | management Wi-Fi password | 設定用Wi-Fiのパスワード |
| **Management UI** | The pages the operator uses. | settings screen | 設定画面 |
| **Management API** | The interface the management UI uses. | — | — |
| **Settings** | What the operator asked the device to be: the country, the guest network (name, mode, on or off), the management network's name. Stored as the desired document (chapter 6); carried by the `settings` shape (chapter 9). | Settings | 設定 |
| **Change** | One saved alteration of the settings, from save to settled or undone. | change | 変更 |
| **Status** | What the device is doing now: the effective state (chapter 6), shown on the first screen. | Status | 状態 |
| **Details** | The screen for facts that no decision needs. | Details | 詳細 |
| **OpenWrt baseline** | The OpenWrt release series that an Enkrnett release is built on. | — | — |
| **Topology** | How the device is connected between the site network and its Wi-Fi networks. | — | — |
| **Device profile** | The description of one device model and hardware revision. | — | — |
| **Device class** | `constrained` (about 8 MB flash, 64 MB RAM) or `standard` (16 MB flash and 128 MB RAM, or more). | — | — |
| **Device tier** | The level of support a device profile receives (chapter 5). | — | — |

### Words that are replaced

| Do not use | Use |
|---|---|
| admin SSID, admin network | management network |
| pairing, pairing credential, pair | claim, owner password; set up |
| PIN | — |
| deployment profile, AP profile, resource profile | topology, settings, device class |
| Wi-Fi configuration, desired configuration (as a term) | settings |
| Reset (as an action) | Erase |
| secure, secured | the specific property |

### Words that never appear in operator-facing text

Operator-facing text means every screen of the management UI, every guide, every status and
error message, and the guest notice. A "Details" view for experts is exempt when clearly marked.

| Never | Use instead |
|---|---|
| secure, insecure, not secure, security (as a label) | the specific property: *encrypted*, *separated*, *protected on the radio link* |
| Transition Mode, transition | *Open to every device* |
| OWE, Enhanced Open, WPA2, WPA3, PMF, SAE | *encrypted* / *not encrypted*; the standard's name may appear once in the guest notice, in brackets |
| SSID, BSS, BSSID | *name*, *Wi-Fi* |
| reset (as an action), press reset | *erase*; for the button: "hold the button for five seconds until the light changes" |
| apply, commit, rollback, transaction | *save*, *saved*, *undone*, *change* |
| pairing, pair | *set up*, *claim* |
| management key, uplink, claim, network (in the three network names) | the display forms above |

Japanese words on the same list: セキュア, セキュリティ, 安全 (as a one-word label), トランジション,
移行モード, SSID, WPA2, WPA3, PMF, SAE, リセット, 適用, コミット, ロールバック, ペアリング, ペア,
管理用, 初期設定, 管理キー, ネットワーク in the three network names. The UI build fails on any of
them (chapter 10 §4).

{{#include requirements/word.md}}

## 1.5 Specification rules

### Where documents live

| Place | Holds | Normative |
|---|---|---|
| `rfcs/` | Decisions and their design. Lifecycle per RFC 000, 5-folder variant. | Yes, once accepted |
| `rfcs/handoffs/NNN-slug/` | How to carry out an RFC. | No. A handoff never overrides its RFC. |
| `docs/src/specification/` | This book: the accepted RFCs restated with amendments applied. | Yes |
| `ROADMAP.md`, `CHANGELOG.md` | Order of work; history of releases. | No |

**Precedence on conflict:** the owner's project rules, then accepted RFCs (the newer wins), then
this book.

**Amending an accepted RFC:** only by a later RFC. In the commit that accepts it, the amended RFC
receives an **Amended by** header line and a pointer at each amended passage, and this book is
rewritten (ENK-SCOPE-006).

### Keywords and attributes

**shall** required · **should** expected, deviation with a written reason · **may** permitted.
"Must", "will", and capitalised variants are not used. A statement is accepted only if two
readers would verify it the same way; "small", "reasonable", "sufficient", "minimal", "clear" are
replaced by a number or an observable behaviour.

### Identifiers and areas

`ENK-<AREA>-<NNN>`, three digits, never reused, never renumbered. A withdrawn requirement keeps
its identifier. Numbering continues from the draft requirement set that preceded RFC 001.

| Area | Subject | Chapter | Area | Subject | Chapter |
|---|---|---|---|---|---|
| `SCOPE` | Purpose and promises | 1 | `CFG` | Settings and changes | 6 |
| `WORD` | Operator-facing words | 1 | `OWE`, `GUEST` | Guest modes, counts, notice | 7 |
| `TOPO`, `ISO`, `NET`, `HARD` | Topology, isolation, exposure, hardening | 2 | `FOOT` | Footprint budgets | 8 |
| `MGMT`, `SEC` | Management channel; request security and limits | 3, 9 | `API` | Management API | 9 |
| `OWN`, `AUTH` | Ownership; sessions | 4 | `UI` | Operator UI | 10 |
| `TIER`, `INST`, `BASE`, `RF` | Devices, installation, baseline, radio regulation | 5 | `RLS`, `UPD`, `BUILD`, `REPO` | Release, update, build, repository | 11 |

**Reserved: areas and numbers of the draft requirement set, never reused.** `RTR`, `NET`
001–007 and 101–104, `UI` 001–007, `SEC` 001–008, `FUT`, `DEV`, `BUILD` 001–006, `TEST`, `SETUP`,
`PWA`, `OWE` 001–006, `HW`, `DIAG`, `OBS`, `LIFE`, `REL`, `PERF`, `DIST`, `MAINT`. A new area is
checked against this list.

### Traceability

Every design section names the requirements it satisfies; every requirement names how it is
verified; every decision is recorded in an RFC, and there is no other list of decisions.
