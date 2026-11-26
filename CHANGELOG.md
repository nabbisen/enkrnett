# Changelog

Releases use semantic versioning; tags carry no prefix (RFC 013 ENK-RLS-001). The history of
decisions is in `rfcs/`.

## Unreleased

### Milestone M3, stage 3 (2026-11-27)

- The package lock (ENK-BUILD-007): `openwrt/lock/packages/<profile>.json`, every OpenWrt package
  an image installs with version, path under the release's download tree and SHA-256, plus
  OpenWrt's own revision and feed commits; written by `openwrt/lock-packages.sh` from the pinned
  Image Builder's resolution, reviewed as a diff. `inputs.json` names the download tree, the Node
  major and where the other locks are. Device profiles gain `update.min_free_kib` and
  `update.install_seconds`, the per-device figures of the index (ENK-RLS-006).
- The image build from the lock (ENK-BUILD-008, 010; ENK-RLS-002): `openwrt/build-image.sh`
  fetches each locked package by its path (from `ENKRNETT_MIRROR`, the project's pinned mirror,
  or the download tree), checks the hash, gives the Image Builder the files and an empty feeds
  list, applies the carried patches to the files of the locked packages (each patch header names
  its installed file), places the `from-image` witness, names the images per ENK-RLS-002, and with
  `--twice` builds again and compares. Both Tier 1 profiles and the x86-64 test profile are
  bit-identical between two builds.
- The double builds (ENK-BUILD-009): `openwrt/build-mips.sh --twice` builds the daemon again into
  another target directory and fails on a difference; CI's `ui` job builds the bundle twice and
  compares. Both identical.
- The release directory (ENK-BUILD-012): `openwrt/release-assemble.sh VERSION` checks the four
  version fields, collects the images, `SHA256SUMS`/`SHA512SUMS`, the lock with its own sums, the
  build report for the release notes, the written source offer (OpenWrt's tag and revision, the
  feed commits, the carried patches, Enkrnett's source), and with `--sequence` the index for the
  signing host. CI's new `images` job builds the Tier 1 images twice, assembles and uploads
  `enkrnett-release-unsigned`; the key stays on the signing host (ENK-RLS-007).
- `tests/vm/successor.sh` builds the x86-64 successor images through `openwrt/build-image.sh`
  from the lock, so the VM chain installs release-built images on every push. Every Image
  Builder now lives under `openwrt/.cache/ib/<target>/` (`openwrt/lib.sh`, shared by the scripts).
- ENK-BASE-005: `crates/enkrnettd/tests/fixtures/stored/{0.1.0,0.2.0}/state.json`, written by the
  daemons of the two tags, read, served and rewritten by today's daemon in
  `tests/stored_data.rs`; a release page without a fixture fails the test.
- Daemon, package and Tier 1 images byte-identical between the workstation and the GitHub
  runner on the same commit. The daemon's real squashfs figure is 300 KiB of 300 (CI's
  `mksquashfs`; the xz estimate had said 285): the budget holds with at most 4 KiB to spare.
- `docs/src/contributors/releasing.md` (the procedure), `docs/src/design/build.md` §7;
  `docs/src/verification/m3.md` stage 3.

### Milestone M3, stage 2 (2026-11-26)

- Installing an update (chapter 11 §3, ENK-UPD-002 to 007). `update/install`, on the owner's
  *Update* after the details screen's own step (version, minutes, the Wi-Fi stops, cannot be
  undone from here; the baseline sentence of ENK-UPD-007): the checks of ENK-UPD-003 one by one,
  `400 cannot_update` with the reason before anything is written; the image downloaded to RAM
  through the platform's downloader and bounded by the size the index states; the SHA-512 and the
  signify signature verified in one pass over the file (`signify::Verifier`); the update marker
  on flash; the hand-over to the platform's root side; `409 update_in_progress` for every other
  POST meanwhile (ENK-API-012).
- The root side: `/usr/libexec/enkrnett/install`, run by procd on the daemon's `enkrnett.install`
  event (a raw trigger of the init script). It copies the image out of the daemon's reach,
  verifies the signature again with `usign` against the keys the image carries, runs
  `sysupgrade -T`, then `sysupgrade` with the configuration kept (`/etc/config`, `/etc/enkrnett`;
  sessions and the log live in RAM, ENK-UPD-004). The package depends on `usign`.
