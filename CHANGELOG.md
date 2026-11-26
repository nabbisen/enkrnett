# Changelog

Releases use semantic versioning; tags carry no prefix (RFC 013 ENK-RLS-001). The history of
decisions is in `rfcs/`.

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
