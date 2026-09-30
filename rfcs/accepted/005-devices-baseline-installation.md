# RFC 005 — Devices, OpenWrt baseline, and installation

**Status.** Accepted (2026-09-29)
**Tracks.** Which hardware Enkrnett promises, on which OpenWrt release, how it gets onto a
device, how it moves forward, and what radio regulation requires. Startup decisions D-02, D-09,
and D-16.
**Touches.** Device profiles; build definitions; install guides; the download page; the setup
flow (country).

Terms are defined in [RFC 001](./001-scope-people-vocabulary.md).

## Summary

| Subject | Decision |
|---|---|
| Support | Defined per device profile, in three tiers |
| First Tier 1 device | A `standard`-class device. 8 MB / 64 MB devices are built and measured from the first day and promoted when they pass. |
| OpenWrt baseline | 25.12 only |
| Installation | A Tier 1 device can be installed by a non-expert |
| Update and upgrade | Both are obligations. The mechanism is decided in Round 2. |
| Radio regulation | The country is established during setup. No prebuilt image is published for a market until the legal basis is documented. |

## Motivation

| Fact | Source |
|---|---|
| OpenWrt 25.12.5 is the current stable release. 24.10 is at its projected end of life. | [OpenWrt releases][owrt-releases] |
| Automatic Transition Mode exists only from 25.12. 24.10 uses a different Wi-Fi script stack. | OpenWrt [`wifi-scripts`][wifi-schema] |
| OpenWrt calls 8 MB flash "barely enough" and warns that support for such devices "may be dropped in future". | [OpenWrt wiki][owrt-864] |
| Default 25.12.5 images for 8 MB MT7628 devices are 5,824–5,888 KB against limits of 7,360–7,936 KB. | [Download server][dl-mt76x8], [`mt76x8.mk`][mt76x8-mk] |
| OpenWrt builds MT7628 with reduced hardening: no process jails, no position-independent executables. MT7621 keeps them. | [`mt76x8/target.mk`][mt76x8-target], [`mt7621/target.mk`][mt7621-target], [`include/target.mk`][target-mk] |
| MT7621 and MT7628 use the same package architecture, `mipsel_24kc`. | [`profiles.json`][profiles-mt7621] |
| Rust distributes no standard library for MIPS. It must be built with the toolchain, as OpenWrt's feed does. | [Rust platform support][rust-platform] |
| MT7628's radio allows 4 interfaces. MT7615 allows 16. | mt76 source: [`mt7603.h`][mt7603], [`mt7615.h`][mt7615] |
| OpenWrt 25.12.5 publishes images for 80 devices of Japanese vendors. 55 have an image made for installation from the vendor's firmware. | [Download server][dl-root] |
| Whether replacing the firmware of a router certified under Japan's Radio Act keeps it within its certification is not settled by any official source found. | — |

## Design

### 1. Tiers

| Tier | Meaning | Promise to operators |
|---|---|---|
| **1** | Tested on the hardware for every release. Meets every requirement of the baseline RFCs. Can be installed by a non-expert. | Supported |
| **2** | Built for every release and checked automatically against the footprint budgets. Tested on hardware before promotion. | None |
| **3** | Maintained by contributors. | None |

| ID | Requirement | Verification |
|---|---|---|
| ENK-TIER-001 | Every supported device shall have a device profile that names the model, the hardware revision, the tier, the device class, and the install class. | Inspection |
| ENK-TIER-002 | A device profile shall declare: the uplink port; the reset button; the light that Enkrnett can control, if any; the radios with their bands and interface limits; and whether the platform's hardening is complete on its target. | Inspection; test on the device |
| ENK-TIER-003 | The management UI shall offer only what the device profile and the run-time checks of the device allow. | Test on each profile |
| ENK-TIER-004 | A device shall enter Tier 1 only when section 2 is met and the owner has accepted it. | Record in an RFC |
| ENK-TIER-005 | A device on a target with reduced platform hardening shall enter Tier 1 only by a separate owner decision that names this reduction. | Record in an RFC |

### 2. Criteria for Tier 1

| # | Criterion | Verified by |
|---|---|---|
| 1 | The OpenWrt baseline publishes an image for the device, including one that the vendor's firmware accepts. | Download server |
| 2 | Non-experts installed it with the guide alone. | RFC 006, M-7 |
| 3 | Non-experts recovered from an interrupted installation with the guide alone, or the device keeps its previous firmware when an installation fails. | RFC 006, M-7 |
| 4 | OWE with PMF works in access-point mode with the client matrix. | RFC 006, M-4 |
| 5 | The radio that carries the management network allows at least 4 interfaces. | Driver source; M-4 |
| 6 | The device has a reset button that OpenWrt reads. | Test |
| 7 | The footprint budgets are met. | RFC 006, M-2, M-3 |
| 8 | The device can be bought in the target market when the release is published. | Owner |
| 9 | The legal basis for operating the device with Enkrnett in the target market is documented. | Section 6 |

