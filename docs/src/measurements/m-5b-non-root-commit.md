# M-5b — Non-root commit of the owned configuration

| | |
|---|---|
| **Measurement** | M-5b (RFC 011 §7; handoff of RFC 006) |
| **RFC** | 006 Feasibility measurements; 011 Configuration model and transactions |
| **Carried out by** | Architect, on the owner's instruction (implementation stage) |
| **Date** | 2026-11-26 |
| **State** | Ready for review |

## Question

Can a non-root user with ownership of `/etc/config/wireless`, `network`, `firewall`, `dhcp`, and a
sticky bit on the directory, commit changes and reload the platform, and can it touch nothing
else? If not, the fallback is a small root helper over a unix socket.

## Setup

OpenWrt 25.12.5 (r33051) x86-64 under QEMU/KVM, the M-6 virtual machine. User `enkrnett`
(uid 1900), `/usr/share/acl.d/enkrnett.json` as in `openwrt/package/enkrnett/files/`, the four
files `chown enkrnett`, `chmod 0600`; `chgrp enkrnett /etc/config; chmod 1775 /etc/config`;
reboot so that `ubusd` reads the ACL.

## Procedure and observations

| # | As `enkrnett` | Observed |
|---|---|---|
| 1 | Copy `wireless` to a temporary name in `/etc/config`, rename it over `wireless` | Succeeds |
| 2 | Rename a new file over `system` (root, 0600) | `mv: can't rename: Operation not permitted` |
| 3 | `rm /etc/config/system` | `Operation not permitted` |
| 4 | `cat /etc/config/system` | `Permission denied` |
| 5 | Create `/etc/config/zz_new` | Succeeds (the directory is group-writable) |
| 6 | `ubus call network.wireless status`, `network.interface.mgmt status`, `hostapd.guest-owe get_status`, `get_clients`, `service list`, `system board` | Answer |
| 7 | `ubus call file read`, `network.interface.lan down`, `uci get` | Refused (`Parsing message data failed` for objects outside the ACL; `Permission denied` for a method outside it) |
| 8 | `sed` the management SSID in the owned `wireless` file, then `ubus call network reload` | Reload accepted; `hostapd get_status` shows the new SSID 9 seconds later |
| 9 | `/proc/net/arp` | Readable |
| 10 | `/sys/class/leds` | Empty on the VM; not measured |

Note on 5: `uci` itself was not needed; the daemon writes the files. Writing through `libuci` with
`-P` for a private delta directory was therefore not measured.

## What it changes

| Affected | Change |
|---|---|
| RFC 011 ENK-CFG-021 | The direct route holds: the daemon's user owns the four files; no `libuci`, no root helper for configuration. The arrangement is the package's first-boot script. |
| RFC 011 §7 fallback (root helper) | Not needed for configuration. Root stays at boot time only: user creation, file ownership, the ACL read by `ubusd`, LED attribute hand-over, and the erase marker. |
| Chapter 6 §7 | Add: the sticky directory lets the user create new files; stated as a known limit. |
| Implementation | `enkrnett_model::uci`, `enkrnettd::platform::openwrt`; `docs/src/design/platform.md`. |

## Raw data

The commands and their output are in the session record; the scripts are
`tests/vm/prepare.sh` and the inline steps above.
