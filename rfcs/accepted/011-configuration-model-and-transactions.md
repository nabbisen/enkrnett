# RFC 011 — Configuration model and transactions

**Status.** Accepted (2026-10-29)
**Tracks.** Decision D-12 (revised by the self-review, amendment A-3), amendment A-6, measurement
M-5, RFC 005 open question 3 (the gap analysis proposed it as "ENK-REL-006"; that identifier is not used).
**Touches.** The daemon (configuration owner, transaction engine, start-up); firmware profiles
(ubusd ACL, file ownership); the management API (apply, confirm); the management UI.
**Amended by.** RFC 015 (what counts as confirmation); RFC 016 (ENK-CFG-008: no country, 600 s; §3 `enk_setup`; §8); RFC 017 (§8 `not_confirmed` sentence; Japanese wording).

Terms are defined in [RFC 001](./001-scope-people-vocabulary.md) as amended by
[RFC 009](./009-vocabulary-amendment-operator-words.md). Networks:
[RFC 002](./002-network-topology-and-separation.md). Ownership:
[RFC 004](./004-ownership-credentials-authorization.md).

## Summary

Enkrnett keeps one **desired configuration** document, translates it into the OpenWrt
configuration sections it **owns**, applies it through OpenWrt's own reload, and verifies the
**effective state** through ubus. Every change is a transaction with a **pending marker on
flash**: the device checks its own health after applying and undoes the change by itself if the
check fails or if it restarts before the change is settled; the operator confirms only a change
after which they had to reconnect. `rpcd` is not used. The daemon is not `root`.

## Motivation

| Fact | Source |
|---|---|
| `rpcd` offers apply with timed rollback, but the rollback lives in `/var/run` and in `rpcd`'s memory: a restart inside the window keeps the unconfirmed change. Using it also needs a session with a password stored on the device and two permission systems. | M-5 |
| `ubusd` confines a non-root user to named objects and methods; the ACL is read at `ubusd` start. | M-5 |
| `uci` as a non-root user fails with "I/O error" because it also needs a writable delta directory, not only a writable file. | M-5 step 8 |
| `ubus call network reload` and `wifi reload` work for the confined user; a rename costs about 7 s of radio outage, a mode change 3–20 s. | M-5 |
| The automatic Transition Mode of 25.12.5 needs an upstream fix; every radio needs a country. | M-6 |
| The network name is limited to 32 bytes, 29 in Transition Mode. | M-6; gap analysis F-OWE-12 |
| A configuration that reports success before the radio is actually in the desired state is the failure the drafts most wanted to avoid. | Draft architecture §9.4 |

## Design

### 1. Three representations

| Representation | Holds | Owner | Where |
|---|---|---|---|
| **Desired** | What the operator asked for, as one document with a version number | Operator, through the API | A file in the daemon's state directory |
| **Platform** | The UCI sections that implement the desired document | The daemon | `/etc/config/*`, only the sections the daemon owns |
| **Effective** | What the device is actually doing | The platform, read through ubus | Never stored; read on demand |

The API exposes desired and effective, never platform. The UI shows effective with the desired
beside it where they differ.

### 2. The desired document

| Field | Type | Rule |
|---|---|---|
| `version` | integer | Increases by one per saved change |
| `country` | two letters | From the device profile when the profile fixes it (RFC 005 ENK-RF-002); otherwise chosen once at setup; changed only through Erase |
| `guest.name` | text | 1–32 bytes of UTF-8 when `guest.mode` is `encrypted_only`, 1–29 bytes when `open_to_all` (see §6) |
| `guest.mode` | `encrypted_only` \| `open_to_all` | RFC 012 |
| `guest.enabled` | boolean | Lets the operator switch the guest network off without losing its settings |
| `management.name` | text | Set by the device at setup; changeable |
| `management.key` | text | Generated (RFC 004 ENK-OWN-014); never returned in this document; replaced through its own operation |

Nothing else is configurable in the first release. Radio channel, width, power, bands, and every
hostapd detail are chosen by the daemon from the device profile and the country.

### 3. Owned platform sections

| Configuration | Sections the daemon owns | Named |
|---|---|---|
| `wireless` | every `wifi-device`; the `wifi-iface` sections for the guest, management, and setup networks | `enk_radio*`, `enk_guest`, `enk_mgmt`, `enk_setup` |
| `network` | the interfaces and bridges of the guest and management networks | `enk_guest`, `enk_mgmt` *(RFC 016 §7: and `enk_setup`)* |
| `firewall` | the zones, forwardings, and rules of RFC 002 | `enk_*` |
| `dhcp` | the DHCP and DNS settings of the two networks | `enk_guest`, `enk_mgmt` *(RFC 016 §7: and `enk_setup`)* |

