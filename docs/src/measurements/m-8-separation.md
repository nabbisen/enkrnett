# M-8 — Separation (part 1: on the virtual machine)

| | |
|---|---|
| **Measurement** | M-8, part 1 of 2 |
| **RFC** | 006 Feasibility measurements (`rfcs/accepted/006-feasibility-measurements.md`) |
| **Handoff** | `rfcs/handoffs/006-feasibility-measurements/README.md` |
| **Carried out by** | Architect (acting for the development team, by the owner's instruction) |
| **Date** | 2026-10-29 |
| **State** | Ready for review (part 1); part 2 (real radios, real site network) waits for a router |

## Question

Does the separation of RFC 002 hold against a client that tries to break it?

## Answer in one paragraph

Built entirely from OpenWrt's own means (UCI: two bridges, three firewall zones, two DNS/DHCP
instances, three BSSes on one radio), the topology of RFC 002 held in 13 of the 14 attempts
that the virtual machine can express: guests reach the Internet and nothing else, the device
answers guests only on DNS and DHCP, the management name does not resolve for guests, routed
guest-to-guest traffic is refused, IPv6 is dead on both networks, the management client reaches
the UI and the Internet but neither the site network nor a guest. **One attempt failed: a guest
on the OWE BSS could ping a guest on the open BSS of Transition Mode (row 6).** The cause is
that OpenWrt 25.12.5 sets the bridge port isolation flag (`bridge_isolate`) only on the first
BSS of the radio; the second and third BSSes stay unisolated whether Transition Mode is
automatic or written as two sections, and a `network` device section with `isolate '1'` does not
reach them either. Setting the kernel flag directly (`/sys/class/net/br-guest/brif/<port>/isolated`)
isolates both directions and leaves Internet access intact. Enkrnett must therefore set and verify
that flag itself after every Wi-Fi reload.

## Environment

| Item | Version |
|---|---|
| OpenWrt | 25.12.5 x86-64 (the M-6 VM), `mac80211_hwsim radios=5` |
| firewall4 | 2025.03.17~b6e51575-r2; nftables-json 1.1.6-r2 |
| dnsmasq | 2.93-r1, two instances |
| wpad-basic-mbedtls | 2025.08.26~ca266cc2-r2 (access point and the four stations) |
| netifd / wifi-scripts | 2026.02.26~cbb83a18-r1 / 1.0-r1 |
| Tools | nmap 7.95-r1 in the station namespaces; busybox `arping`, `ping`, `nslookup` |

## Setup

