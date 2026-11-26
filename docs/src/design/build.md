# Build for the device (milestone M1, stage 3)

How the daemon is built for OpenWrt, how its size is held to the budgets of chapter 8, how the
OpenWrt package is made, and what a device profile says. The design is chapter 11 (RFC 013); this
document says what exists.

## 1. What this stage delivers

| Delivered | Not yet |
|---|---|
| `openwrt/lock/inputs.json`: the pinned inputs (SDK and Image Builder URLs with SHA-256 for both targets, the Rust nightly, the squashfs settings, the budgets) | The image build with the Image Builder from the lock, and the lock of every package version (ENK-BUILD-007 in full; stage 4 and M3) |
| `openwrt/build-mips.sh`: fetch and verify the SDK, build `enkrnettd` for `mipsel_24kc`, measure against ENK-FOOT-001 to 003, optionally build the `.apk` | Bit-identical double build in CI (ENK-BUILD-009) |
| `openwrt/package/enkrnett/`: the OpenWrt package with init script, `ubusd` ACL, UCI defaults, first-boot script | Installed and exercised through `procd` on the VM in stage 5 (`docs/src/design/platform.md` §8) |
| Two device profiles with the schema in `enkrnett_model::profile`, parsed and checked by tests | Verification of any hardware field on a unit (M-3, M-4, M-7) |
| The `mips` CI job: budgets enforced on every build (ENK-BUILD-011), artifacts uploaded | |

## 2. The MIPS build

`mipsel-unknown-linux-musl` is a Tier 3 Rust target without a prebuilt standard library (M-2).
The script builds it with the pinned nightly and `-Zbuild-std=std,panic_abort`, links with the
SDK's `mipsel-openwrt-linux-musl-gcc`, and keeps dynamic linking against the platform's musl
(ENK-FOOT-015). The release profile is the workspace's: `opt-level = "z"`, fat LTO, one codegen
unit, `panic = "abort"`, stripped.

| Input | Pinned as |
|---|---|
| OpenWrt SDK 25.12.5, `ramips/mt7621` | URL and SHA-256 in the lock; checked before extraction |
| Rust | `nightly-2026-11-21` with `rust-src` |
| Target and flags | `mipsel-unknown-linux-musl`, `-C target-feature=-crt-static -Zlocation-detail=none`; `std` built with `-Zbuild-std=std,panic_abort -Zbuild-std-features=optimize_for_size` (no backtrace machinery: with `panic = "abort"` a panic's message still reaches the log, its file and line do not) |

Both supported targets (`mt7621`, `mt76x8`) share `mipsel_24kc` and the same gcc 14.3.0 musl
toolchain, so one daemon binary serves both.

## 3. Sizes measured on 2026-11-26

| Item | Measured | Budget | Method |
|---|---:|---:|---|
| Daemon (ENK-FOOT-001) | 224 KiB | 300 KiB | xz with OpenWrt's squashfs settings plus 1 KiB per file; `mksquashfs` was not installed on the workstation |
| UI, both languages (ENK-FOOT-002) | 49 KiB | 150 KiB | same |
| Everything else the package adds (ENK-FOOT-003) | 7 KiB | 50 KiB | same |
| Daemon raw | 723,488 bytes | — | `stat` |

The script prefers `mksquashfs` with the settings from the lock (`-comp xz -Xdict-size 100% -b
256K`) and records which method produced the figure in `openwrt/out/sizes.json`; CI installs
`squashfs-tools` and produces the real figure. M-2 found the estimate and the real figure within
about 1 KiB of each other.

History: stage 3's daemon (the API against the mock platform only) was 790 KB raw and 256 KiB.
Stage 4 added the OpenWrt platform and brought it to 987 KB and 303 KiB, over the budget. Stage 5
built `std` for size and without backtraces and panic locations: 723 KB and 224 KiB, with the
platform code in. M2 stages 1 to 3 (the transaction, the kernel read-backs, drift and
renumbering) brought it to 240 KiB (raw 789,028 bytes; the xz estimate moves by one between builds). The budget stands; the next lever, if it is ever needed, is
`panic_immediate_abort`, which costs the panic message in the log.

## 4. The OpenWrt package

The daemon is not compiled inside the SDK. `openwrt/build-mips.sh --package` copies
`openwrt/package/enkrnett/` into the SDK and runs `make package/enkrnett/compile` with
`ENKRNETT_PREBUILT` pointing at the daemon and `ui/dist`, so that the bytes measured are the bytes
packaged. The result is `enkrnett-0.1.0-r1.apk` for `mipsel_24kc` (OpenWrt 25.12 uses `apk`).

| File in the package | Purpose |
|---|---|
| `/usr/sbin/enkrnettd` | The daemon |
| `/usr/share/enkrnett/ui/` | The UI (`index.html` and hashed assets) |
| `/etc/init.d/enkrnett` | `procd` service: user and group `enkrnett` (created at install by `USERID`), no capability (port 80 is redirected to the daemon's port 8080 by the owned firewall sections; `procd` grants capabilities only inside a `ujail`), ownership of the four owned files restored at every start, `--listen` from `/etc/config/enkrnett`, state in `/etc/enkrnett`, respawn |
| `/usr/share/acl.d/enkrnett.json` | `ubusd` ACL (ENK-CFG-020): `network` reload; `network.wireless` status, reload, up, down; `network.interface.*` and `network.device` status; `hostapd.*` get_status, get_clients, del_client; `service` list; `system` board, info, reboot; `led` set |
| `/etc/config/enkrnett` | `listen` and `development`; nothing the operator decides |
| `/etc/uci-defaults/90-enkrnett` | First boot: the state directory belongs to the daemon's user |

The state directory is `/etc/enkrnett` (on the overlay, so it survives a reboot; `/var` is a
tmpfs on OpenWrt). The arrangement of the owned configuration sections (ENK-CFG-021) waits for
measurement M-5b and the OpenWrt platform.

## 5. Device profiles

`openwrt/profiles/<id>/profile.json`, schema `enkrnett_model::profile::DeviceProfile`, rules in
`DeviceProfile::check` (tier bounds; Tier 1 needs hardware verification, ENK-TIER-004; Tier 1 on
reduced hardening needs a separate decision, ENK-TIER-005; at least one radio with four
interfaces; class against flash and RAM; a two-letter fixed country; positive argon2 parameters;
the label address declared). The model crate's tests parse the shipped profiles, so a profile
that breaks a rule fails `cargo test`.

| Profile | Class | Target | State |
|---|---|---|---|
| `elecom_wrc-1167gst2` | standard | `ramips/mt7621` | Tier 2; hardware fields unverified |
| `buffalo_wcr-1166ds` | constrained | `ramips/mt76x8` | Tier 2; image measured in M-3 part 1; hardware fields unverified |

## 6. Running it in CI

The `mips` job installs `squashfs-tools`, the pinned nightly with `rust-src`, caches the SDK by
the lock's hash, downloads the UI built by the `ui` job, runs `openwrt/build-mips.sh --package`,
and uploads the daemon, `sizes.json`, and the `.apk`. A budget overrun fails the job
(ENK-BUILD-011).