| ID | Requirement | Verification |
|---|---|---|
| ENK-CFG-001 | The daemon shall write only the sections it owns, and shall write them completely from the desired document on every apply; it shall never merge with their previous content. | Inspection; test |
| ENK-CFG-002 | The daemon shall never write a hostapd configuration file or open a hostapd control socket. | Inspection (RFC 002 layering rules) |
| ENK-CFG-003 | Every `wifi-device` section shall carry the country, including radios the daemon does not use. | Test (M-6 observation 1) |
| ENK-CFG-004 | Until the OpenWrt baseline includes the fix for the Transition Mode BSSID (commit `1789118599`), the release image shall carry it, and the release checklist shall record whether it is still needed. | Release checklist (M-6) |

### 4. The transaction

```text
 desired(vN) ─save─► validate ─► snapshot owned sections
                               ─► write pending marker {from: vN-1, to: vN, needs_confirm}
                               ─► write owned sections
                               ─► reload (network, wifi) through ubus
                               ─► health check (effective state == desired, within 60 s)
                                     │ ok, no confirmation needed ──► settle: remove marker, drop snapshot
                                     │ ok, confirmation needed ───► wait for confirm (≤ 180 s) ──► settle
                                     │ failed, or no confirm in time, or restart with marker ──► undo:
                                            restore snapshot ─► reload ─► health check ─► remove marker ─► report
```

| ID | Requirement | Verification |
|---|---|---|
| ENK-CFG-005 | A change shall be validated completely (syntax, limits of §6, consistency with the device profile and the country) before anything persistent is written. | Test with every invalid input |
| ENK-CFG-006 | Before writing the owned sections the daemon shall persist a snapshot of their previous content and a pending marker, both on flash, each written to a temporary file and renamed into place. | Inspection; power-loss test |
| ENK-CFG-007 | After reloading, the daemon shall verify the effective state through ubus (`network.wireless status`, `hostapd.* get_status`, `network.interface.* status`) against the desired document within 60 s. A difference is a failed health check. | Test with a change the platform cannot apply |
| ENK-CFG-008 | A change that alters the management network's name or key, or the country, shall additionally require the owner's confirmation through the management network within 180 s of a passed health check. No other change requires confirmation. *What counts as confirmation is defined by RFC 015 ENK-API-021 and 022: the device disconnects the management stations, and the first authenticated request after that confirms. There is no button. RFC 016 §7: "or the country" withdrawn; 180 s reads 600 s.* | Test |
| ENK-CFG-009 | On a failed health check, on a missing confirmation, or at start-up with a pending marker present, the daemon shall restore the snapshot, reload, verify, remove the marker, and report the undone change with its reason. | Test, including power loss at every step |
| ENK-CFG-010 | The daemon shall report success only after the health check has passed and, where required, the confirmation has arrived. Until then the API reports the change as *pending*. | Test |
| ENK-CFG-011 | One transaction at a time. A second change while one is pending shall be refused with the state of the pending one. | Test |
| ENK-CFG-012 | Durations in this section shall be measured with a monotonic clock (RFC 004 ENK-AUTH-004). | Test with a wrong system date |

Why confirmation is limited to three fields: the device can verify everything else by itself
(ENK-CFG-007). Asking the operator to confirm a guest-network change they can see working on
their own phone is friction without safety. The three fields are the ones after which a healthy
device can still be unreachable for its owner.

### 5. Effective state and drift

| ID | Requirement | Verification |
|---|---|---|
| ENK-CFG-013 | The API shall expose the effective state separately from the desired document, with, for each network: up or down, the effective name and mode, and the counts of RFC 012. | Test |
| ENK-CFG-014 | At start-up and at least every 10 minutes the daemon shall compare the owned sections with what it last wrote. A difference is *drift*: the daemon shall report it, shall not overwrite it by itself, and shall offer the owner one action, "restore my settings", which is an ordinary transaction. | Test by editing a section by hand |
| ENK-CFG-015 | The daemon shall never read or report sections it does not own, except to detect that a foreign section conflicts with its own (same interface, same port). | Inspection |

### 6. The network name

