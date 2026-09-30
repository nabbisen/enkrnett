# RFC 004 — Ownership, credentials, and authorization

**Status.** Accepted (2026-09-29)
**Tracks.** How a person becomes the owner of a device, what proves it afterwards, and how
ownership ends. Startup decisions D-06 and D-07.
**Touches.** The daemon (state, credentials, sessions); the setup flow of the management UI;
device profiles (button, light); operator guides.

Terms are defined in [RFC 001](./001-scope-people-vocabulary.md). Networks are defined in
[RFC 002](./002-network-topology-and-separation.md). The channel is defined in
[RFC 003](./003-management-channel-and-ui-delivery.md).

## Summary

An unclaimed device opens a **setup window** when it is powered on. Whoever completes setup in
that window becomes the **owner** and chooses the **owner password**. Every change afterwards
needs that password. There is one owner and no other role. A lost password or key is recovered by
**reset**, which erases everything.

## Motivation

| Fact | Source |
|---|---|
| Enkrnett runs on routers that someone reflashes. No Enkrnett PIN is printed on them. | — |
| OpenWrt's reset erases everything written after installation. A secret created on the device does not survive it. | [OpenWrt][reset] |
| A value derived from the hardware address is public: the address is broadcast. | — |
| Push-button pairing carries no secret. A device that acts first within the window wins. | [Wi-Fi Alliance][wps], [hostapd][hostapd-wps] |
| A credential sent over an unprotected connection can be captured and replayed. | [RFC 6750][rfc6750] §5 |
| Safari deletes a site's stored data after seven days without a visit. A credential kept only in the browser can vanish. | [WebKit][webkit-itp] |
| Established products pair through a password-authenticated key exchange with a code printed at manufacture. | [Matter][matter], [Apple][homekit] |

The last row describes the strongest known approach. It needs a secret that reaches the operator
outside the network, which reflashed hardware does not have. This RFC therefore takes the
simplest approach that is honest about its limits, and leaves room for the stronger one
(section 9).

## Design

### 1. States

```text
              power on            operator confirms        owner logs in
  UNCLAIMED ───────────►  SETUP  ──────────────────►  PENDING  ─────────────►  CLAIMED
      ▲                     │  ▲                         │                        │
      │    window closes    │  │   10 minutes pass,      │                        │
      └─────────────────────┘  │   or the device restarts│                        │
                               └─────────────────────────┘                        │
      ▲                                                                           │
      └────────────────────────────────── reset ──────────────────────────────────┘
```

| State | Meaning | Networks offered |
|---|---|---|
| `UNCLAIMED` | No owner. Setup window closed. | None |
| `SETUP` | No owner. Setup window open. | Setup network |
| `PENDING` | Setup was confirmed. The claim is not final yet. | Management network |
| `CLAIMED` | The device has an owner. | Management network; guest network according to the Wi-Fi configuration |

| From | Event | To |
|---|---|---|
| `UNCLAIMED` | The device finishes starting | `SETUP` |
| `SETUP` | The setup window closes | `UNCLAIMED` |
| `SETUP` | The operator confirms setup | `PENDING` |
| `PENDING` | The owner logs in through the management network | `CLAIMED` |
| `PENDING` | 10 minutes pass without that login, or the device restarts | `SETUP` |
| `CLAIMED` | The device restarts | `CLAIMED` |
| `CLAIMED` | Reset | `UNCLAIMED`, then the device restarts |

These are the only states and the only transitions. A device that restarts without an owner
starts in `UNCLAIMED` and opens the setup window, whatever its state was before.

### 2. Setup window

| ID | Requirement | Verification |
|---|---|---|
| ENK-OWN-001 | A device without an owner shall open the setup window when it finishes starting. Nothing else shall open it. | Test |
| ENK-OWN-002 | The setup window shall stay open for 30 minutes. Time spent in `PENDING` does not count. If a claim session exists when the 30 minutes end, or the device has returned from `PENDING`, the window shall stay open for 15 minutes more, once. | Test |
| ENK-OWN-003 | When the window closes, the device shall stop offering any Wi-Fi network. | Test |
| ENK-OWN-004 | The setup network's name shall end with the last four characters of the hardware address printed on the device's label. | Test per device profile |

Opening the window needs a physical act: connecting power. To open it again, the operator
disconnects and reconnects power. A return from `PENDING` does not open a window; it continues
the one that was open.

### 3. Claim

| Step | Who | What happens |
|---|---|---|
| 1 | Operator | Joins the setup network whose name matches the label. |
| 2 | Operator | Opens the management UI. |
| 3 | Device | Grants this client the claim session. Any other client is told that setup is in progress elsewhere. |
| 4 | Operator | Asks the device to identify itself. The device's light blinks. The operator confirms seeing it. |
| 5 | Operator | Chooses the country, the guest network's name, and the owner password. |
| 6 | Device | Checks every value. Shows the management network's name, the management key, and the address of the management UI. |
| 7 | Operator | Keeps them, then confirms. |
| 8 | Device | State `PENDING`. Ends the setup network. Starts the management network. |
| 9 | Operator | Joins the management network and logs in with the owner password. |
| 10 | Device | State `CLAIMED`. The claim is **final**. Starts the guest network. |

