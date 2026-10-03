# M-5 — Confirm-or-revert by a process that is not root

| | |
|---|---|
| **Measurement** | M-5 |
| **RFC** | 006 Feasibility measurements (`rfcs/accepted/006-feasibility-measurements.md`) |
| **Handoff** | `rfcs/handoffs/006-feasibility-measurements/README.md` |
| **Carried out by** | Architect (acting for the development team, by the owner's instruction) |
| **Date** | 2026-10-04 |
| **State** | Ready for review |

## Question

Can configuration be applied with confirm-or-revert through OpenWrt's own interfaces, by a
process that is not `root`?

## Answer in one paragraph

Yes, with three conditions. A user that is not `root` can be confined by `ubusd`'s own access
control (`/usr/share/acl.d/*.json`) to exactly the objects and methods it needs; everything else
answers "Not found". Through `rpcd`'s `uci` object the user can stage a change, apply it with
`rollback: true` and a timeout, and either confirm it or let it revert: without confirmation the
previous configuration returned 30–35 s after the apply, and with confirmation the new one stayed.
The conditions: (1) `ubusd` reads its ACL files only when it starts, so the ACL must be installed
in the image, not at run time; (2) `rpcd`'s `uci apply` needs an `rpcd` session, obtained with a
password login of a system user whose rights are limited by an `rpcd` ACL (here: the `wireless`
configuration only; a `set` on `network` was refused with "Permission denied"); (3) **the rollback
does not survive a restart**: the snapshot lives in `/var/run/rpcd/` and the timer in `rpcd`'s
memory, so a reboot inside the window leaves the unconfirmed configuration active. Without `rpcd`,
`uci` as a non-root user failed with "I/O error" regardless of file ownership, because it also
needs a writable delta directory; `ubus call network reload` as the confined user works.

## Environment

| Item | Version or model |
|---|---|
| OpenWrt | 25.12.5, the M-6 virtual machine (see M-6 for the full package list) |
| rpcd | 2026.06.04~28faf640-r1 with `rpcd-mod-file`, `rpcd-mod-iwinfo`, `rpcd-mod-rpcsys`, `rpcd-mod-ucode`, `rpcd-mod-luci` (as shipped in the x86-64 image) |
| ubus / ubusd | 2026.06.28~24864e78-r2 |
| Added | `shadow-su` (to run commands as the test user) |
| Access point | `radio0`, `phy0-ap0`/`phy0-ap1` in Transition Mode, with the M-6 OWE station `sta1` connected |

## Setup

1. System user `enk` (uid 900, no shell, password set via `/etc/shadow`).
2. `/usr/share/acl.d/enk.json` grants `enk`: `network.wireless status`; `hostapd.* get_clients, get_status`; `network reload`; `network.interface.lan status`; `uci get, set, commit, changes, apply, confirm, rollback`; `session login, access, destroy`; sending the event `config.change`.
3. `/usr/share/rpcd/acl.d/enk.json` defines an `rpcd` access group `enk` with read and write rights on `uci` for the configuration `wireless` only, plus the `ubus` methods above; `/etc/config/rpcd` gets a `login` section `username 'enk'`, `password '$p$enk'`, read/write group `enk`.
4. `rpcd` restarted; the VM rebooted once so that `ubusd` loads the ACL.

## Results

| Step | Result | Evidence |
|---|---|---|
| 1 Methods needed | `network.wireless: status`; `hostapd.<ifname>: get_clients, get_status`; `uci: get, set, changes, commit, apply, confirm, rollback` (all take `ubus_rpc_session`); `network: reload`; `session: login, access`. `ubus -v list` output in the log. | `m5.out` step 1 |
| 2 Confinement | `ubus send ubus.acl.sequence` as root: "Permission denied". Before the reboot `enk` saw nothing ("Not found" for every object). After the reboot `enk` could call the permitted methods. | `m5.out` step 2–3, `m5b.out` |
| 3 Needed calls as `enk` | `network.wireless status` ✓, `hostapd.phy0-ap0 get_status` ✓, `get_clients` ✓, `session login` → session id, `session access` for `ubus/uci/apply` = true, for `uci/wireless/write` = true, for `uci/network/write` = false; `uci get` on `wireless` ✓. | `m5b.out` step 3 |
| 4 Refusals | `file read /etc/shadow`: Not found. `system info`: Not found. `network.interface.lan down`: Not found. Through the session, `uci set` on `network`: **Permission denied**; `uci get` on `network`: Permission denied. | `m5.out` step 4, `m5b.out` step 4 |
| 5 Apply, no confirm | `uci apply {rollback:true, timeout:30}` → config and on-air SSID changed within 5 s; snapshot directories appeared under `/var/run/rpcd/`; at t+35 s config **and on-air SSID were back** to the previous value. | `m5b.out` step 5 |
| 6 Apply, confirm | Same apply; `uci confirm` at t+8 s; at t+40 s the new SSID was still active. | `m5b.out` step 6 |
| 7 Restart inside the window | `uci apply {rollback:true, timeout:90}`, reboot at t+5 s. After the reboot: config `m5-reboot-test`, on air `m5-reboot-test`; `/var/run/rpcd/` empty. **The unconfirmed change stayed.** | `m5b.out` step 7 and the post-reboot check |
| 8 Without rpcd | `uci set`/`uci commit` as `enk`: "I/O error" with the file root-owned, with the file owned by `enk`, and with `/etc/config` group-writable. `uci` needs its delta directory (`/tmp/.uci`, root-only) or `-t`/`-P` options. `ubus call network reload` as `enk`: exit 0, the access point reloaded. | `m5c.out` |
| 9 Service interruption | See the table below. | `m5.out` steps 9a, 9b |

### Step 9: time without connectivity for the connected OWE station

Measured with one ping per 0.5 s from the station to its gateway; a reload was triggered by
`uci commit` + `wifi reload`.

| Change | Samples failed | ≈ time without service |
|---|---:|---:|
| Network name changed (same mode) | 14 | 7 s |
| Transition Mode → OWE-only | 6 | 3 s |
| OWE-only → Transition Mode | 39 | 19.5 s |

The third case includes the station's search for the renamed hidden BSS (see M-6, observation 2).
A change of the key of a WPA2/WPA3 network was not measured; it is the same reload path as the
first row.

## Measured values

| Quantity | Value | Unit | Provisional budget |
|---|---:|---|---:|
| Rollback after apply without confirm (timeout 30 s) | between 20 and 35 | s | — |
| Outage on rename | ≈ 7 | s | — |
| Outage OWE-only → Transition Mode | ≈ 19.5 | s | — |

## Deviations from the procedure

| Deviation | Why |
|---|---|
| Two runs (`m5.sh`, then `m5b.sh`) | In the first run the test user's password was not set (`chpasswd` is absent on OpenWrt), so the session login failed and steps 5–7 ran without a session. The second run repeated steps 3–7 with the password set through a `crypt(3)` hash. |
| Step 8 run a third time | `su` without `/sbin` in the path could not find `uci`. |
| The reboot for step 7 was triggered from inside the window by the script | As specified; the check was made over SSH after the reboot. |

## What contradicts an RFC

| RFC | Requirement | Observation |
|---|---|---|
| 004 | ENK-REL-006 (proposed in the gap analysis; RFC 005 open question 3 / D-12): "a change that the operator cannot confirm shall revert" | `rpcd`'s rollback does not survive a restart. The platform mechanism alone does not meet the intended semantics; Enkrnett must persist its own "pending" marker and revert at start, or implement the window itself. |

## Observations beyond the question

1. **ACL at start only.** `ubusd` evaluates `/usr/share/acl.d/` when it starts. A daemon that installs its ACL at first run would have no rights until the next boot. Ship the ACL in the image or the package, and document a `ubusd` restart for the package path.
2. **Hidden, not denied.** Objects outside the ACL are invisible ("Not found"), which also hides their existence from `ubus list`. Good for exposure, confusing for diagnostics.
3. **Two layers of permission.** `ubusd` controls which methods the user may call; `rpcd` controls, per session, which configurations `uci` may touch. Both are needed: the `ubusd` ACL alone would let the user set any configuration through `uci set`.
4. **Session lifetime.** `rpcd` sessions expire after 300 s without use by default; a daemon using them must renew or re-login.
5. **Default `rpcd` apply timeout** is 60 s when `timeout` is omitted; the call accepted 30 and 90.
6. Nothing in this path needed `root` for the daemon itself; `rpcd` runs as root on the system's behalf.

## How to repeat

```sh
# in the M-6 VM, as root; HASH from: openssl passwd -6 m5-measure-pw
sh /root/m5.sh            # sets up user, ACLs, rpcd login; runs step 1-2 and the outage measurements
reboot                    # ubusd loads the ACL
sh /root/m5b.sh "$HASH"   # steps 3-8, reboots inside the window at the end
uci get wireless.default_radio0.ssid   # after the reboot
```

Scripts and logs (`m5.sh`, `m5b.sh`, `m5.out`, `m5b.out`, `m5c.out`) are kept under
`.git-exclude/tmp/m/vm/` on the workstation.
