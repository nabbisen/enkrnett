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
| Port 80 | Not by capability: `procd` applies a capabilities file only inside a `ujail`, which the constrained target lacks (verified on the VM: `sysntpd` wraps it in a jail; a plain instance got none). The daemon listens on port 8080 on the management address and the owned firewall sections redirect port 80 of the management interface to it (`enk_mgmt_redirect`). The operator types the address alone. |
| Ownership of the four files after a root-side `uci commit` | The init script restores owner, mode, and the sticky directory at every start (`prepare_files`); found when the erase path's own `uci commit` handed the files back to root |
| Blinking the LED | The init script hands the profile's LED `brightness` attribute to the user at boot |
| Erase | The daemon writes `/etc/enkrnett/ERASE`, takes the Wi-Fi down, and asks for a reboot (`system reboot` through ubus). The init script's `boot()` sees the marker: on an image install `firstboot -y` resets the overlay to the image; on a package install it removes Enkrnett's state and sections, because `firstboot` would remove Enkrnett itself. |
| Reboot | `ubus call system reboot` (in the ACL) |
| Flashing an update | The daemon stages and verifies the image, then fires the `enkrnett.install` event (`service event`, in the ACL already); the init script's raw trigger runs `/usr/libexec/enkrnett/install` as root (§11) |
| Disconnecting the management stations (ENK-API-021) | `hostapd.<mgmt> del_client {addr, reason 3, deauth true}` for every associated station, found by the management SSID (in the ACL) |

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
drives the claim through an SSH tunnel to the management address (port 8080; a local tunnel does
not pass the firewall's port-80 redirect). Observed on 2026-11-26:

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

Done in M2 stage 1 (`docs/src/design/daemon.md` §6): the pending marker on flash and the undo at
start-up; the disconnection of management stations and confirmation by the first request on a
new connection (ENK-API-021, 022). Still open: the bridge-port `isolated` read-back (ENK-CFG-023,
needs `CAP_NET_ADMIN` and netlink, or the init script); the listener port scan (ENK-CFG-025);
drift detection every 10 minutes (ENK-CFG-014); renumbering on an uplink collision (ENK-TOPO-003,
006); the 5 GHz radio. For the next amendment RFC: the port-80
redirect (the listener port scan of ENK-CFG-025 then concerns 8080 too) and the setup network on
the management interface (§2).

## 8. Stage 5: the package through `procd`

`tests/vm/package.sh` builds the package with the pinned x86/64 SDK (the test vehicle's target in
the lock), installs it with `apk add --allow-untrusted`, and reboots. `tests/vm/contract.sh` then
runs the checks derived from RFC 003 and RFC 004 against the service `procd` started: request
security, authorization by state, the claim, sessions, the login delay with `Retry-After`, and
Erase through a real boot on the package path (the init script removes Enkrnett's state and
sections; `firstboot` is for image installs). Results: `docs/src/verification/m1.md`.

## 9. M2 stage 2: separation read back from the kernel

Chapter 2 is mostly configuration the daemon already wrote in M1 (two bridges, three zones with
input and forward `REJECT`, the reject of private and uplink ranges before the forwarding to
`wan`, IPv6 dropped on input and forward, one `dnsmasq` instance per network kept off loopback
except one, `isolate` on every BSS). What this stage adds is the part the platform does not do
and the read-backs that make the health check tell the truth (ENK-CFG-026).

| Fact | Set by | Read back by the health check |
|---|---|---|
| Bridge-port `isolated` on every port of `br-guest` and `br-mgmt` (ENK-CFG-023; M-8 row 6: OpenWrt 25.12 gives it to the radio's first BSS only) | `/etc/hotplug.d/net/50-enkrnett-isolate`, as root, when a network interface appears, and the init script at every service start. sysfs is root's; the daemon cannot write it. | `/sys/class/net/<bridge>/brif/<port>/isolated` for every port of every expected bridge; a `0` is `isolation_missing` |
| One DNS listener per network (ENK-CFG-024) | The owned `dhcp` sections | The `dnsmasq` instances running (ubus `service list`) and a UDP socket on port 53 bound to each network's address (`/proc/net/udp`); otherwise `dhcp_missing` |
| The daemon's listener on the management address only (ENK-CFG-025) | `--listen` from `/etc/config/enkrnett` | Any TCP listener on port 80 or the daemon's port bound to another address, or to every address, is `listener_exposed` (`/proc/net/tcp`, `tcp6`). One exception: when an apply moved the management network (ENK-TOPO-006), the daemon's own listener on the address it is leaving is not exposed, since that address is on no interface any more; without the exception the health check of the move waited its full 60 s and reported `ListenerExposed` on the VM. |
| Each network sees its own services only (chapter 2 §3) | The DNS and UI rules of a zone carry `dest_ip` of that network's address, so a management client cannot reach the guest DNS instance on the device's guest-side address (found by `tests/vm/isolation.sh`) | `tests/vm/isolation.sh` |
| Nothing else listening (ENK-HARD-005) | The image (M3) | The hardening report: every bound socket other than DNS, DHCP, and the daemon on the management address makes ENK-HARD-005 "not met" |

Interpretation to carry into the next amendment RFC: ENK-CFG-025 asks the health check for "a
port scan of the device from the guest network". The daemon lives on the device and cannot stand
on the guest network; what it can verify is the kernel's list of bound sockets, which is what a
scan would find before the firewall. The scan from a real guest station is
`tests/vm/isolation.sh`, which runs the rows of measurement M-8 against the daemon's own
configuration with four simulated stations in network namespaces.

Not read back, and why: PMF on the OWE BSS (`pmf_missing`): hostapd's `get_status` does not
expose `ieee80211w`, and the generated configuration is root's. wifi-scripts always writes
`ieee80211w=2` for `encryption owe`, and every OWE station's `mfp` flag in `get_clients` is the
observable consequence; the status counts encrypted guests by it. `mode_differs`: the set of
running BSSes per SSID is what the health check compares; hostapd does not report the encryption
of a BSS through ubus.

## 10. M2 stage 3: adopting, drift, renumbering

| Operation | What the OpenWrt platform does |
|---|---|
| `adopt_settings` (start-up without a marker) | Builds the plan (radios, country, addresses after `renumber` against `network.interface.wan`'s subnet). If renumbering changed an address: write and reload, that is the daemon's own duty. Otherwise compare the four files' owned sections with the regenerated ones; equal: set the current plan and verify; different: the same, with `drift: true`. Nothing is written on drift. |
| `owned_drift` | The same comparison against the current plan, for the periodic check. Sections compare as sets of entries (`Section::equivalent`): a section the daemon rendered and the one regenerated from it may list the same options in another order, and `uci` re-renders in its own order. |
| `uplink_collides` | `renumber(current addresses, uplink subnet) != current addresses` |
| Addresses on flash | The daemon keeps no address file: the owned `network` sections *are* the record (`enk_mgmt`, `enk_guest` `ipaddr`), read at start-up before the plan is built |
| A write refused with `EPERM` | `PlatformError::NotOurs`: the sticky directory lets the daemon replace only its own files; a root-side `uci commit` made the file root's |
| The listen address | `listen_follows_management_address` is true here: the daemon binds the management address of its current plan with the configured port. A moved address means `restart_service` (the `config.change` event, `service: event` in the ACL) and a new process. |
| `restart_service` | A `config.change` event for the package `enkrnett` on procd's `service` object (the same call that reloads dnsmasq and firewall4); the init script's `procd_add_reload_trigger "enkrnett"` turns it into `/etc/init.d/enkrnett restart`: stop, then `start_service` with `prepare_files`, `isolate_ports`, and the fresh command line. A respawn after a plain exit would repeat the old command line and skip the init script (the first attempt of M2 stage 3 showed a respawn loop until procd gave up). The `rc` object used first belongs to rpcd, which a minimal image does not carry; the reproducible test VM of stage 4 found that. |
| `update_index_url` | The `--update-index` argument, which the init script reads from `/etc/config/enkrnett` (`option update_index`), a root-owned file the daemon does not read itself; `None` when unset, and then every check answers `unreachable` with "no index url" in the log |
| `fetch` | `/bin/uclient-fetch -q -O - --timeout=15 URL` through `posix_spawn` (`platform/spawn.rs`: `/dev/null` for input, a pipe for output bounded by the caller's limit, a fixed environment, the program's standard error joined to the daemon's, which procd logs). Only `https://` URLs are tried: libustream-mbedtls verifies the server's certificate against the image's bundle in `/etc/ssl/certs`; the authenticity of what arrives rests on the signify signature, not on the transport (RFC 016 §9). The URL is the configured one, never a value from a request (ENK-SEC-014) |
| `release_keys` | The `*.pub` files of `/usr/share/enkrnett/keys`, which the package installs from `openwrt/release-keys/` (none yet: the release key is the owner's act on the signing host) |

What the comparison does not count as drift: a change to a radio's hardware options (channel,
`htmode`) by hand, because the daemon writes those from the existing section and only adds the
country; chapter 6 §3 lists every `wifi-device` as owned, so this is an interpretation to carry
into the next amendment RFC.

## 11. M3 stage 2: the update's root side

The daemon's part is `docs/src/design/daemon.md` §6c. On this platform:

| Method | How |
|---|---|
| `download` | `/bin/uclient-fetch -q -O /tmp/enkrnett-update/image --timeout=15 URL` through `spawn::run_watching`: the downloader's output goes to the file (`/tmp` is tmpfs: RAM, never flash) while the daemon measures the file every 100 ms and kills the downloader when it passes the size the index states, so a server cannot fill the memory. `https://` only, as `fetch`. |
| `install` | Writes `request` (the attempt's token) beside the staged `image` and `image.sig`, then `ubus call service event '{"type":"enkrnett.install","data":{}}'`. procd runs the init script's `procd_add_raw_trigger "enkrnett.install" 1000 /usr/libexec/enkrnett/install` as root, with `PATH` set; verified on the VM, also that the daemon's user may fire the event (the ACL's `service.event`, which `restart_service` uses already). |
| `install_verdict` | Reads `/var/run/enkrnett-install/result`, `TOKEN WORD [REASON]`, and answers only for its own token. `/var/run` is root's: the daemon's user cannot place a file or a symbolic link there (and OpenWrt sets `fs.protected_symlinks`). |

The root helper, `openwrt/package/enkrnett/files/enkrnett.install`, installed to
`/usr/libexec/enkrnett/install`:

1. copies `image.sig` and `image` out of the daemon's directory into `/var/run/enkrnett-install/`,
   which only root writes, and removes the daemon's copy: the daemon's user cannot swap the bytes
   between the checks and the flash (the copy is the second image in RAM for a few seconds; the
   daemon's memory check, image plus 16 MB, allows for it);
2. verifies the signature once more with `usign` (OpenWrt's signify-compatible tool, 736 KB of
   peak memory on a 20 MB file, in the image already and a dependency of the package) against
   every `*.pub` of `/usr/share/enkrnett/keys`: a compromised daemon cannot have root flash what
   the project did not sign (`refused bad_signature`);
3. `sysupgrade -T`: the system's own check of the image against the device
   (`refused wrong_device`);
4. `say flashing`, then `sysupgrade -q`, which keeps the configuration (`/etc/config/*`,
   `/etc/enkrnett/` through `/lib/upgrade/keep.d/enkrnett`, the package's `conffiles`) and does
   not return: procd ends every process and the device restarts into the new release. Should it
   return, `failed install_failed`.

The helper never trusts a path from the daemon beyond the fixed staging directory, writes
nothing there but a removal, and reads the token through `tr -cd 'A-Za-z0-9_-'`. On x86
`sysupgrade` accepts the gzip ext4 combined image (`platform_check_image`: the boot sector's
magic and the partition table compared with the disk's); the VM's disk, resized on the host,
keeps the image's table. The configuration goes along as `/sysupgrade.tgz` on the new root
filesystem and is unpacked at the first boot, before the init script runs: the daemon finds
`state.json` and `update.json` where it left them.

`installed_as` (ENK-INST-005, which decides ENK-UPD-008's default) read `/rom/usr/sbin/enkrnettd`
alone until the VM test: an x86 ext4 image has no `/rom`, so a device updated to an image still
said `package`. The project's image build now places `/usr/share/enkrnett/from-image` in every
image, outside the package, and the platform reads either witness; the package never installs
that file. `tests/vm/successor.sh` does it for the test images; the release image build of stage
3 must do the same.

What the VM test observed is in `docs/src/verification/m3.md` (stage 2).
