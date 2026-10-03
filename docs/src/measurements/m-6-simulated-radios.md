# M-6 — Simulated radios in a virtual machine

| | |
|---|---|
| **Measurement** | M-6 |
| **RFC** | 006 Feasibility measurements (`rfcs/accepted/006-feasibility-measurements.md`) |
| **Handoff** | `rfcs/handoffs/006-feasibility-measurements/README.md` |
| **Carried out by** | Architect (acting for the development team, by the owner's instruction) |
| **Date** | 2026-10-04 |
| **State** | Ready for review |

## Question

Does OpenWrt in a virtual machine with simulated radios run OWE from association to traffic?

## Answer in one paragraph

Yes. On OpenWrt 25.12.5 x86-64 under QEMU/KVM with `kmod-mac80211-hwsim` (three radios) an
access point configured through UCI with `encryption 'owe'` accepts a station with OWE and PMF,
the station obtains an address and exchanges traffic, and `ubus` reports the station as
authorized with `mfp: true`. Automatic Transition Mode (`owe_transition '1'`) starts both BSSes,
but with the stock 25.12.5 scripts hostapd receives the **same BSSID for both BSSes**; a legacy
station connects to the open BSS, while an OWE station is rejected by the OWE BSS ("tried to
associate with unknown SSID"). With the upstream fix (OpenWrt commit `1789118599`, not in 25.12.5)
applied to `/usr/share/ucode/wifi/ap.uc`, the two BSSes get distinct BSSIDs and an OWE station
with a clean scan cache joins the hidden OWE BSS with PMF. Two further things were needed to get
this far: the Wi-Fi packages must be installed **before** netifd starts (a reboot after
installation), and every radio's `country` must be set, because the scripts take the country from
the last radio in the list. The whole run is scripted and completes in about 2.5 minutes.

## Environment

| Item | Version or model |
|---|---|
| OpenWrt | 25.12.5 (r33051-f5dae5ece4), `openwrt-25.12.5-x86-64-generic-ext4-combined.img.gz`, sha256 verified |
| Kernel | 6.12.94 |
| Virtual machine | QEMU 11.1.1 with KVM, 2 vCPU, 256 MB, two user-mode NICs (eth0 LAN unused, eth1 WAN with Internet) |
| Host | Linux 7.2.8 (CachyOS), x86-64 |
| Packages added | `kmod-mac80211-hwsim 6.12.94.6.18.26-r1`, `wpad-basic-mbedtls 2025.08.26~ca266cc2-r2`, `hostapd-utils`, `wpa-cli`, `iw 6.17-r1`, `ip-full 6.18.0-r2`, `tcpdump-mini`, `rpcd 2026.06.04~28faf640-r1`, `patch` |
| Already in the image | `wifi-scripts 1.0-r1`, `netifd 2026.02.26~cbb83a18-r1`, `hostapd-common`, `ucode 2026.01.16~85922056-r1`, `wireless-regdb 2026.05.30-r1`, LuCI and `rpcd` (the official x86-64 image includes them) |
| Station software | `wpa_supplicant` from the same `wpad-basic-mbedtls` (v2.12-hostap_2_12), one per network namespace |

## Setup

1. The image was decompressed, resized to 512 MB, and booted with a serial console on a unix
   socket and host port 2222 forwarded to the guest's SSH port on the WAN side (a firewall rule
   accepting TCP 22 from `wan` was added for the measurement only).
2. Packages were installed with `apk`; `/etc/modules.d/mac80211-hwsim` was set to
   `mac80211_hwsim radios=3`; the VM was rebooted.
3. `m6.sh` (kept outside the repository) runs the procedure: it generates `/etc/config/wireless`
   with `wifi config`, configures `radio0` as the access point and moves `phy1` and `phy2` into
   network namespaces `sta1` and `sta2`, where `wpa_supplicant` runs as station.

## Results

| Step | Result | Evidence |
|---|---|---|
| 1 Install | All packages install from the default feeds, including the kmod feed. | `apk list --installed` recorded in the run log |
| 2 Radios | Three radios: `phy0`…`phy2` → `hwsim0`…`hwsim2`. `ubus list network.wireless` exists only after a reboot following the installation. | Run 1 failed at this point: `wifi up` answered "Command failed: Not found" |
| 3 OWE access point | Generated hostapd config: `wpa_key_mgmt=OWE`, `ieee80211w=2`, `owe_groups=19 20 21`, `owe_ptk_workaround=1`, `wpa=2`. Interface `phy0-ap0` up on channel 6. | Run 2 and run 3 logs |
| 3 Country | With `radio0.country='JP'` alone, hostapd received `country_code=00` and refused to start ("Invalid country_code '00'"). `wifi-scripts/hostapd.uc` `device_country_code()` loops over all radios and keeps the **last** value; `wifi config` had created `radio1`/`radio2` with `country '00'`. Setting all radios to `JP` fixed it. | Run 1 log; `hostapd.uc` lines 70–84 |
| 4 OWE station | `wpa_state=COMPLETED`, `key_mgmt=OWE`, `pmf=2`, `pairwise_cipher=CCMP`, address by DHCP, ping to the gateway 0 % loss. | Run 3 step 4 |
| 5 ubus | `hostapd.phy0-ap0 get_clients`: station `auth`, `assoc`, `authorized: true`, `mfp: true`; `get_status` answers; `network.wireless status` lists the radio, the interface, and its stations. | Run 3 step 5 |
| 6–7 Transition Mode, stock scripts | Two BSSes start: `phy0-ap0` hidden `enkrnett-m6OWE` (OWE) and `phy0-ap1` `enkrnett-m6` (open). The generated config carries `bssid=02:00:00:00:00:00` for **both**; the kernel gave `phy0-ap1` `42:00:00:00:00:00` anyway. | Run 2 step 6–7 |
| 8 Stations, stock scripts | Legacy station (no OWE) joins `42:…` and gets traffic. OWE station is rejected: `phy0-ap0: STA … tried to associate with unknown SSID 'enkrnett-m6'`; after several attempts `wpa_supplicant` falls back to the open BSS. | Run 2 step 8 |
| 6–7 Transition Mode, with the upstream fix | `patch -p10` of commit `1789118599` applies to the 25.12.5 `ap.uc` (one hunk offset by 17 lines). The generated config now has `bssid=02:00:00:00:00:00` for the OWE BSS and `bssid=42:00:00:00:00:00` for the open BSS. | Run 3 step 6–7 |
| 8 Stations, with the fix | OWE station still fails right after the switch; its scan cache held the pure-OWE entry (same BSSID, old SSID). After `wpa_cli bss_flush` it joined the hidden OWE BSS within 1 s: `bssid=02:…`, `key_mgmt=OWE`, `pmf=2`, `authorized: true`, `mfp: true`. Legacy station on `42:…`, `authorized: true`, `mfp: false`. A brand-new OWE station started during the run ended on the open BSS in state `ASSOCIATED` after bouncing between the two BSSes for 30 s. | Run 3 step 8; addendum 8b, 8c |
| 9 Unattended | Steps 2–8 run from one script without manual action; wall time about 2.5 min per run. The VM boots in about 15 s. | Timing of runs 2 and 3 |

## Measured values

| Quantity | Value | Unit | Provisional budget |
|---|---:|---|---:|
| Boot to shell prompt (KVM) | ≈ 15 | s | — |
| Script run, steps 2–8 | ≈ 150 | s | — |
| OWE association to `COMPLETED` after `bss_flush` | 1 | s | — |

## Deviations from the procedure

| Deviation | Why |
|---|---|
| Three runs instead of one | Run 1 was invalid (netifd started before the Wi-Fi packages existed; interface generated with `disabled='1'`). Run 2 is the stock-script result. Run 3 is with the upstream fix. |
| An addendum (`m6b.sh`) after run 3 | To separate the scan-cache effect from the BSSID defect. |
| Station interfaces were created with `iw phy … interface add` | OpenWrt removes the default `wlanN` interfaces of unmanaged phys. |
| The stations received addresses from QEMU's user-mode DHCP (`192.168.100.0/24`) rather than from `dnsmasq` | `eth0` is bridged into `br-lan`; both servers answer, QEMU's first. Irrelevant to the question. |

## What contradicts an RFC

| RFC | Requirement | Observation |
|---|---|---|
| 005 | ENK-BASE-001 (baseline 25.12 only) | Automatic Transition Mode is unusable for OWE clients on 25.12.5 without the upstream fix. Enkrnett's build must carry commit `1789118599` until a 25.12.x release includes it, or generate the two-section form itself. |
| — | A-10 in the ledger (automatic Transition Mode works on 25.12.x) | Refuted for the stock scripts; confirmed with the fix. |

## Observations beyond the question

1. **Country code from the last radio.** `device_country_code()` in `hostapd.uc` takes the country of whichever radio comes last in `network.wireless status`. Enkrnett must set `country` on every `wifi-device`, including unused radios, or hostapd fails on all of them.
2. **Switching from OWE-only to Transition Mode keeps the OWE BSS's BSSID and changes its SSID.** Clients that cached the old entry keep trying the old SSID until the entry expires (`wpa_supplicant` default 180 s). Expect a pause after such a switch; it is a client-side effect, not an AP defect.
3. **Refusal looks like absence.** `ubus` answers "Not found" for objects that exist but are not visible to the caller; the same text appears when a package is missing. Diagnostics must not confuse the two.
4. **The official x86-64 image already contains `rpcd`, `uhttpd`, and LuCI.** A LuCI-less Enkrnett image will not, so `rpcd` must be added explicitly if it is used (RFC 006 M-5).
5. `hostapd` shares one process for all phys; the station side of `wpad-basic-mbedtls` is usable as a test client in a namespace without extra packages.

## How to repeat

```sh
# host, from .git-exclude/tmp/m/vm (kept outside the repository)
./vm.sh start && python3 vmsh.py wait 90
# first time only: install packages, set radios=3, reboot (see vm.sh header and m6.sh)
./vmssh 'sh /root/m6.sh' > m6-runN.out      # stock scripts
./vmssh 'apk add patch; cd /usr/share/ucode/wifi && patch -p10 < /root/fix.patch'  # upstream fix
./vmssh 'reboot'; sleep 40; ./vmssh 'sh /root/m6.sh; sh /root/m6b.sh'
```

The scripts (`vm.sh`, `vmsh.py`, `m6.sh`, `m6b.sh`) and the three run logs are kept under
`.git-exclude/tmp/m/vm/` on the workstation. The fix is
<https://github.com/openwrt/openwrt/commit/1789118599>.