### 3. Install classes

| Class | Meaning |
|---|---|
| **Self-install** | The operator installs through the vendor's own firmware update screen, with one file, following the guide. |
| **Expert install** | Anything else, including installation as a package on a device that already runs OpenWrt. |

| ID | Requirement | Verification |
|---|---|---|
| ENK-INST-001 | For every Tier 1 device the project shall provide one installation file and one guide in every language of the management UI. | Inspection |
| ENK-INST-002 | The guide shall make the operator confirm the model and the hardware revision on the device's label before anything is changed. | Usability test |
| ENK-INST-003 | The guide shall state which versions of the vendor's firmware the installation was tested from. | Inspection |
| ENK-INST-004 | The guide shall describe recovery from an interrupted installation. | Usability test |
| ENK-INST-005 | Enkrnett shall also be installable as a package on a device that runs the OpenWrt baseline. This is an expert install. The device then reports which hardening requirements are not met (ENK-HARD-006). | Test |

### 4. OpenWrt baseline, update, and upgrade

| ID | Requirement | Verification |
|---|---|---|
| ENK-BASE-001 | An Enkrnett release shall be built on exactly one OpenWrt baseline. The first is 25.12. | Inspection |
| ENK-BASE-002 | Enkrnett shall publish releases for a baseline only while OpenWrt maintains that baseline. | Release checklist |
| ENK-BASE-003 | Before the first release, the project shall define and test how an installed device is updated, and how it is upgraded to the next baseline. | Decision D-11; test |
| ENK-BASE-004 | Update and upgrade shall keep the owner, the management key, and the Wi-Fi configuration. | Test |
| ENK-BASE-005 | Everything Enkrnett stores shall carry a format version, so that a newer release can read what an older one wrote. | Inspection; test with stored data of every earlier release |
| ENK-BASE-006 | No release shall leave an installed Tier 1 device without a path to the release that follows it. | Release checklist |
| ENK-BASE-007 | When a new baseline no longer builds a Tier 1 device, operators of that device shall be told in the management UI at least one release before support ends. | Release checklist |

OpenWrt ends support for a release series one year after its first release, or six months after
the next series appears, whichever is later ([policy][owrt-security]). ENK-BASE-002 and
ENK-BASE-003 together mean that the upgrade path must work before 25.12 ends.

### 5. Device classes and what they cost

| Property | `standard` (MT7621 example) | `constrained` (MT7628 example) |
|---|---|---|
| Flash for the image | 14–62 MB | 7.4–7.9 MB |
| RAM | 128–256 MB | 64 MB |
| Platform hardening | Complete | Reduced |
| Interfaces per radio | 16 (MT7615) | 4 (MT7603), 8 (MT7612) |
| Package architecture | `mipsel_24kc` | `mipsel_24kc` |
| Toolchain | Same | Same |

Both classes share one toolchain and one package architecture. Work on the `standard` device is
not lost when the `constrained` device follows.

### 6. Radio regulation

This section states project policy. It is not legal advice.

| ID | Requirement | Verification |
|---|---|---|
| ENK-RF-001 | Setup shall establish the country before the management or guest network starts. | Test |
| ENK-RF-002 | Where a device profile belongs to one market, the profile shall fix the country and the operator shall not be asked. | Test |
| ENK-RF-003 | Before the country is established the device shall transmit only on channels and at power levels that the platform's regulatory database allows in every country. | Test |
| ENK-RF-004 | Enkrnett shall offer no setting for transmit power, and no setting that selects a channel or a behavior outside what the platform's regulatory database allows for the country. | Inspection |
| ENK-RF-005 | The project shall publish no prebuilt image for a device of a market until the legal basis for operating that device with replaced firmware in that market is documented in `docs/src/`. | Release checklist |

| Market | State |
|---|---|
| Japan | **Open.** The owner seeks clarification from the authority. Until it is documented, ENK-RF-005 holds all images for devices sold in Japan. |

**ENK-RF-005 and ENK-SCOPE-003 depend on each other.** A non-expert cannot build firmware, so
self-installation needs a published image. For Japan, the clarification is therefore on the
critical path of the first release.

### 7. Candidate devices

From OpenWrt 25.12.5. **None is verified for Enkrnett.** Availability and legal status are for
the owner to judge.