| ID | Requirement | Verification |
|---|---|---|
| ENK-OWN-005 | At most one claim session shall exist. It shall end after 5 minutes without a request from its client. | Test with two clients |
| ENK-OWN-006 | Where the device profile declares a light that Enkrnett can control, the claim shall include the identify step. Where it declares none, the UI shall say that the step is not available on this device. | Test per device profile |
| ENK-OWN-007 | The device shall change nothing that persists until the operator confirms in step 7. | Test: abandon at every step, restart, inspect |
| ENK-OWN-008 | A claim shall become final only when the owner has logged in through the management network. If this has not happened 10 minutes after step 8, or if the device restarts before it, the device shall forget everything the claim created and return to `SETUP`. | Test, including power loss after step 8 |

ENK-OWN-008 protects the operator who did not manage to keep the management key. Without it, a
confirmed but unreachable device would need a reset.

### 4. Owner password

| ID | Requirement | Verification |
|---|---|---|
| ENK-OWN-009 | The owner password shall have 10 to 64 characters from the printable ASCII set. | Test |
| ENK-OWN-010 | The device shall refuse an owner password that equals the name of one of its networks, the management key, or the device identifier, or that appears in the list of common passwords shipped with the release. | Test |
| ENK-OWN-011 | The device shall keep the owner password only as a salted hash made with a function designed to be slow. It shall never return, display, or log the password. | Inspection; search of all outputs |
| ENK-OWN-012 | After 5 consecutive failed logins the device shall refuse further attempts for 30 seconds, doubling after each further failure, up to 15 minutes. A successful login or a restart ends the delay. | Test |
| ENK-OWN-013 | The owner shall be able to change the owner password by entering the current one. The change ends every session. | Test |

Only ASCII is accepted so that the same password can be typed on every phone and keyboard
without ambiguity.

### 5. Management key

| ID | Requirement | Verification |
|---|---|---|
| ENK-OWN-014 | The device shall generate the management key. The operator does not choose it. | Inspection |
| ENK-OWN-015 | The management key shall carry at least 80 bits of randomness, shall have at most 24 characters, and shall use no characters that are easily mistaken for one another. | Inspection of the generator; statistical test |
| ENK-OWN-016 | The owner shall be able to display the management key and to replace it with a new one. A replacement becomes final as in ENK-OWN-008; otherwise the previous key returns. | Test |

A generated key cannot be guessed from a recording of the network, even with WPA2.

### 6. Sessions

| ID | Requirement | Verification |
|---|---|---|
| ENK-AUTH-001 | A session shall be created only by presenting the owner password. | Test |
| ENK-AUTH-002 | A session shall be identified by a random value of at least 128 bits, which the client sends in a request header. | Inspection |
| ENK-AUTH-003 | A session shall end after 15 minutes without a request, after 12 hours in any case, at logout, at restart, at reset, and when the owner password changes. | Test |
| ENK-AUTH-004 | Session durations shall be measured with a clock that does not depend on the date and time being set. | Test with a wrong system date |
| ENK-AUTH-005 | At most 4 sessions shall exist. Creating another ends the one that has been idle longest. | Test |
| ENK-AUTH-006 | Sessions shall be kept in memory only. | Inspection |

### 7. Authorization

There is one role: the owner.

| Resource | No session | Claim session | Owner session |
|---|---|---|---|
| Files of the management UI | ✓ | ✓ | ✓ |
| Greeting: product name, API version, state (section 1), languages | ✓ | ✓ | ✓ |
| Claim (section 3) | ✗ | ✓ | ✗ |
| Everything else | ✗ | ✗ | ✓ |

| ID | Requirement | Verification |
|---|---|---|
| ENK-AUTH-007 | Without a session the device shall disclose nothing beyond the greeting. In particular it shall not disclose versions of the firmware or its components, the device identifier, or any configuration. | Test of every resource without a session |
| ENK-AUTH-008 | Authorization shall be decided on the device for every request. | Test with a modified client |

### 8. Reset

| ID | Requirement | Verification |
|---|---|---|
| ENK-OWN-017 | Reset shall be started by holding the reset button as the platform defines, or by the owner in the management UI after entering the owner password again. | Test per device profile |
| ENK-OWN-018 | Reset shall erase the owner password, the management key, the Wi-Fi configuration, every session, the device identifier, and every log. Nothing that the operator or the device created shall remain. | Inspection after reset |
| ENK-OWN-019 | Reset shall be the only way to end ownership. | Inspection |
| ENK-OWN-020 | The device shall create a new random device identifier when it has none. The identifier is not a secret. | Test |
| ENK-OWN-021 | Enkrnett shall keep the platform's meaning of each button and shall assign no further meaning to the duration of a press. | Test per device profile |

| Situation | Recovery |
|---|---|
| Owner password forgotten | Reset, then set up again |
| Management key lost | Reset, then set up again |
| Both lost | Reset, then set up again |

One recovery path for every case. Setting up again takes a few minutes.

OpenWrt's default button behavior: a press shorter than 1 second restarts the device; holding for
5 seconds or longer resets it ([source][reset]).

