# OpenWrt platform (milestone M1, stage 4)

How the daemon drives OpenWrt: chapter 6 (settings and changes), chapter 2 (the networks), RFC 014
(what the daemon must do itself). This stage brings the claim end to end on the x86-64 virtual
machine with real `netifd`, `hostapd`, `dnsmasq`, and `firewall4`; the transaction engine of
chapter 6 §4 follows in M2.

## 1. Measurement M-5b (2026-11-26, x86-64 VM, OpenWrt 25.12.5)

Question (RFC 011 §7): can the daemon's own user, not `root`, commit the owned configuration
without `rpcd` and without a root helper?

| Step | Result |
|---|---|
| `/etc/config/{wireless,network,dhcp,firewall}` owned by `enkrnett`, mode 0600; `/etc/config` group `enkrnett`, mode `1775` (sticky) | The user replaces its own files by temp-file-and-rename |
| Rename over `/etc/config/system` (root's) | `Operation not permitted` |
| Delete `/etc/config/system` | `Operation not permitted` |
| Read `/etc/config/system` | `Permission denied` |
| Create a new file in `/etc/config` | **Allowed** by the directory. Noted: the user could add a configuration file for a service that has none. Not a path to any existing service's configuration. |
| `ubus` through `/usr/share/acl.d/enkrnett.json` | `network reload`, `network.wireless status`, `network.interface.* status`, `hostapd.* get_status/get_clients`, `service list/event`, `system board/info` answer; `file read`, `network.interface.lan down`, `uci get` are refused |
| A direct edit of the owned `wireless` file, then `ubus call network reload` as the user | The new SSID on the air within 9 seconds |

Conclusion: the direct route of ENK-CFG-021 works without `libuci` and without a root helper.
The daemon writes the UCI files itself (`enkrnett_model::uci`) and keeps every foreign section
byte for byte. What still needs root is confined to boot time (§4).

## 2. How a phase is applied

```text
state → Phase (Closed | Setup | Pending | Claimed) → enkrnett_model::plan::owned_sections
      → wireless: radios rewritten in place (country on every one, ENK-CFG-003), enk_* ifaces appended
        network, dhcp, firewall: enk_* sections replaced, foreign lines untouched (ENK-CFG-001, 015)
      → files written 0600, temp-and-rename
      → ubus network reload; ubus service event config.change {dhcp}, {firewall}
      → health check, up to 60 s (ENK-CFG-007): interfaces up, every expected BSS enabled, dnsmasq instances running
```

| Phase | Wi-Fi | Networks | Internet |
|---|---|---|---|
| `Setup` | `Enkrnett Setup <suffix>`: an OWE Transition Mode pair on the management interface | `enk_mgmt` 192.168.207.1/24 | No (no forwarding) |
| `Pending` | `Enkrnett-<suffix>`, WPA2/WPA3-Personal (`sae-mixed`), the generated key | `enk_mgmt` | Yes |
| `Claimed` | The management network and, when enabled, the guest network: a Transition Mode pair in *Open to every device*, one OWE BSS in *Encrypted only* | `enk_mgmt`; `enk_guest` 10.207.0.1/24 | Yes, both; guests never reach private ranges |
| `Closed` | None (the setup window closed; also right after an erase) | `enk_mgmt` kept | No |

Two facts learned on the VM that the code depends on:

- **`wifi-scripts` names a Transition Mode pair `phyN-apM`**, whatever `ifname` says, and gives the
  hidden OWE BSS the SSID of the open one followed by `OWE`. The health check, the counts, and
  the connection line therefore find BSSes by SSID (`plan::expected_bss`), not by object name.
  Single BSSes (`enk-mgmt`, `enk-guest`) get an explicit `ifname`.
- **An empty bridge is down.** The management bridge has no port until a Wi-Fi interface joins
  it, and during a reload none does; without `bridge_empty '1'` the address disappears, the
  daemon cannot bind, and anything connected to the address is cut. Both bridges carry
  `bridge_empty '1'`.

One deviation from RFC 011 §3 as amended by RFC 016 F-13: the setup network attaches to the
`enk_mgmt` interface and DHCP instance instead of having `enk_setup` network and dhcp sections of
its own. Two interfaces cannot hold one address, and the address must be the same in both states
(ENK-MGMT-007). The `wifi-iface` section is still `enk_setup`. To be carried into the next
amendment RFC (ENK-SCOPE-006).

## 3. Readings

| Reading | Source |
|---|---|
| Interface up, address | `network.interface.<name> status` |
| BSS running, SSID | `hostapd.<iface> get_status` for every `hostapd.*` object |
| Encrypted / unencrypted guests | `get_clients` on the OWE BSS (`authorized` and `mfp`) and on the open BSS (`assoc`) |
| Management clients | `get_clients` on the management BSS (`authorized`) |
| Uplink | `network.interface.wan status` |
| The requesting station's protection (`connection.security`) | `/proc/net/arp` for its address, then which BSS lists the station; on the management BSS `mfp` distinguishes WPA3 (SAE requires it) from WPA2. A heuristic until M-4a confirms the flags on real phones. |
| Memory, overlay | `/proc/meminfo`, `statvfs("/overlay")` |
| Hardening report | `service list` (dropbear present?), LuCI paths, listening TCP ports other than 53/67/80, `wps_*` options in `wireless`. ENK-HARD-001 and 003 cannot be observed by the daemon's user and are reported as not met rather than guessed. |
| OpenWrt version, board | `system board` |

## 4. What stays with root, and where

| Need | How |
|---|---|
| Creating the user, owning the files, the sticky directory | Package install (`USERID`) and the first-boot script |
| Reading the `ubusd` ACL | `ubusd` at its start; a package install needs a reboot before the daemon runs |
| Port 80 as a non-root user | `procd` capability `CAP_NET_BIND_SERVICE` |
| Blinking the LED | The init script hands the profile's LED `brightness` attribute to the user at boot |
| Erase | The daemon writes `/etc/enkrnett/ERASE`, takes the Wi-Fi down, and asks for a reboot (`system reboot` through ubus). The init script's `boot()` sees the marker: on an image install `firstboot -y` resets the overlay to the image; on a package install it removes Enkrnett's state and sections, because `firstboot` would remove Enkrnett itself. |
| Reboot | `ubus call system reboot` (in the ACL) |

## 5. The device profile at run time

`--profiles-dir` holds one JSON per profile; the daemon picks the one whose `board_names` contains
`ubus call system board`'s `board_name`. The profile gives the label address source (the setup
suffix), the LED, the fixed country, the argon2 parameters, the radios. `x86_64_generic` is the
VM's profile (Tier 3, no light, label from `eth0`).