| Device | Class | SoC | Radios | Flash for image | RAM | Image for vendor firmware |
|---|---|---|---|---:|---:|---|
| ELECOM WRC-1167GST2 | standard | MT7621A | MT7615D | 24,576 KB | 256 MB | Yes |
| ELECOM WRC-1750GST2, WRC-2533GST2 | standard | MT7621 | MT7615 | 24,576 KB | not checked | Yes |
| Buffalo WSR-2533DHPL2 | standard | MT7621AT | 2 × MT7615N | 62,592 KB | 128 MB | Yes |
| I-O DATA WNPR2600G | standard | MT7621 | MT7615 | 13,952 KB | not checked | Yes |
| Buffalo WCR-1166DS | constrained | MT7628AN | MT7628AN, MT7612E | 7,936 KB | 64 MB | Yes |
| ELECOM WRC-1167FS | constrained | MT7628 | not checked | 7,360 KB | not checked | Yes |

Sources: image limits from [`mt7621.mk`][mt7621-mk] and [`mt76x8.mk`][mt76x8-mk]; images from the
[download server][dl-root]; RAM and radios from the OpenWrt hardware table.

## Consequences

| Affected | Change |
|---|---|
| Draft "constrained profile" and "standard profile" | Become device classes. Support is expressed by tiers. |
| Draft requirement "8 MB / 64 MB class" as a compatibility target | Becomes Tier 2 until section 2 is met and ENK-TIER-005 is decided. |
| Draft acceptance "a low-resource profile can be built" | Replaced by criterion 7: budgets are measured, not only built. |
| Draft "cryptographic backend" as an open issue | Closed. The baseline's default is used. |
| Draft "minimum supported OpenWrt release" as an open issue | Closed by ENK-BASE-001. |
| Draft future item "automatic update system" | Update and upgrade become obligations (ENK-BASE-003). The mechanism is decided in Round 2. |
| Draft open issue "licence" | Closed by the project rules: Apache-2.0. |

## Alternatives considered

| Alternative | Why not |
|---|---|
| 8 MB / 64 MB device as the first Tier 1 device | Least headroom, reduced hardening, unmeasured memory. It would put the schedule at risk. |
| ARM devices only | Gives up the least expensive hardware, which is a project goal. |
| Support 24.10 and 25.12 | Two Wi-Fi script stacks for a release series that is ending. |
| Package delivery only | A non-expert cannot install OpenWrt first. |
| Publish images with a disclaimer | The project would recommend something whose lawfulness it has not established. |

## Verification

| Item | Where |
|---|---|
| Size and memory of the daemon on `mipsel_24kc` | RFC 006, M-2 |
| Image size and free memory on a constrained device | RFC 006, M-3 |
| OWE, Transition Mode, interface count, and PMF on the candidates | RFC 006, M-4 |
| Installation and recovery by non-experts | RFC 006, M-7 |

## Open questions

| # | Question | For |
|---|---|---|
| 1 | Which candidate devices can be obtained for measurement? | Owner |
| 2 | The inquiry for section 6. | Owner |
| 3 | Mechanism of update and upgrade. | Round 2, decision D-11 |

## References

[owrt-releases]: https://openwrt.org/releases/start
[owrt-864]: https://openwrt.org/supported_devices/864_warning
[owrt-security]: https://openwrt.org/docs/guide-developer/security
[wifi-schema]: https://github.com/openwrt/openwrt/blob/openwrt-25.12/package/network/config/wifi-scripts/files-ucode/usr/share/schema/wireless.wifi-iface.json
[dl-root]: https://downloads.openwrt.org/releases/25.12.5/targets/
[dl-mt76x8]: https://downloads.openwrt.org/releases/25.12.5/targets/ramips/mt76x8/
[profiles-mt7621]: https://downloads.openwrt.org/releases/25.12.5/targets/ramips/mt7621/profiles.json
[mt76x8-mk]: https://github.com/openwrt/openwrt/blob/openwrt-25.12/target/linux/ramips/image/mt76x8.mk
[mt7621-mk]: https://github.com/openwrt/openwrt/blob/openwrt-25.12/target/linux/ramips/image/mt7621.mk
[mt76x8-target]: https://github.com/openwrt/openwrt/blob/openwrt-25.12/target/linux/ramips/mt76x8/target.mk
[mt7621-target]: https://github.com/openwrt/openwrt/blob/openwrt-25.12/target/linux/ramips/mt7621/target.mk
[target-mk]: https://github.com/openwrt/openwrt/blob/openwrt-25.12/include/target.mk
[mt7603]: https://github.com/openwrt/mt76/blob/master/mt7603/mt7603.h
[mt7615]: https://github.com/openwrt/mt76/blob/master/mt7615/mt7615.h
[rust-platform]: https://doc.rust-lang.org/stable/rustc/platform-support.html

- OpenWrt releases and support policy: [releases][owrt-releases], [policy][owrt-security]
- OpenWrt, warning about 8 MB / 64 MB devices: [wiki][owrt-864]
- OpenWrt 25.12.5 images: [download server][dl-root]
- Rust platform support: [documentation][rust-platform]
