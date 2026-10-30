# 7. Guest modes and status

From RFC 012, as amended by RFC 016 (the notice's last paragraph) and RFC 017 (Japanese wording).

The guest network has two modes. The operator chooses between **Open to every device** (the
default: devices that can encrypt do, older ones connect without encryption) and **Encrypted
only** (older devices cannot join). The status always shows how many connected devices are
encrypted and how many are not. The device never claims more than the radio link, and the
operator gets a notice to show guests that says the same in plain words.

## 7.1 Modes

| `guest.mode` | What the device does | On screen (EN / JA) |
|---|---|---|
| `open_to_all` (default) | OWE Transition Mode: a visible open BSS and a hidden OWE BSS, cross-referenced | **Open to every device.** Devices that can encrypt do; older devices connect without encryption. / すべての端末が接続できる |
| `encrypted_only` | OWE only, one BSS | **Encrypted only.** Older devices cannot join this network. / 暗号化できる端末のみ |

## 7.2 Effective state of the guest network

| Field | Meaning | Source on the device |
|---|---|---|
| `up` | Both BSSes (or the one) are running | `network.wireless status`, `hostapd.* get_status` |
| `mode` | The mode the radio actually runs | generated configuration confirmed by `get_status` |
| `clients_encrypted` | Stations associated and authorized on the OWE BSS with `mfp: true` | `hostapd.<owe-iface> get_clients` |
| `clients_unencrypted` | Stations associated on the open BSS | `hostapd.<open-iface> get_clients` |
| `checked_ago` | Seconds since the figures were read | daemon |

## 7.3 Requirements

{{#include requirements/owe.md}}

{{#include requirements/guest.md}}

## 7.4 What the operator sees

```text
Guest Wi-Fi  "Cafe Example"        ● running
Open to every device
12 devices connected: 10 encrypted · 2 not encrypted
```

```text
Guest Wi-Fi  "Cafe Example"        ● running
Encrypted only
8 devices connected, all encrypted
```

Japanese and every other line: chapter 10 §3 (RFC 017 §5.1). Colour marks only running / not
running.

## 7.5 What is protected, and what is not

One text, used three times: the operator's "What this protects" screen, the printable guest
notice, and the documentation. Japanese: RFC 017 §5.8.

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
>
> This access point keeps no record of the devices that connect to it.

## 7.6 Client compatibility text

The UI answers "which devices can encrypt?" with one sentence per platform. Until M-4a and M-4
confirm or correct them on real devices, the sentences carry the ledger's values: Android 10 with
a capable chip; iPhone 11 or later (iOS 16.1+); macOS 13 or later; Windows 10 version 2004 with a
capable driver (ENK-OWE-019).
