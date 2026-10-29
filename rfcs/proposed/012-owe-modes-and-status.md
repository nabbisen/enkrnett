# RFC 012 — Guest network modes and protection status

**Status.** Proposed
**Tracks.** Decision D-08 (accepted direction), amendment A-2, requirements ENK-SCOPE-004 and the
proposed ENK-OWE-007 / ENK-GUEST-001 of the gap analysis.
**Touches.** The desired document (RFC 011 `guest.mode`); the effective state; the management UI;
the guest notice; the OWE policy in the firmware.

Terms are defined in [RFC 001](../accepted/001-scope-people-vocabulary.md) as amended by
[RFC 009](./009-vocabulary-amendment-operator-words.md). Facts about OWE are in the gap analysis
ledger (F-OWE-01 to 13) and in measurement M-6.

## Summary

The guest network has two modes. The operator chooses between **Open to every device** (the
default: devices that can encrypt do, older ones connect without encryption) and **Encrypted
only** (older devices cannot join). The status always shows how many connected devices are
encrypted and how many are not. The device never claims more than the radio link, and the
operator gets a notice to show guests that says the same in plain words.

## Motivation

| Fact | Source |
|---|---|
| OWE encrypts each client's radio link individually and authenticates nobody; in Transition Mode, clients without OWE join an open BSS unencrypted. | RFC 8110 §7; Wi-Fi Alliance deployment guide §2.11 |
| OWE needs a recent client: Android 10 with a capable chip, iPhone 11 or later, Windows 10 2004 with a capable driver. | Ledger F-OWE-06 |
| Clients show an OWE network like an open one, by design. | RFC 8110 §6; F-OWE-07 |
| Transition Disable is off by default and the specification warns against it for shared names. | F-OWE-04 |
| Per-BSS station lists with `authorized` and `mfp` are available through ubus; the device cannot observe its own beacons. | M-6; gap analysis 03 §6.3 |
| The regulator asks providers to tell guests which protection is in place. | F-REG-08 |
| "Transition" and "secure" must not reach the operator. | RFC 009 |

## Design

### 1. Modes

| `guest.mode` | What the device does | Operator wording |
|---|---|---|
| `open_to_all` (default) | OWE Transition Mode: a visible open BSS and a hidden OWE BSS, cross-referenced | **Open to every device.** Devices that can encrypt do; older devices connect without encryption. |
| `encrypted_only` | OWE only, one BSS | **Encrypted only.** Older devices cannot join this network. |

| ID | Requirement | Verification |
|---|---|---|
| ENK-OWE-007 | The guest network shall offer exactly these two modes, named as above in operator-facing text. | Inspection |
| ENK-OWE-008 | The default at setup shall be `open_to_all`. The setup screen shall state both modes' consequence in one line each and shall not pre-select silently: the default is shown as chosen, with the other beside it. | Inspection; M-7 |
| ENK-OWE-009 | In both modes PMF shall be required on the OWE BSS (the platform does this; the daemon shall verify it in the effective state). | Test |
| ENK-OWE-010 | The device shall not send Transition Disable in the first release. | Inspection of the generated configuration |
| ENK-OWE-011 | When the operator switches from `open_to_all` to `encrypted_only`, the UI shall show, before saving, the number of currently connected devices that will lose the network (those not encrypted). | Test |
| ENK-OWE-012 | When the operator switches from `encrypted_only` to `open_to_all`, the UI shall say that encrypted devices may take up to a few minutes to reconnect. | Inspection (M-6 observation 2) |

### 2. Effective state of the guest network

| Field | Meaning | Source on the device |
|---|---|---|
| `up` | Both BSSes (or the one) are running | `network.wireless status`, `hostapd.* get_status` |
| `mode_effective` | The mode the radio actually runs | generated configuration confirmed by `get_status` |
| `clients_encrypted` | Stations associated and authorized on the OWE BSS with `mfp: true` | `hostapd.<owe-iface> get_clients` |
| `clients_unencrypted` | Stations associated on the open BSS | `hostapd.<open-iface> get_clients` |
| `last_checked` | Seconds since the figures were read | daemon |

