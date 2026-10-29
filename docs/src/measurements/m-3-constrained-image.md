# M-3 — Complete image on a `constrained` device (part 1: without a device)

| | |
|---|---|
| **Measurement** | M-3, part 1 of 2 |
| **RFC** | 006 Feasibility measurements (`rfcs/accepted/006-feasibility-measurements.md`) |
| **Handoff** | `rfcs/handoffs/006-feasibility-measurements/README.md` |
| **Carried out by** | Architect (acting for the development team, by the owner's instruction) |
| **Date** | 2026-10-29 |
| **State** | Ready for review (part 1); part 2 waits for a `constrained` router |

## Question

Does a complete image fit and run on a `constrained` device?

## Answer in one paragraph

**Fit: yes, with room.** An image for the Buffalo WCR-1166DS and the ELECOM WRC-1167FS built
from the OpenWrt 25.12.5 Image Builder with the default package set minus PPP and IPv6 clients,
plus placeholders for the Enkrnett daemon (the measured 725 KB lean binary) and a 150 KiB UI
bundle, is **5,696 KB** — 192 KB smaller than the stock image with LuCI — leaving **2,240 KB**
headroom to the WCR-1166DS's 7,936 KB limit and **1,664 KB** to the WRC-1167FS's 7,360 KB limit.
**Run: indicative only.** An emulated MIPS 24Kc OpenWrt 25.12.5 with 64 MB reports 12.7 MB
available after boot while its root filesystem occupies 14.9 MB of RAM (initramfs); on a real
device the root filesystem is in flash, so about 27 MB would be available before clients. The
lean daemon adds about 0.6 MB. The remaining steps (overlay after first boot, memory with clients,
throughput, 24 hours) need the router; they are part 2.

## Environment

| Item | Version |
|---|---|
| Image Builder | `openwrt-imagebuilder-25.12.5-ramips-mt76x8.Linux-x86_64.tar.zst`, sha256 verified |
| Profiles | `buffalo_wcr-1166ds`, `elecom_wrc-1167fs` |
| Packages removed from the default set | `ppp`, `ppp-mod-pppoe`, `odhcp6c`, `odhcpd-ipv6only` (`kmod-nft-offload` was requested removed but stays as a dependency) |
| Files added | `/usr/sbin/enkrnettd` = the M-2 lean binary (724,988 B raw); `/www/ui-placeholder.bin` = 153,600 B of incompressible data; `/usr/share/acl.d/enkrnett.json` |
| Emulation for the memory indication | `qemu-system-mipsel -M malta -cpu 24Kc -m 64`, `openwrt-25.12.5-malta-le-vmlinux-initramfs.elf` |

## Results

### Step 1: image size

| Device | Image limit | Stock 25.12.5 image (with LuCI) | This image | Headroom | Packages |
|---|---:|---:|---:|---:|---:|
| Buffalo WCR-1166DS | 7,936 KB | 5,888.3 KB | **5,696.3 KB** | 2,239.7 KB | 100 |
| ELECOM WRC-1167FS | 7,360 KB | 5,888.3 KB | **5,696.3 KB** | 1,663.7 KB | 100 |

Root filesystem (squashfs, xz, 1 MiB blocks as the `small_flash` target uses): 3,580.9 KB for
9,705.7 KB of content (36.9 %). The kernel and headers make up the rest. The placeholders cost
about 300 KiB of the 5,696 KB; the removed packages saved about 500 KB against stock.

Package set kept (relevant part): `wpad-basic-mbedtls`, `hostapd-common`, `dnsmasq`,
`firewall4`, `nftables-json`, `kmod-mt7603`, `kmod-mt76x2` (second radio of the WCR-1166DS),
`dropbear`, `libmbedtls21`, `ca-bundle`. Dropbear stays only until RFC 002 ENK-HARD-002 is
implemented; it costs about 100 KB.

### Steps 2–5: run-time (indication from emulation; the device measurement is part 2)

| Quantity | Emulated 64 MB (`malta/le`, root in RAM) | Expected on a device (root in flash) |
|---|---:|---:|
| MemTotal seen by the kernel | 53,144 KiB | about 60,000 KiB |
| MemAvailable after boot, no clients | 12,744 KiB | about 27,600 KiB (the 14,904 KiB `Shmem` of the initramfs root is not present) |
| Lean daemon started | −616 KiB available | same |
| Processes after boot | 66 | similar |

The emulated kernel is the `malta` build, not `mt76x8`; drivers, firmware blobs (`mt7603`,
`mt7615`), and the Wi-Fi stack with clients will take more on the device. Budget ENK-FOOT-009
(≥ 12 MB available with 20 clients) remains open until part 2.

## Measured values

| Quantity | Value | Unit | Budget (RFC 010) |
|---|---:|---|---:|
| Image, WCR-1166DS | 5,696 | KB | ≤ stock 5,888 (ENK-FOOT-004) |
| Image, WRC-1167FS | 5,696 | KB | ≤ stock 5,888 |
| Headroom to the smaller limit | 1,664 | KB | — |
| Available memory, 64 MB, emulated, root in RAM | 12.7 | MB | ≥ 12 with 20 clients (ENK-FOOT-009), device figure pending |

## Deviations from the procedure

| Deviation | Why |
|---|---|
| No device: steps 2–5 replaced by an emulated indication | No `constrained` router at hand yet; the owner is obtaining one. |
| Placeholder UI | The real UI does not exist yet; 150 KiB is the budget of ENK-FOOT-002. |
| Dropbear included | The hardening of RFC 002 is not implemented; it is part of the stock set. |

## What contradicts an RFC

None. ENK-FOOT-004 is met with the placeholders at budget size.

## Observations beyond the question

1. The Image Builder honours `FILES=` and the package removals without any patching; a profile for each device is a one-line difference.
2. `kmod-nft-offload` cannot be removed by request on this target; it is pulled in by the default set. 20 KB.
3. The `small_flash` target compresses the root filesystem with 1 MiB blocks (not 256 KiB); M-2's 256 KiB figures slightly overstate the daemon's cost on this class.
4. The ELECOM WRC-1167FS's 7,360 KB limit is the tighter of the two candidates; the same image fits both.

## How to repeat

```sh
cd .git-exclude/tmp/m/m3/openwrt-imagebuilder-25.12.5-ramips-mt76x8.Linux-x86_64
make image PROFILE=buffalo_wcr-1166ds PACKAGES="-ppp -ppp-mod-pppoe -odhcp6c -odhcpd-ipv6only" FILES=../files BIN_DIR=../out-wcr
# output lands under target/linux/ramips/out-wcr/ of the Image Builder tree
cd ../../malta && sed -i 's/-m 128/-m 64/' mvm.sh && ./mvm.sh start   # memory indication
```