### 9. Limits that remain

Enkrnett states these to the operator where they matter (ENK-SCOPE-004).

| # | Limit | Who can exploit it | How it is noticed or reduced |
|---|---|---|---|
| 1 | Someone else claims the device during the setup window. | A person within radio range at that moment | The operator cannot finish setup. Reset and repeat. |
| 2 | A fake setup network relays the operator's setup and learns what was entered. | An active attacker within range at that moment | The name matches the label (ENK-OWN-004). The identify step defeats an attacker who does not relay. A relaying attacker is not detected. |
| 3 | On a setup network joined without OWE, what the operator enters can be overheard. | A listener within range at that moment | The UI tells the operator when the phone joined without encryption, before anything is entered. |
| 4 | With WPA2, someone who knows the management key can decrypt another client's session, including the owner password at login. | A person who was given or saw the management key | The key is private to the owner. Phones that support WPA3 are not affected. |
| 5 | Anyone who can touch the device can restart or reset it. | A person at the device | After a reset the guest network disappears. The guide advises placing the device out of reach, as the regulator does. |
| 6 | Guessing the owner password. | A client on the management network | ENK-OWN-012. |

Limits 2, 3, and 4 can be closed by a secret that reaches the operator outside the network,
combined with a password-authenticated key exchange. That is a later RFC. It needs a way to
give each device its own secret at installation.

## Consequences

| Affected | Change |
|---|---|
| Draft "printed PIN", "QR secret" | Withdrawn. |
| Draft "pairing record", "principal", "pairing lifecycle" | Withdrawn. Replaced by section 1. |
| Draft roles Owner, Operator, Read-only Auditor, Device Controller; permission matrix; ownership transfer | Deferred. Transfer of ownership is: reset, then the new owner sets up. |
| Draft acceptance criterion "a read-only auditor can inspect diagnostics" | Withdrawn from the first release. |
| Draft device lifecycle states `PAIRED`, `CONFIGURED`, `ACTIVE`, `DEGRADED`, `RECOVERY` | Replaced by the four states of section 1 for ownership. States of configuration and health are defined by their own RFCs and are not mixed with ownership. |
| Draft "device identity survives factory reset" | Withdrawn (ENK-OWN-018, ENK-OWN-020). |
| Draft audit events | Replaced by log lines. Their content is defined by a later RFC. |
| Draft field `updated_by` and similar | Withdrawn. There is one owner. |

## Alternatives considered

| Alternative | Why not |
|---|---|
| A credential per phone, stored in the browser | Browser storage can vanish. Needs records per client. |
| Printed code with a password-authenticated key exchange | No printed code exists on the target hardware. Kept as the path for later (section 9). |
| Button press to open the window | Buttons differ per device and their press durations are already taken by the platform. Connecting power is the same act on every device. |
| The management key as the only credential | The API would then accept every device on the management network. |
| Operator chooses the management key | Chosen keys are weak. WPA2 allows guessing from a recording. |
| Recovery that keeps the configuration | A second way into a claimed device. One path is easier to explain and to secure. |
| Unicode in the owner password | The same password can be produced differently by different keyboards. |

## Verification

| Item | Where |
|---|---|
| Every requirement above | Tests derived from this RFC |
| Whether non-experts complete the claim without help, and how long it takes | Usability test, RFC 006, M-7 |
| Whether phones keep the owner password when asked to save it on an HTTP page | RFC 006, M-1 |
| Durations in ENK-OWN-002, 005, 008 | Reviewed after the usability test |

## Open questions

| # | Question |
|---|---|
| 1 | The function for ENK-OWN-011 must run within about half a second on the slowest supported device. Its choice belongs to the internal design and is measured in RFC 006, M-2. |
| 2 | The size of the list in ENK-OWN-010 is bounded by the footprint budget (decision D-10). |

## References

[reset]: https://github.com/openwrt/openwrt/blob/openwrt-25.12/package/base-files/files/etc/rc.button/reset
[wps]: https://www.wi-fi.org/system/files/Wi-Fi_Protected_Setup_Best_Practices_v2.0.2.pdf
[hostapd-wps]: https://w1.fi/cgit/hostap/plain/hostapd/README-WPS
[rfc6750]: https://www.rfc-editor.org/rfc/rfc6750.txt
[webkit-itp]: https://webkit.org/tracking-prevention/
[matter]: https://csa-iot.org/wp-content/uploads/2024/11/24-27349-006_Matter-1.4-Core-Specification.pdf
[homekit]: https://support.apple.com/guide/security/communication-security-sec3a881ccb1/web

- OpenWrt, reset button handler: [source][reset]
- Wi-Fi Alliance, Wi-Fi Protected Setup Best Practices v2.0.2: [PDF][wps]
- hostapd, README-WPS: [text][hostapd-wps]
- RFC 6750, Bearer Token Usage: [text][rfc6750]
- WebKit, Tracking prevention: [link][webkit-itp]
- Connectivity Standards Alliance, Matter 1.4 Core Specification: [PDF][matter]
- Apple Platform Security, Communication security: [link][homekit]