- The first boot after an update (ENK-UPD-005): the marker says `installing`; the running version,
  the carried owner and the start-up health check decide `done` or `failed` with the reason, kept
  as `update.last_result` until the next attempt. A broken successor keeps serving the carried
  configuration and says so.
- `install_failed`, a `last_result.reason` the accepted API lacks (the root side did not flash
  the verified image, or it did not take): a deviation recorded for the next amendment RFC.
- `tests/vm/update-install.sh` and `tests/vm/successor.sh`: two successor releases built from
  this source (`ENKRNETT_VERSION` at build time), packaged and put into x86-64 images by the
  pinned Image Builder, signed and served over HTTPS; installed through the daemon, the helper and
  `sysupgrade` on the VM; the device back with its owner and settings; the broken successor's
  report; the root side refusing a stranger's signature; the disk put back afterwards. In the
  chain.
- The browser harness serves a signed index to the mock from disk; `update.spec.ts` walks the
  step and the install to "Installing…".
- MIPS daemon 285 KiB of 300 (274 after stage 1).
- `docs/src/design/daemon.md` §6c, `platform.md` §11, `release.md` §5; `docs/src/verification/m3.md`.

### Milestone M3, stage 1 (2026-11-26)

- The release index and its signature (chapter 11 §1–2): `enkrnett_model::signify` reads and
  verifies OpenBSD signify's key and signature formats with Ed25519 (`ed25519-compact`);
  `enkrnett_model::release` is the index: format, sequence, one entry per profile and baseline,
  the file name of ENK-RLS-002, and the offer rule (the highest newer release for this profile
  from the installed baseline).
- The daemon checks the index by itself at most once a day and on `update/check`, through
  `/bin/uclient-fetch` over HTTPS (RFC 016 §9), verifies the signature against the keys the image
  carries (`/usr/share/enkrnett/keys`, the current and the designated next, ENK-RLS-005), keeps
  the highest accepted sequence on flash and refuses a lower one (ENK-UPD-009), and offers only a
  release newer than the installed one. A package install or a development build does not look
  for updates until the owner turns the checks on (ENK-UPD-008); the switch rides on
  `update/check` as `{"enable_checks": true}`, a deviation to record (no resource for it in the
  accepted API). The details screen offers it.
- The index URL is `option update_index` of `/etc/config/enkrnett`, passed by the init script;
  the shipped default is a provisional GitHub releases path until the owner decides the host.
- `crates/enkrnett-release`: the release tool (`keygen`, `sign`, `verify`, `sha512`, `index`),
  signify-compatible; `openwrt/release-keys/` holds the public keys an image carries (none yet).
- The downloader is started with `posix_spawn` through `libc` rather than `std::process`, which
  kept 13 KiB of the budget: the MIPS daemon is 274 KiB of 300 (240 at 0.2.0).
- `tests/vm/update-check.sh`: a signed index served from the host over HTTPS with a test
  authority; the offer, every refusal, the sequence on flash, on the real platform. In the chain.
- `docs/src/design/release.md`; `docs/src/design/daemon.md` §6b; `docs/src/verification/m3.md`.

### After 0.2.0

- The VM launcher pins the SMBIOS vendor and product string: Ubuntu's QEMU build names the machine
  after the Ubuntu release, which gave OpenWrt another board name on the GitHub runner and left the
  daemon without a profile (the first `vm` run in CI). `tests/vm/claim.sh` says so plainly when
  the daemon does not start; `run-all.sh` keeps the daemon log after a failed foundation step.

## 0.2.0 (2026-11-26)

The wrap of milestone M2, *Guest Wi-Fi* (ROADMAP), on top of 0.1.0. Tagged 2026-11-26 on the
owner's authorization, with the Tier 1 device checks (the exit of M1) still open; they carry into
M3. Release notes: `docs/src/releases/0.2.0.md`.

### Milestone M2, stage 4 (2026-11-26)

- The x86-64 OpenWrt test VM is built from the pinned Image Builder by `tests/vm/image.sh`
  (launcher in `tests/vm/launcher`, first-boot overlay in `tests/vm/files`); the private
  measurement VM is no longer needed to run the VM tests.
- `tests/vm/run-all.sh` runs the chain (prepare, package, claim, isolation, drift, renumber,
  contract); `.github/workflows/vm.yml` runs it on every push to `main`, on request, and weekly,
  with KVM on the standard runners.
