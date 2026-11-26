# Build for the device (milestone M1, stage 3; the release build, M3 stage 3)

How the daemon is built for OpenWrt, how its size is held to the budgets of chapter 8, how the
OpenWrt package is made, what a device profile says, and (§7) how a release's images are built
from the lock and compared. The design is chapter 11 (RFC 013); this document says what exists.

## 1. What this stage delivers

| Delivered | Not yet |
|---|---|
| `openwrt/lock/inputs.json`: the pinned inputs (SDK and Image Builder URLs with SHA-256 for both targets, the Rust nightly, the squashfs settings, the budgets) | — (M3 stage 3 added the package locks and the image build, §7) |
| `openwrt/build-mips.sh`: fetch and verify the SDK, build `enkrnettd` for `mipsel_24kc`, measure against ENK-FOOT-001 to 003, optionally build the `.apk` | — (`--twice` compares two builds, §7) |
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

**The real figure (CI, `mksquashfs`), 2026-11-27: the daemon is 300 KiB of 300**, the UI 48, the
extras 8; the xz estimate on a workstation without `squashfs-tools` runs about 14 KiB under the
squashfs figure for the daemon, and the figure is padded to 4 KiB. The headroom is between
nothing and 4 KiB; see `docs/src/verification/m3.md` stage 3.

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

## 7. The release build (M3 stage 3)

Chapter 11 §3: every published file can be rebuilt from the repository and a lock of pinned,
hash-recorded inputs. What exists, script by script; the procedure is
`docs/src/contributors/releasing.md`.

### The lock (ENK-BUILD-007)

| File | Records | Changed by |
|---|---|---|
| `openwrt/lock/inputs.json` | Per target the Image Builder and SDK archive URL and SHA-256; the release's download tree (`openwrt.downloads`); the Rust nightly, target, `build-std` flags; the Node major; the squashfs settings; the budgets; where the other locks are | Hand, on purpose |
| `openwrt/lock/packages/<profile>.json` | Every OpenWrt package the image of that profile installs: name, version, path under the download tree, SHA-256; the three packages the Image Builder archive carries itself (`base-files`, `kernel`, `libc`), covered by the archive's hash; OpenWrt's own `version.buildinfo` revision and `feeds.buildinfo` commits, for the source offer | `openwrt/lock-packages.sh PROFILE --apk ENKRNETT.apk`: the pinned Image Builder resolves the profile's set (`make manifest`, reading the live feeds this once), each resolved package is fetched from the feed that lists it and hashed. A library the feed lists under its plain name and ships with an ABI suffix (`libubox` → `libubox20260213-….apk`) is matched by the suffix rule |
| `Cargo.lock`, `ui/package-lock.json`, `openwrt/patches/*.patch` | The Rust and npm dependency trees; the carried fixes | Committed with the tree |

The project's own package, `enkrnett`, is not in the package lock: it is what the release builds,
from the lock's toolchain.

### The image build (`openwrt/build-image.sh`; ENK-BUILD-008, 010; ENK-RLS-002)

```text
openwrt/build-image.sh PROFILE --apk ENKRNETT.apk [--out DIR] [--files DIR] [--add PKGS] [--drop PKGS] [--live] [--twice] [--version V]
```