## 6. The VM test

`tests/vm/prepare.sh VM_DIR` brings the measurement VM to what the package leaves behind on a
device (user, ACL, ownership, sticky directory, a base configuration without a stock `dnsmasq`
instance) and reboots it. `tests/vm/claim.sh VM_DIR` installs the daemon built for
`x86_64-unknown-linux-musl`, the UI, and the VM profile, starts the daemon as `enkrnett`, and
drives the claim through an SSH tunnel to the management address (port 8080, because `su` grants
no capability). Observed on 2026-11-26:

| Step | Observed |
|---|---|
| Start | `platform_applied phase=setup`; `hostapd.phy0-ap1` "Enkrnett Setup 3456", `hostapd.phy0-ap0` "Enkrnett Setup 3456OWE"; `enk_dns_mgmt` running; listener bound |
| Greeting | `SETUP`, `connection.network: setup` |
| `claim/check`, `claim/confirm` | Proposal "Enkrnett-3456" with a key of the specified form; `PENDING`, `final_in: 600`; the setup section gone from `wireless`; `hostapd.enk-mgmt` "Enkrnett-3456" |
| `login` | `CLAIMED`; the owner record on flash with an argon2id hash; `hostapd.phy0-ap2` "Cafe Example", `hostapd.phy0-ap1` "Cafe ExampleOWE"; `enk_guest` up at 10.207.0.1 |
| `status`, `device` | Effective state from ubus; `installed_as: package`; hardening report honest about dropbear and LuCI on the VM |
| `POST settings` to *Encrypted only* | `settled`, version 2; `hostapd.enk-guest`, one BSS with `encryption owe` |
| Foreign sections | `wan`, `wan6`, the measurement SSH rule untouched; the four files owned by the daemon's user |

Not covered on the VM: real stations joining (hwsim stations in network namespaces are the
M-6/M-8 material and return with the M2 isolation tests), the LED, the erase marker through a
real boot, `procd` with capabilities (the package is built but was not installed on the VM).

## 7. What this stage leaves for M2

The pending marker on flash and the undo at start-up; the disconnection of management stations
and confirmation by the first request on a new connection (ENK-API-021, 022); the bridge-port
`isolated` read-back (ENK-CFG-023, needs `CAP_NET_ADMIN` and netlink, or the init script); the
listener port scan (ENK-CFG-025); drift detection every 10 minutes (ENK-CFG-014); renumbering on
an uplink collision (ENK-TOPO-003, 006); the 5 GHz radio.
