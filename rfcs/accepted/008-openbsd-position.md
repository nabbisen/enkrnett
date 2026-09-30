# RFC 008 — Where OpenBSD fits

**Status.** Accepted (2026-09-30)
**Tracks.** The owner's question of 2026-09-30: is there room for OpenBSD in Enkrnett?
**Touches.** Nothing in the first release. The RFC on build, release, and update; a later RFC on
a site controller.

Terms are defined in [RFC 001](./001-scope-people-vocabulary.md).

## Summary

OpenBSD cannot be the Enkrnett access point, because its wireless stack offers neither OWE nor
WPA3 in access-point mode and has no Bluetooth. It fits the project's infrastructure, and later a
site controller or an expert two-box deployment. This RFC records the question, the facts, and
the decision so that the question need not be answered twice.

## Motivation

The owner prefers OpenBSD for its proactive security and asked whether it has a place. The
earlier study had compared it once; that comparison was not recorded in the repository.

## Facts

| Fact | Source |
|---|---|
| OpenBSD supports access-point mode ("Host AP") on a list of drivers: acx, ath, athn, bwfm, pgt, ral, ural, rtw, rum, wi. Most are 802.11a/b/g/n parts. | [OpenBSD FAQ 6][faq6] |
| In access-point and client mode, `ifconfig(8)` accepts `wpaprotos wpa1, wpa2` and `wpaakms psk, sha256-psk, 802.1x`. SAE (WPA3-Personal) and OWE do not appear. | [`ifconfig(8)`][ifconfig] |
| The Bluetooth stack was removed in OpenBSD 5.6. | [OpenBSD 5.6 release notes][obsd56] |
| Enkrnett does not develop kernels, drivers, or Wi-Fi stacks. | RFC 001 §1 |
| The operator needs one device that isolates guests by itself. | RFC 002 |

## Decision

| Place | Decision | When |
|---|---|---|
| **The access point** | No. OWE is the product, and OpenBSD's access-point mode has no OWE. Adding it would be kernel work. | — |
| **Two-box deployment**: OpenBSD as the venue's router and firewall, an Enkrnett device behind it | Not part of the product. It is an expert arrangement that RFC 002 makes unnecessary for the operator. Nothing in Enkrnett prevents it. | Expert documentation only, if asked for |
| **Project infrastructure**: the pinned package mirror, the documentation site, release signing | **Preferred platform**, to be decided in the RFC on build, release, and update. `httpd`, `relayd`, and `signify` cover the needs. | That RFC |
| **Site controller** for several access points | A candidate platform. | The milestone that introduces a controller |
| **Design influence** | Least privilege, few services, and safe defaults are already the direction of RFC 002 and RFC 007. On OpenWrt the means are the firewall, `ujail`, and `seccomp` where the target offers them. | Now |

## When to revisit

| Trigger | Then |
|---|---|
| OpenBSD's `ifconfig(8)` gains SAE or OWE in access-point mode, and a supported driver offers 802.11ac or later in that mode | Reconsider the access point, starting with a device profile |
| The project needs infrastructure hosts | Apply the "preferred platform" row |

## Consequences

| Affected | Change |
|---|---|
| RFC 001 §1 | Unchanged. OpenBSD is neither in the "is" nor the "is not" column; it is a platform choice for infrastructure. |
| The RFC on build, release, and update | Names OpenBSD as the preferred platform for the mirror and for signing, or records why not. |

## Alternatives considered

| Alternative | Why not |
|---|---|
| Port OWE to OpenBSD's wireless stack | Kernel work outside the project's scope; the drivers with access-point mode are old parts. |
| Run OpenBSD as the router and buy a certified OWE access point | Two boxes and two vendors for a person without IT training. |

## References

[faq6]: https://www.openbsd.org/faq/faq6.html#Wireless
[ifconfig]: https://man.openbsd.org/ifconfig.8
[obsd56]: https://www.openbsd.org/56.html

- OpenBSD FAQ, "Wireless Networking": [link][faq6]
- OpenBSD, `ifconfig(8)`, IEEE 802.11 section: [link][ifconfig]
- OpenBSD 5.6 release notes: [link][obsd56]