- CI fails on an uncovered 0.1.0 or 0.2.0 requirement (`tests/traceability.py`); the remaining
  0.2.0 requirements carry inspection notes, with the open ones named as open.
- The health check no longer counts the daemon's own listener on the address it leaves as exposed
  (the move waited its full 60 s before).
- The daemon asks procd for its restart with a `config.change` event for its own package (the init
  script's reload trigger), not through the `rc` object, which belongs to rpcd and is absent on a
  minimal image; the ACL lost its `rc` entry.
- The browser harness in CI: the Japanese setup test left a claim session open, which failed every
  WebKit test that followed since stage 2; it finishes its claim now.
- Version 0.2.0 in the workspace, the UI, the package.

### Milestone M2, stage 3 (2026-11-26)

- `status` answers from readings at most 2 s old (ENK-API-027); `checked_ago` is their age.
- Drift (ENK-CFG-014): the owned sections compared with what the daemon last wrote at start-up,
  every 10 minutes, and while the status is watched at most every 30 s; reported, never
  overwritten; *restore* is the ordinary change. At start-up without a marker the device is
  adopted as the boot left it (`Platform::adopt_settings`); a restore cut short by a restart is
  completed by the start-up apply and reported settled.
- A file handed back to root by a root-side `uci commit` makes the daemon ask procd to restart
  its service (`PlatformError::NotOurs`, a `config.change` event on procd's `service` object); the init script restores ownership
  at every start.
- Renumbering (ENK-TOPO-003, 006, `enkrnett_model::subnet`): only the colliding network moves,
  never back; the uplink is checked every 10 s. The daemon binds the management address the
  platform reports; when it moved, the daemon asks for a service restart. The health check of
  that move does not count the daemon's own listener on the address it is leaving as exposed.
- `tests/effective.rs` (6 tests); `tests/vm/drift.sh` and `tests/vm/renumber.sh`.

### Milestone M2, stage 2 (2026-11-26)

- Separation read back from the kernel in the health check (ENK-CFG-026): the `isolated` flag of
  every port of the guest and management bridges (ENK-CFG-023), a DNS listener on each network's
  address (ENK-CFG-024), the daemon's listener on the management address only (ENK-CFG-025);
  the hardening report counts every bound socket (ENK-HARD-005).
- Package: `/etc/hotplug.d/net/50-enkrnett-isolate` sets the bridge-port isolation flag as root
  when an interface appears (OpenWrt 25.12 gives it to the first BSS only, M-8 row 6); the init
  script sets it at every start.
- `tests/vm/isolation.sh`: the rows of measurement M-8 against the daemon's own configuration,
  with four simulated stations in network namespaces, plus the client counts of chapter 7 and
  the connection line.

### Milestone M2, stage 1 (2026-11-26)

- The transaction of chapter 6 §4 (`crates/enkrnettd/src/app/change.rs`): a change is accepted
  as *applying* with its marker already on flash, applied and verified on its own thread, and
  settled, or undone with its reason. A change of the management network's name or key
  disconnects the management stations after the health check and waits up to 600 s for the
  owner's first request on a new connection; a marker found at start-up is reported as undone by
  the restart. Restore keeps the version.
- `Platform::disconnect_management_stations` (hostapd `del_client`); the accept instant on every
  request; the UI's change display polls every second until the change is decided.
- `pending.json` in the state directory; `tests/changes.rs` (9 tests); `tests/vm/claim.sh` drives
  the packaged service and both kinds of change on the VM.

## 0.1.0 (2026-11-26)

Milestone M1, *Skeleton*: daemon and management UI together on the virtual machine; claim, login,
changes, Erase. Tagged 2026-11-26 on the owner's authorization at the stage 5 commit (`f089392`),
with the Tier 1 device part of the exit condition open. Release notes:
`docs/src/releases/0.1.0.md`.

### Milestone M1, stage 5 (2026-11-26)

- The package installed through `apk` and started by `procd` on the x86-64 VM; the x86/64 SDK
  pinned in the lock as the test vehicle (`tests/vm/package.sh`).
- Port 80 reaches the daemon by an owned firewall redirect to port 8080; the capabilities file is
  gone (`procd` grants capabilities only inside a `ujail`).
- The init script restores ownership of the four owned files at every start.
- `retry_in` rounds up; `Retry-After` agrees with it.
- `tests/vm/contract.sh`: the RFC 003 and RFC 004 checks against the real platform, through Erase
  and a real boot: 51 checks green (`docs/src/verification/m1.md`).
- `tests/traceability.py` and `docs/src/verification/`: requirement-to-test traceability.
- MIPS: std built with `optimize_for_size`, without backtraces, without panic locations, to keep
  the daemon inside its 300 KiB budget with the platform code.

### Milestone M1, stage 4 (2026-11-26)

- Measurement M-5b: the daemon's own user commits the owned configuration directly; no root
  helper for configuration (`docs/src/measurements/m-5b-non-root-commit.md`).
- `enkrnett-model`: the UCI file format (`uci`), the translation from state to owned sections
  (`plan`), `board_names` in device profiles; the `x86_64_generic` VM profile.
- `enkrnettd`: the OpenWrt platform (UCI files 0600 by temp-and-rename, ubus reload and events,
  health check by SSID, readings, hardening report, erase marker, reboot); profile selection by
  board name; the listener binds after the platform is up.
- Package: `boot()` acts on the erase marker; LED attribute hand-over; profiles installed.
- `openwrt/patches/0001-…`: the carried Transition Mode fix with its header (ENK-REPO-002).
- `tests/vm/`: the claim end to end on the OpenWrt VM, observed green on 2026-11-26.

### Milestone M1, stage 3 (2026-11-25)

- `openwrt/build-mips.sh`: the daemon for `mipsel_24kc` from the pinned SDK toolchain and
  `nightly-2026-11-21`; the budgets of chapter 8 measured and enforced (daemon 256 KiB of 300,
  UI 49 of 150, extras 7 of 50, xz estimate; CI measures the real squashfs figure).
- `openwrt/lock/inputs.json`: pinned SDK and Image Builder inputs with checksums, the nightly,
  the squashfs settings.
- `openwrt/package/enkrnett/`: the OpenWrt package (`procd`, non-root user,
  `ubusd` ACL, UCI config, first-boot script); built as `enkrnett-0.1.0-r1.apk` in the SDK.
- `openwrt/profiles/`: device profiles for ELECOM WRC-1167GST2 and Buffalo WCR-1166DS, schema
  and rules in `enkrnett_model::profile` with tests.
- `enkrnettd`: default state directory `/etc/enkrnett` (the overlay).
- CI: the `mips` job.

### Milestone M1, stage 2 (2026-11-25)

- `ui/`: the management UI, every screen of chapter 10 in English and Japanese; catalogues as
  JSON; the word check and the size check in the build (44 KiB compressed against 150 KiB).
- `tests/browser/`: the Playwright harness, 14 tests at phone size against `enkrnettd --mock`.
- `enkrnettd`: UI files are no longer bounded by the 64 KiB API body limit; `enkrnett-ubus` is
  not a dependency of the daemon until the OpenWrt platform uses it.
- One Japanese sentence of RFC 017 §5.6 corrected by the word check (see `docs/src/design/ui.md` §5).
- CI: UI and browser-harness jobs.

### Milestone M1, stage 1 (2026-10-31)

- Workspace of RFC 013 §5: `enkrnett-model`, `enkrnett-ubus`, `enkrnettd`.
- `enkrnettd`: every resource of the management API against a mock platform; HTTP/1.1 server
  with the limits and headers of chapter 9; ownership state machine; sessions; argon2id; login
  delay; persistence with format version; Erase; static files.
- Tests: unit tests per module, API contract tests over TCP (chapters 3, 4, 9).
- Docs: daemon internal design; building and testing. Example: `claim-flow.sh`.
- Not yet: the UI, the OpenWrt platform, update install, MIPS build.

### Design baseline, 2026-10-31

- RFC 001 to 017 accepted: scope and vocabulary, topology and separation, management channel,
  ownership and sessions, devices and baseline, measurements, threat model, OpenBSD position,
  vocabulary amendment, footprint budgets, settings and changes, guest modes and status, build
  and update, platform findings, management API, amendments from the full review, operator UI.
- Consolidated specification published in `docs/src/specification/` (241 requirements, 44
  threats, 14 remaining risks).
- Measurements M-2, M-5, M-6 complete; M-3 and M-8 part 1 complete on virtual machines.
- No code beyond the Cargo scaffold. Nothing is installable.
