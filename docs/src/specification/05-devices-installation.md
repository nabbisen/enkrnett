# 5. Devices, baseline, installation

From RFC 005, as amended by RFC 016 (ENK-TIER-002; *settings*). Update and upgrade mechanics are
chapter 11.

| Subject | Decision |
|---|---|
| Support | Defined per device profile, in three tiers |
| First Tier 1 device | A `standard`-class device. 8 MB / 64 MB devices are built and measured from the first day and promoted when they pass. |
| OpenWrt baseline | 25.12 only |
| Installation | A Tier 1 device can be installed by a non-expert |
| Update and upgrade | One path, a signed full image (chapter 11) |
| Radio regulation | The country is established during setup. No prebuilt image is published for a market until the legal basis is documented. |

## 5.1 Tiers

| Tier | Meaning | Promise to operators |
|---|---|---|
| **1** | Tested on the hardware for every release. Meets every requirement of this book. Can be installed by a non-expert. | Supported |
| **2** | Built for every release and checked automatically against the footprint budgets. Tested on hardware before promotion. | None |
| **3** | Maintained by contributors. | None |

{{#include requirements/tier.md}}

### Criteria for Tier 1

| # | Criterion | Verified by |
|---|---|---|
| 1 | The OpenWrt baseline publishes an image for the device, including one that the vendor's firmware accepts. | Download server |
| 2 | Non-experts installed it with the guide alone. | M-7 |
| 3 | Non-experts recovered from an interrupted installation with the guide alone, or the device keeps its previous firmware when an installation fails. | M-7 |
| 4 | OWE with PMF works in access-point mode with the client matrix. | M-4 |
| 5 | The radio that carries the management network allows at least 4 interfaces. | Driver source; M-4 |
| 6 | The device has a reset button that OpenWrt reads. | Test |
| 7 | The footprint budgets (chapter 8) are met. | M-2, M-3 |
| 8 | The device can be bought in the target market when the release is published. | Owner |
| 9 | The legal basis for operating the device with Enkrnett in the target market is documented. | §5.4 |
| 10 | The setup network's suffix matches the address printed on the label (ENK-OWN-004, ENK-TIER-002). | M-7 |

## 5.2 Install classes

| Class | Meaning |
|---|---|
| **Self-install** | The operator installs through the vendor's own firmware update screen, with one file, following the guide. |
| **Expert install** | Anything else, including installation as a package on a device that already runs OpenWrt. |

{{#include requirements/inst.md}}

## 5.3 OpenWrt baseline

{{#include requirements/base.md}}

OpenWrt ends support for a release series one year after its first release, or six months after
the next series appears, whichever is later. ENK-BASE-002 and 003 together mean that the upgrade
path must work before 25.12 ends.

## 5.4 Radio regulation

This section states project policy. It is not legal advice.

{{#include requirements/rf.md}}

| Market | State |
|---|---|
| Japan | **Open.** The owner seeks clarification from the authority (whether replacing firmware on equipment bearing the conformity mark is a modification under Radio Act Art. 38-7 para. 3). Until it is documented, ENK-RF-005 holds all images for devices sold in Japan. |

ENK-RF-005 and ENK-SCOPE-003 depend on each other: a non-expert cannot build firmware, so
self-installation needs a published image. For Japan the clarification is on the critical path of
the first release.

## 5.5 Device classes

| Property | `standard` (MT7621 example) | `constrained` (MT7628 example) |
|---|---|---|
| Flash for the image | 14–62 MB | 7.4–7.9 MB |
| RAM | 128–256 MB | 64 MB |
| Platform hardening | Complete | Reduced (no `ujail`, no PIE) |
| Interfaces per radio | 16 (MT7615) | 4 (MT7603), 8 (MT7612) |
| Package architecture | `mipsel_24kc` | `mipsel_24kc` |

## 5.6 Candidate devices

From OpenWrt 25.12.5. None is verified for Enkrnett. Availability and legal status are for the
owner to judge.

| Device | Class | SoC | Radios | Flash for image | RAM | Image for vendor firmware |
|---|---|---|---|---|---:|---|
| ELECOM WRC-1167GST2 | standard | MT7621A | MT7615D | 24,576 KB | 256 MB | Yes |
| ELECOM WRC-1750GST2, WRC-2533GST2 | standard | MT7621 | MT7615 | 24,576 KB | not checked | Yes |
| Buffalo WSR-2533DHPL2 | standard | MT7621AT | 2 × MT7615N | 62,592 KB | 128 MB | Yes |
| I-O DATA WNPR2600G | standard | MT7621 | MT7615 | 13,952 KB | not checked | Yes |
| Buffalo WCR-1166DS | constrained | MT7628AN | MT7628AN, MT7612E | 7,936 KB | 64 MB | Yes |
| ELECOM WRC-1167FS | constrained | MT7628 | not checked | 7,360 KB | not checked | Yes |

Measurement M-3 part 1 showed an Enkrnett-shaped image of 5,696 KB for the two constrained
candidates, against stock 5,888 KB (`docs/src/measurements/m-3-constrained-image.md`).