| Step | What happens |
|---|---|
| The Image Builder | Fetched by the lock's URL into `openwrt/.cache/ib/<target>/`, checked against the lock's SHA-256, extracted once |
| The packages | Each file of the package lock fetched by its path from `ENKRNETT_MIRROR` (the project's pinned mirror, RFC 013 §6) or else from the release's download tree, into `openwrt/.cache/mirror/<path>`, checked against the lock's SHA-256 (a mismatch stops the build); copied into the Image Builder's `packages/` directory, which it indexes and signs with its own local key; the `repositories` file emptied, so the Image Builder has no feed to read. `--live` (never for a release) restores the feeds instead, for the test VM's extra packages |
| The carried patches | Each patch header names the installed file and its package (`Installed file: /usr/share/ucode/wifi/ap.uc (package wifi-scripts)`). The file is extracted from the locked package with the Image Builder's own `apk extract`, patched with `patch`, and placed in the image's overlay. A patch that no longer applies stops the build with its removal condition to review (ENK-CFG-004, ENK-REPO-002). The test VM of `tests/vm/image.sh` still applies it at first boot; a release image carries it from the build |
| The witness | `/usr/share/enkrnett/from-image`, holding the image's name: what tells the daemon it was installed as an image on a target without `/rom` (`platform.md` §11); the package never places it |
| `make image` | `PROFILE=` the profile's Image Builder name, `PACKAGES=` the profile's removals and additions (`image.packages_remove`, `packages_add`) plus `--add`, minus `--drop`, `FILES=` the overlay; the Image Builder's `SOURCE_DATE_EPOCH` is fixed in its `include/version.mk`, so the result does not depend on the clock |
| The names | `enkrnett-VERSION-BASELINE-PROFILE-sysupgrade.bin` (the ext4 combined image on x86/64), `…-VENDORKIND.bin` when the profile's `install.vendor_image_kind` is not `none` and the Image Builder made one; the `.manifest` beside them. VERSION from the package's file name unless `--version` says otherwise |
| `--twice` | The same build again into a temporary directory and `cmp` of every image; the result goes into `enkrnett-VERSION-BASELINE-PROFILE.build.json` (`built_twice_identical`) with sizes and SHA-256/SHA-512. A difference is stated, not failed: ENK-BUILD-010 is a should and the release notes state the result |

Measured on 2026-11-27 (the three profiles, `--twice`, from the lock): every image bit-identical
between the two builds, and the Tier 1 images, the daemon and the package byte-identical between
the workstation and the GitHub runner on the same commit; `elecom_wrc-1167gst2` sysupgrade 7,668,278 bytes and factory 7,667,806
of a 24 MiB limit, `buffalo_wcr-1166ds` 5,701,922 and 5,702,164 of 7,936 KiB, `x86_64_generic`
14,935,265 (ext4, gzip). The `elecom` image carries no `dropbear` and no `ppp`, as its profile
removes them.

### The double builds of the daemon and the UI (ENK-BUILD-009)

`openwrt/build-mips.sh --twice` builds the daemon a second time into `target/mips-again` with
the same lock and compares; a difference fails. Measured 2026-11-27: identical
(`b2f3390d…`), with `-Zlocation-detail=none`, `strip = true` and `panic = "abort"` leaving no
path in the binary. The UI: `vite build` into a second directory and `diff -r`; identical, the
asset hashes being content hashes. CI does both on every push (`ci.yml`, jobs `mips` and `ui`).

### The release directory (`openwrt/release-assemble.sh`; ENK-BUILD-012)

Checks that the version is the same in `Cargo.toml`, the package Makefile, `ui/package.json` and
`tests/browser/package.json`; refuses images built with `--live`; copies the images, manifests and
build reports; writes `SHA256SUMS` and `SHA512SUMS`; copies the lock (`inputs.json`, the package
locks, `Cargo.lock`, the UI lock, the patches) with its own sums; writes `build-report.md` (the
comparison table for the release notes) and `source-offer.md` (OpenWrt's tag and revision, the
feed commits from the lock, where the binary packages came from, the carried patches, Enkrnett's
own source, how to ask for a copy); with `--sequence N --notes-url URL` also `index.json` for the
signing host, with `min_free_kib` and `install_seconds` from each profile's `update` block. CI's
`images` job uploads the directory as `enkrnett-release-unsigned`; the signatures are the
signing host's (ENK-RLS-007).

### The CI job `images`

Needs the `mips` job's package; caches `openwrt/.cache/ib` and `openwrt/.cache/mirror` by the
hash of `openwrt/lock/**`; builds the two Tier 1 profiles with `--twice`; assembles and uploads
the release directory. The x86-64 profile is built from the lock by the `vm` workflow instead,
as the successor images of `tests/vm/update-install.sh`, and so is installed and booted on every
push.