| ID | Requirement | Verification |
|---|---|---|
| ENK-CFG-016 | The guest network name shall be 1–32 bytes of UTF-8, 1–29 bytes when `guest.mode` is `open_to_all`; no leading or trailing space; no control characters. The management network name shall be 1–32 bytes. | Test with names of 29, 30, 32, 33 bytes and with Japanese names of 9 and 10 characters |
| ENK-CFG-017 | Wherever a name is entered, the UI shall show the remaining room in the characters the operator sees, shall count a multi-byte character as the bytes it costs, and shall state the limit and its reason before the first character is typed. The rule and the text shall be identical in setup and in later changes. | Inspection; M-7 |
| ENK-CFG-018 | Switching `guest.mode` to `open_to_all` with a name of 30–32 bytes shall be refused with an explanation, never truncated. | Test |

### 7. Privilege

| ID | Requirement | Verification |
|---|---|---|
| ENK-CFG-019 | The daemon shall run as its own system user, not as `root`. | Inspection |
| ENK-CFG-020 | Its `ubusd` ACL shall list exactly the objects and methods it uses and shall be part of the release image (and of the package, with the note that `ubusd` reads it at its next start). | Inspection (M-5 observation 1) |
| ENK-CFG-021 | The daemon shall write the owned sections through the UCI library with its own delta directory, on configuration files arranged at installation so that the daemon's user may replace them and no other file in `/etc/config`. | Measurement before internal design (M-5b below); test |
| ENK-CFG-022 | `rpcd` shall not be a dependency. | Inspection of the package list |

M-5b (an addition to the handoff of RFC 006): confirm on the virtual machine that a non-root
user with a private delta directory and ownership of `/etc/config/wireless`, `network`,
`firewall`, `dhcp` — with the directory's sticky bit set — can commit through `libuci` and
cannot replace any other file. If the platform makes this impractical, the fallback is a small
root helper that accepts only the owned sections from the daemon over a unix socket; that choice
is recorded in the internal design, not here.

### 8. What the operator sees

| State of a change | Operator wording (English; Japanese in the UI RFC) |
|---|---|
| validating | "Checking…" |
| applying, health check running | "Saving… The Wi-Fi restarts for a few seconds." |
| waiting for confirmation | "Saved. Join the new management Wi-Fi within 3 minutes, or the change is undone." with the new name and key and a countdown. *(Amended by RFC 015: the earlier text asked the operator to press *Keep*; there is no button. RFC 016: 10 minutes.)* |
| settled | "Saved." |
| undone, health check failed | "Not saved: the device could not start the network this way. Your previous settings are back." + the reason in plain words |
| undone, no confirmation | "Undone: you did not confirm in time. Your previous settings are back." *(RFC 017 §5.5: "Undone: the new management Wi-Fi was not joined in time. Your previous settings are back.")* |
| undone, after a restart | "A change from before the restart was not finished and has been undone." |
| drift | "Settings were changed outside Enkrnett. Restore my settings?" |

Never: "apply", "commit", "rollback", "transaction" (RFC 009 table C).

## Consequences

| Affected | Change |
|---|---|
| D-12 | Decided: Enkrnett owns the transaction; `rpcd` not used. |
| RFC 005 open question 3 (the gap analysis' proposed "ENK-REL-006") | Closed by ENK-CFG-006 to 010. |
| RFC 006 handoff | Gains M-5b. |
| Draft ED §10.3 states `DESIRED, STAGED, APPLYING, VERIFYING, ROLLED_BACK, SYNCHRONIZED` | Replaced by §4 and §8. |
| Draft API routes `PUT /config` and `POST /config/apply` | One operation: save the desired document; the API RFC names it. |
| Gap analysis GAP-050 to 055 | Closed. |

## Alternatives considered

| Alternative | Why not |
|---|---|
| `rpcd` apply/confirm/rollback | Rollback lost at restart; session password on disk; two ACL systems; a dependency the image does not otherwise need (M-5). |
| Confirmation for every change | Friction without safety for changes the device can verify itself. |
| No confirmation at all, health check only | A healthy device with a new management key is unreachable for an owner who mistyped nothing but lost the display; 180 s of patience is the cheaper insurance. |
| Daemon as `root` | Threat 37 (RFC 007); on MT7628 there is no sandbox to fall back on. |
| Let the operator edit hostapd options | Out of scope by RFC 001; the device profile chooses them. |

## Open questions

| # | Question | Closed by |
|---|---|---|
| 1 | M-5b result and the file-permission arrangement | Internal design |
| 2 | Exact health-check criteria per network (which ubus fields) | Internal design, derived from ENK-CFG-007 |
| 3 | Japanese wording of §8 | Operator UI RFC: RFC 017 §5.5 |
