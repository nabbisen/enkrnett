# 6. Settings and changes

From RFC 011, as amended by RFC 014 (ENK-CFG-023 to 026), RFC 015 (what counts as confirmation),
RFC 016 (ENK-CFG-008; `enk_setup`; 600 s), and RFC 017 (wording).

Enkrnett keeps one **settings document** (RFC 011 calls it the desired document), translates it
into the OpenWrt configuration sections it **owns**, applies it through OpenWrt's own reload, and
verifies the **effective state** through ubus and the kernel. Every change is a transaction with
a **pending marker on flash**: the device checks its own health after applying and undoes the
change by itself if the check fails or if it restarts before the change is settled. The operator
confirms only a change after which they had to join a new management Wi-Fi, and they confirm it
by joining. `rpcd` is not used. The daemon is not `root`.

## 6.1 Three representations

| Representation | Holds | Owner | Where |
|---|---|---|---|
| **Settings** (desired) | What the operator asked for, with a version number | Operator, through the API | A file in the daemon's state directory |
| **Platform** | The UCI sections that implement the settings | The daemon | `/etc/config/*`, only the sections the daemon owns |
| **Effective** | What the device is actually doing | The platform, read through ubus and sysfs | Never stored; read on demand |

The API exposes settings and effective state, never platform. The UI shows the effective state
with the settings beside it where they differ.

## 6.2 The settings document

| Field | Type | Rule |
|---|---|---|
| `version` | integer | Increases by one per settled change |
| `country` | two letters | From the device profile when the profile fixes it (ENK-RF-002); otherwise chosen once at setup; changed only through Erase |
| `guest.name` | text | 1–32 bytes of UTF-8 when `guest.mode` is `encrypted_only`, 1–29 bytes when `open_to_all` (ENK-CFG-016) |
| `guest.mode` | `encrypted_only` \| `open_to_all` | Chapter 7 |
| `guest.enabled` | boolean | Lets the operator switch the guest network off without losing its settings |
| `management.name` | text | Set by the device at setup; changeable (needs confirmation) |
| `management.key` | text | Generated (ENK-OWN-014); never in this document as returned; replaced through its own operation |

Nothing else is configurable in the first release. Radio channel, width, power, bands, and every
hostapd detail are chosen by the daemon from the device profile and the country.

## 6.3 Owned platform sections

| Configuration | Sections the daemon owns | Named |
|---|---|---|
| `wireless` | every `wifi-device`; the `wifi-iface` sections for the guest, management, and setup networks | `enk_radio*`, `enk_guest`, `enk_mgmt`, `enk_setup` |
| `network` | the interfaces and bridges of the guest, management, and setup networks | `enk_guest`, `enk_mgmt`, `enk_setup` |
| `firewall` | the zones, forwardings, and rules of chapter 2 | `enk_*` |
| `dhcp` | the DHCP and DNS settings of the three networks, one instance per network | `enk_guest`, `enk_mgmt`, `enk_setup` |

## 6.4 The transaction

```text
 settings(vN) ─save─► validate ─► snapshot owned sections
                                ─► write pending marker {from: vN-1, to: vN, needs_confirmation}
                                ─► write owned sections
                                ─► reload (network, wifi) through ubus
                                ─► set bridge-port isolation; health check (effective == settings, within 60 s)
                                      │ ok, no confirmation needed ──► settle: remove marker, drop snapshot
                                      │ ok, confirmation needed ───► disconnect management stations
                                      │                              ─► wait ≤ 600 s for the first owner request
                                      │                                 on a connection accepted after that ──► settle
                                      │ failed, or no such request in time, or restart with marker ──► undo:
                                             restore snapshot ─► reload ─► health check ─► remove marker ─► report
```

Confirmation is needed only for a change of the management network's name or key: after it, a
healthy device can still be unreachable for its owner. The device can verify everything else
itself. The health check reads back every property of chapter 2 that the platform implements
(ENK-CFG-026): the `isolated` flag of every bridge port, one DNS/DHCP listener per network, the
daemon's listener bound to the management address only, PMF on the OWE BSS, the effective mode
and names.

{{#include requirements/cfg.md}}

## 6.5 What the operator sees

The sentences, in English and Japanese, are chapter 10 §3 (RFC 017 §5.5). States: checking;
saving; saved, join the new management Wi-Fi within 10 minutes; saved; not saved with the reason
and the previous settings back; undone because the new management Wi-Fi was not joined in time;
undone after a restart; settings changed outside Enkrnett. Never "apply", "commit", "rollback",
"transaction", "Keep".

## 6.6 Open

| # | Question | Closed by |
|---|---|---|
| 1 | Measurement M-5b (non-root commit through `libuci` with a private delta directory) and the file-permission arrangement; fallback: a small root helper over a unix socket | Internal design, M1 |
| 2 | Exact health-check criteria per network (which ubus fields) and further `change.detail` codes | Internal design, M1; added to chapter 9 by amendment |