| Element | How |
|---|---|
| Uplink | `eth1`, zone `wan`, QEMU user-mode network 10.0.2.0/24: 10.0.2.2 plays the site router, 10.0.2.3 (QEMU's DNS) plays a site host; outbound TCP reaches the Internet |
| Other Ethernet port | `eth0` left without a network (`lan` proto `none`) |
| Guest network | bridge `br-guest` 10.42.1.1/24, zone `guest`; BSS `enkrnett-guest` with OWE + automatic Transition Mode, `isolate 1`, `bridge_isolate 1` |
| Management network | bridge `br-mgmt` 10.42.2.1/24, zone `mgmt`; BSS `enkrnett-mgmt`, WPA2/WPA3-Personal (`sae-mixed`), `isolate 1`, `bridge_isolate 1` |
| Management UI stand-in | `uhttpd` bound to 10.42.2.1:80 only |
| Firewall | zones `guest` and `mgmt`: input REJECT, forward REJECT; forwarding to `wan`; a rule before it rejecting destinations in 10/8, 172.16/12, 192.168/16, 169.254/16, 100.64/10; input rules for DHCP (udp 67) and DNS (53), plus tcp 80 for `mgmt`; IPv6 dropped on input and forward for both; `wan` input REJECT (plus the measurement-only SSH rule) |
| DNS/DHCP | two `dnsmasq` instances, one per network; the management instance alone carries `address=/enkrnett.home.arpa/10.42.2.1`; the guest instance is kept off loopback (`notinterface loopback`), otherwise the second instance fails to start on 127.0.0.1:53 |
| Clients | `g1` OWE, `g3` OWE (same hidden BSS), `g2` open BSS, `m1` WPA2-PSK, each a `wpa_supplicant` in its own network namespace on its own simulated radio |

Complete configuration files: `m8.sh` (outside the repository).

## Results

| # | From | Attempt | Required | Observed | Holds |
|---|---|---|---|---|---|
| 1 | guest | site host 10.0.2.3:53; ping 10.0.2.2 | fails | tcp closed (reject); 0 ping replies | yes |
| 2 | guest | site router 10.0.2.2:80 | fails | closed. Reference: 1.1.1.1:80 and 1.0.0.1:443 open | yes |
| 3 | guest | scan the device 10.42.1.1 (tcp 1–1024, 8080, 8443; udp 53, 67) | only DNS answers | tcp 53 open; udp 53 and 67 open; nothing else | yes (DHCP is required; the handoff's wording omitted it) |
| 4 | guest | DNS query for the management name | no address | NXDOMAIN; from `m1`: 10.42.2.1 | yes |
| 5 | guest1 | guest3, same OWE BSS | fails | 0 replies | yes (hostapd `ap_isolate`) |
| **6** | **guest1 (OWE BSS)** | **guest2 (open BSS)** | **fails** | **2 of 2 ping replies** | **no** |
| 7 | guest | guest on another radio | fails | not applicable: one simulated radio | — (part 2) |
| 8 | guest1 | packet to the device's MAC with guest3's IP | not delivered | 0 replies | yes (zone forward REJECT) |
| 9 | guest1 | guest3's IPv6 link-local; all-nodes | fails | 0 replies to guest3; all-nodes answered only by the sender's own stack | yes |
| 10 | guest1 | ARP who-has guest3 (broadcast) | reaches the other guest (stated limit) | 0 replies | better than the limit on simulated radios; the real-radio group-key path is part 2 |
| 11 | management | UI 10.42.2.1:80 | works | open; HTML returned | yes |
| 12 | management | Internet 1.1.1.1:80 | works | open | yes |
| 13 | management | site host 10.0.2.3:53; ping 10.0.2.2 | fails | closed; 0 replies | yes |
| 14 | management | guest1 | fails | 0 replies; tcp 22 closed | yes |
| 15 | site network | scan the uplink address | nothing answers | by inspection: `input_wan` = measurement SSH rule, then `reject_from_wan` | yes (inspection) |
| — | guest | management address 10.42.2.1:80 | fails | closed | yes |

### Row 6: diagnosis and remedy

| Step | Observation |
|---|---|
| Bridge port flags after the automatic Transition Mode (`/sys/class/net/br-*/brif/*/isolated`) | management BSS `phy0-ap0`: isolated = 1; guest BSSes: 0 |
| Explicit two-section Transition Mode (`owe_transition_ifname`, both sections with `isolate 1` and `bridge_isolate 1`) | both guest ports still 0; `network.wireless status` shows `bridge_isolate: true` for all three |
| `network` device sections `guest-owe`/`guest-open` with `isolate '1'`, network reload, wifi reload, wifi down/up | still 0 |
| Flag written by hand: `echo 1 > …/brif/guest-owe/isolated` and the same for `guest-open` | guest1 ↔ guest2: 0 replies in both directions; guest1 → Internet still open |

Only the first BSS created on the radio received the flag. hostapd's `ap_isolate=1` is present in
all three BSS blocks of the generated configuration, so isolation *within* a BSS works everywhere;
isolation *between* BSSes of one bridge depends on the kernel flag, which the platform sets once.

## Measured values

| Quantity | Value | Budget |
|---|---|---|
| Attempts that hold on the VM | 13 of 14 applicable | all |
| Services visible to a guest on the device | DNS (tcp/udp 53), DHCP (udp 67) | RFC 002 §3: DHCP, DNS |

## Deviations from the procedure

| Deviation | Why |
|---|---|
| Simulated radios and a QEMU user-mode network instead of a device and a site network | No router at hand; the topology, the firewall, and the DNS/DHCP split are platform behaviour and transfer to the device; radio-layer effects (row 7, row 10, driver support of the bridge flag) are part 2. |
| Row 15 by inspection | The host cannot scan the guest's uplink address through QEMU's user-mode network. |
| Three runs | Run 1: `uci` section names collided (`guest` used for both a `dnsmasq` and a `dhcp` section); run 2: the second `dnsmasq` instance failed on loopback; run 3 is the reported one. |

## What contradicts an RFC

| RFC | Requirement | Observation |
|---|---|---|
| 002 | ENK-ISO-002 (no forwarding between guests, between BSSes) | Not met by OpenWrt 25.12.5's `bridge_isolate` on the second and third BSS. Met once the kernel flag is set directly. **Enkrnett must set and verify the `isolated` flag of every guest and management bridge port after every Wi-Fi reload** (candidate requirement for RFC 011's health check, ENK-CFG-007). |
| 002 | ENK-ISO-006 (separate bridges) | Confirmed necessary: the firewall never sees bridged frames; only the port flag or separate bridges separate guests. |

## Observations beyond the question

1. **Two `dnsmasq` instances need `notinterface loopback` on all but one**, or the second never starts and its network silently has no DHCP. The failure is only in `logread`.
2. **UCI section names are one namespace per file**: a `dnsmasq` section and a `dhcp` section may not share the name `guest`. The error is a parse error of the whole file; every section in it disappears.
3. `uhttpd` binds to all addresses by default; the single `listen_http` line is what keeps the UI off the guest network. The daemon must bind explicitly, and ENK-ISO-003's port scan is the test that catches a regression.
4. The firewall's zone `forward REJECT` already refuses routed guest-to-guest traffic (row 8); the bridge flag is needed only for traffic that never reaches the IP layer.
5. The whole topology is 180 lines of UCI; nothing outside OpenWrt's configuration was needed except the one sysfs flag.

## How to repeat

```sh
# host, .git-exclude/tmp/m/vm: VM with mac80211_hwsim radios=5
./vmssh 'sh /root/m8.sh' > m8.out      # topology, stations, 15 rows
./vmssh 'sh /root/m8b.sh' > m8b.out    # row 6 diagnosis and the two-section form
# m8c.out / m8d.out: flags read from sysfs, flag set by hand, network device sections
```