| ID | Requirement | Verification |
|---|---|---|
| ENK-OWE-013 | The effective state shall report the two counts separately and shall never sum them into one "connected" figure without both parts. | Test |
| ENK-OWE-014 | Counts shall be current readings, not stored histories (decision D-15). | Inspection |
| ENK-OWE-015 | The device shall state what it can observe: configured, running, and associated. It shall not claim knowledge of what is on the air. | Inspection of wording |

### 3. What the operator sees

The status screen, in `open_to_all`:

```text
Guest Wi-Fi  "Cafe Example"        ● running
Open to every device
12 devices connected: 10 encrypted · 2 not encrypted
```

In `encrypted_only`:

```text
Guest Wi-Fi  "Cafe Example"        ● running
Encrypted only
8 devices connected, all encrypted
```

| ID | Requirement | Verification |
|---|---|---|
| ENK-OWE-016 | The status shall show the mode and the two counts on the first screen, with no further tap. | Inspection |
| ENK-OWE-017 | The status shall never use a single badge, colour, or word that summarises protection as good or bad. Colour marks only *running* / *not running*. | Inspection; review against RFC 001 principle 4 |
| ENK-OWE-018 | A "What this protects" link shall open the text of §4, the same text as the guest notice. | Inspection |

### 4. What is protected, and what is not

One text, used three times: on the operator's "What this protects" screen, in the printable guest
notice, and in the documentation. Technical names appear once, in brackets, because guests' phones
show them.

> **This Wi-Fi encrypts the connection between your device and the access point** if your
> device supports it (most phones and laptops since about 2020; the standard is called
> "Enhanced Open" or "OWE"). Older devices connect without encryption.
>
> The encryption covers the radio link only. It does not check who runs the network, it does not
> protect you from a fake network with the same name, and it does not protect what happens
> beyond this access point. Use websites and apps that encrypt end to end (the lock in your
> browser) for anything private.
>
> Devices on this Wi-Fi are kept apart from each other and from the shop's own network by the
> access point. This reduces, but does not remove, what another guest could attempt.

| ID | Requirement | Verification |
|---|---|---|
| ENK-GUEST-001 | The device shall provide the guest notice as a printable page in every language of the UI, with the network name filled in. | Inspection |
| ENK-GUEST-002 | The notice and every operator-facing statement about protection shall be consistent with RFC 007 section 7 (remaining risks) and shall be reviewed whenever that section changes. | Review at release |

### 5. Client compatibility text

The operator will ask "which devices can encrypt?". The UI answers with the table of ledger
F-OWE-06 reduced to one sentence per platform, and the setup screen for `encrypted_only` repeats
the oldest supported models. M-4 confirms or corrects the sentences on real devices before
release; until then the text carries the ledger's values.

| ID | Requirement | Verification |
|---|---|---|
| ENK-OWE-019 | The compatibility text shall name the oldest supported system version and hardware generation per platform, and shall be updated from M-4 before the first release. | Release checklist |

## Consequences

| Affected | Change |
|---|---|
| D-08 | Decided by this RFC. |
| Draft `wireless_security = OWE \| OPEN \| WPA2 \| WPA3 \| TRANSITION \| UNKNOWN` | Replaced by §2. |
| Draft `associated_owe_clients` | Replaced by the two counts. |
| Gap analysis GAP-031, 032, 037, 072 | Closed. |
| RFC 011 | Uses `guest.mode` with these two values. |

## Alternatives considered

| Alternative | Why not |
|---|---|
| `encrypted_only` as default | Every guest with an older device fails silently; the operator learns of it as complaints. |
| Ask the operator at setup without a default | One more decision for a non-expert, with the consequences not yet visible. |
| A traffic-light badge | Summarises what cannot be summarised; forbidden by principle 4. |
| Transition Disable on | Irreversible on clients, dangerous with common names (F-OWE-04). |
| Hiding the unencrypted count to avoid alarm | Exactly the theater ENK-SCOPE-004 forbids. |

## Open questions

| # | Question | Closed by |
|---|---|---|
| 1 | Japanese wording of §1, §3, §4 | Operator UI RFC |
| 2 | Compatibility sentences | M-4 |
