# Roadmap

**State.** The M0 design baseline is accepted (RFC 001 to 017, 2026-10-31) and the specification is published. M1 stages 1 to 5 are implemented (2026-10-31 to 2026-11-26): daemon, UI, MIPS build and package, the OpenWrt platform, and the package through `procd` on the x86-64 VM with the RFC 003 and RFC 004 checks passing against the real platform (`docs/src/verification/m1.md`). What M1 still needs is the Tier 1 device in the owner's hands. RFC 006 (measurements) is part-way: M-2, M-5, M-5b, M-6 done; M-3 and M-8 half done; M-1, M-4, M-7 outstanding.
**Updated.** 2026-11-27

Order of work, not a calendar. Dates are set when milestone M0 ends, because its measurements
decide how much work the later milestones contain.

## Milestones

| Milestone | Version | Goal | Ends when |
|---|---|---|---|
| **M0 Foundations** | — | Decisions, measurements, and the specification baseline | All RFCs listed under M0 are accepted and the specification is published (met 2026-10-31). **Proposed amendment, for the owner:** the outstanding measurements move to the milestone that needs them (M-1 → M1 harness and M3 guides; M-4, M-8 part 2 → M2; M-3 part 2, M-7 → M3), so that M0 does not stay open on hardware that cannot be switched on yet. RFC 006 moves to `done/` when its last report is accepted. |
| **M1 Skeleton** | 0.1.0 | Daemon and management UI run together on the virtual machine and on the Tier 1 device. A device can be claimed and reset. First task: a browser harness in GitHub Actions (iOS Simulator, Android emulator) that loads the management UI from a private address against a mock daemon (owner decision 2026-10-29). | The tests derived from RFC 003 and RFC 004 pass |
| **M2 Guest Wi-Fi** | 0.2.0 | The operator configures the guest network. Separation and hardening are in place. The status says what is protected and what is not. | The tests derived from RFC 002 and the OWE and configuration RFCs pass |
| **M3 Installable** | 0.3.0 | A non-expert installs, sets up, updates, and resets a Tier 1 device with the guides. English now; Japanese as a future plan (owner, 2026-11-27). | The usability test passes; update and upgrade are tested. Update and upgrade: tested on the VM (2026-11-27); the usability test waits for the router and the legal basis |
| **M4 Constrained device** | 0.4.0 | An 8 MB / 64 MB device meets the budgets. | Measured on the device; the owner decides on promotion |
| **First release** | 1.0.0 | Release for operators | Release checklist below |

M3 cannot publish an image for a device sold in Japan until the legal basis is documented
(RFC 005, ENK-RF-005).

## M0 in detail

| Step | Content | Who | State |
|---|---|---|---|
| 1 | Gap analysis of the drafts | Architect | Done |
| 2 | Round 1 decisions | Owner | Done |
| 3 | RFC 001 to 006 proposed | Architect | Done |
| 4 | RFC 001 to 006 accepted | Owner | Done |
| 5 | Measurements M-6, M-5, M-2 | Development team | **Ready to start** |
| 6 | Measurement M-1, with the Pixel phone and the iPad: the seven network rows by hand; the page rows move to the M1 browser harness | Kit prepared by the architect; owner observes | **Owner, now** |
| 7 | RFC 007 Threat model accepted | Owner | Done |
| 8 | Inquiry on radio regulation | Owner | The owner will start it. It may take time. |
| 9 | Candidate routers obtained (one `standard`, one `constrained`, RFC 005 §7); a lawful basis for switching their radios on | Owner | **Owner buys the routers now; the radios wait for the legal answer** |
| 10a | M-3 part 1 (image fits) and M-8 part 1 (separation) on VMs | Architect | Done 2026-10-29 |
| 10c | M-4a: OWE with the Pixel and the iPad against the workstation's Wi-Fi card | Architect prepared the kit; owner runs it | **Owner, when convenient** (`.git-exclude/tmp/m/m4a/README.md`) |
| 10b | Measurements M-4, M-8, M-7 and the rest of M-3 | Holder of the hardware; owner | Blocked by the legal answer |
| 11 | Round 2 decisions | Owner | Done (D-08/D-10/D-12 by RFC 010–012; D-11/D-13/D-14/D-15 accepted 2026-10-29) |
| 12 | RFC 013 (build, release, update) and 014 (platform findings) | Architect; owner | Accepted 2026-10-31 |
| 12a | RFC 015 Management API | Architect; owner | Accepted 2026-10-31 |
| 12c | Review of RFC 001–015 against the owner's philosophy; RFC 016 (amendments) | Architect; owner | Done; 016 accepted 2026-10-31 |
| 12b | RFC 017 Operator UI | Architect; owner | Accepted 2026-10-31 |
| 13 | Consolidated specification in `docs/src/specification/` (13 chapters, 241 requirements, threat model, index) | Architect | Done 2026-10-31 |
| 14 | Repository layout (RFC 013 §5) | Architect | Done 2026-10-31 (first structural commit of M1) |

## M1 in detail (owner's instruction 2026-10-31: implement the first stage carefully, with tests, documentation, examples)

| Stage | Content | State |
|---|---|---|
| 1 | Workspace; `enkrnett-model`; `enkrnett-ubus` codec; `enkrnettd` with every API resource against a mock platform; contract tests; internal design; CI | Done 2026-10-31 |
| 2 | `ui/` (chapter 10) in both languages with the word check; the browser harness in GitHub Actions against `enkrnettd --mock` | Done 2026-11-25 |
| 3 | MIPS build with the OpenWrt SDK; squashfs size check against ENK-FOOT-001 to 003 in CI; the OpenWrt package skeleton; first device profile | Done 2026-11-25 |
| 4 | The OpenWrt platform: UCI and ubus, setup and management networks, the claim end to end on the x86-64 VM (M2 brings the full transaction engine) | Done 2026-11-26 |
| 5 | The package on the VM through `procd`; the erase marker through a real boot; the tests derived from RFC 003 and 004 against the real platform; requirement-to-test traceability (`tests/traceability.py`) | Done 2026-11-26 |
| 6 | On the Tier 1 device: image build with the Image Builder, boot, claim, LED, erase by `firstboot`; the 0.1.0 release | 0.1.0 tagged 2026-11-26 on the owner's authorization at the stage 5 commit; the device checks wait for hardware and carry into M3 |

## M2 in detail

| Stage | Content | State |
|---|---|---|
| 1 | The transaction of chapter 6 §4 in the daemon: marker on flash, apply on its own thread, health check, disconnection and confirmation by the next request, undo on failure, on the deadline, and at start-up; the UI polls the change; tests against the mock and the VM | Done 2026-11-26 |
| 2 | Separation and hardening on the platform (chapter 2): bridge-port isolation set and read back, one DNS/DHCP instance per network checked, the listener read-back, IPv6 dropped on the Wi-Fi networks, the firewall rules of ENK-ISO-001 to 006; hwsim stations in network namespaces on the VM (M-8 rows) | Done 2026-11-26 |
| 3 | Effective state and drift (ENK-CFG-013, 014): the 2 s status cache, the drift check every 10 minutes with *restore*; the device adopted at start-up without overwriting; the guest notice page (ENK-GUEST-001, done in M1 stage 2); renumbering on an uplink collision (ENK-TOPO-003, 006) | Done 2026-11-26 |
| 4 | VM integration for chapters 2, 6, 7 in CI; the 0.2.0 wrap: changelog, verification record, release notes | Done 2026-11-26 (`.github/workflows/vm.yml`; `docs/src/releases/0.2.0.md`). 0.2.0 tagged 2026-11-26 on the owner's authorization; the device checks carry into M3 |

## M3 in detail

The exit condition needs the Tier 1 device (the guides' usability test, M-7; the install from the
vendor firmware) and the legal basis (ENK-RF-005); what does not is built first.

| Stage | Content | State |
|---|---|---|
| 1 | The release index and the keys (chapter 11 §1–2): signify-format Ed25519 verification in the daemon, the index format with its sequence, the current and the designated next key in the image (ENK-RLS-005), the check through the platform's downloader over HTTPS, daily and on request, the switch of ENK-UPD-008, the highest accepted sequence on flash (ENK-UPD-009); the release tool (`crates/enkrnett-release`); a signed index served to the VM over HTTPS | Done 2026-11-26 (`docs/src/design/release.md`; `tests/vm/update-check.sh`) |
| 2 | Installing (chapter 11 §3, ENK-UPD-002 to 007): download to RAM, the checks of ENK-UPD-003 one by one, `sysupgrade` keeping the owner, the key, the settings and the identifier (ENK-UPD-004), the marker and the first boot's verification, health check and report (ENK-UPD-005), the screens of RFC 017 §5.6; on the VM: an image updated to the next through the daemon with settings kept, and a deliberately broken successor | Done 2026-11-26 (`docs/src/design/daemon.md` §6c, `platform.md` §11; `tests/vm/update-install.sh`) |
| 3 | Reproducible build and the release steps (chapter 11 §4, ENK-BUILD-007 to 012): the package set with versions and hashes in the lock and used by the image build, the double build compared in CI, the image comparison stated, images named per ENK-RLS-002, hashes, the signed index and the image signatures, the `from-image` witness in every image (`docs/src/design/platform.md` §11), the source offer; the stored-data test with the documents of 0.1.0 and 0.2.0 (ENK-BASE-005) | Done 2026-11-27 (`docs/src/design/build.md` §7, `docs/src/contributors/releasing.md`; `openwrt/lock/packages/`, `openwrt/build-image.sh`, `openwrt/release-assemble.sh`; the release key and the project site remain the owner's acts) |
| 4 | The guides in English (ENK-INST-001 to 005: install per Tier 1 device, set up, update, erase, recover; the expert install), the usability test plan for M-7, the 0.3.0 wrap. What needs the device or the legal basis is stated as open. **Owner's decision 2026-11-27:** the guides are required in English only now; the Japanese guides are a controlled future plan (table below) | Done 2026-11-27 (`docs/src/guides/`, `rfcs/handoffs/006-feasibility-measurements/m-7-usability-test-plan.md`, `docs/src/releases/0.3.0.md`); 0.3.0 awaits the owner's authorization to tag |

### M3 exit, honestly

Implemented and verified on the test vehicle: update and upgrade mechanics (stages 1 to 3), the
guides in English. Not met, because they need the Tier 1 router and the legal basis: the
usability test (M-7), the install from the vendor firmware, the device checks carried from M1.
0.3.0 is the development release that closes the implementable part; the release notes list the
rest as open.

## Future plans

Work the owner has decided on but not scheduled. Each row names its trigger and what accepts it,
so that it is controlled rather than forgotten.

| Plan | Decided | Trigger | Accepted when | Where it is tracked |
|---|---|---|---|---|
| The operator guides in Japanese (ENK-INST-001 names every UI language; ENK-UI-011 asks for a native reader's review) | Owner, 2026-11-27: "The guide in English is only required now. One in Japanese should be recorded and controlled as a future plan." | The English guides have been through the usability test (M-7) once, so that the translation follows text non-experts have read | The Japanese guides exist for every English one, with the display forms of chapter 1 §4 and the word check, reviewed by a native reader (ENK-UI-011); the deviation on ENK-INST-001 closed by the amendment RFC | This table; `docs/src/guides/index.md` *Other languages*; the 0.3.0 release notes *Open*; the next amendment RFC |
| The release key and the project site (RFC 008; RFC 013 open question 2) | Owner, standing | The owner's act on the signing host; the host chosen | A published, signed index a device accepts | `docs/src/contributors/releasing.md` §6; `openwrt/release-keys/README.md` |
| The Tier 1 device checks (M1 stage 6, M-3 part 2, M-4, M-7, M-8 part 2) and the legal basis for Japan (ENK-RF-005) | Owner, standing | The router in hand and the authority's answer | Chapter 5 §5.1 criteria met; the profile's `verified.on_hardware` true; the owner's promotion | `docs/src/measurements/index.md`; the release notes *Open* |

## RFC order

| Order | RFC | State | Needed for |
|---|---|---|---|
| 1 | 001 Scope, people, and vocabulary | Accepted | Everything |
| 2 | 002 Network topology and separation | Accepted | M2 |
| 3 | 003 Management channel and UI delivery | Accepted | M1 |
| 4 | 004 Ownership, credentials, and authorization | Accepted | M1 |
| 5 | 005 Devices, OpenWrt baseline, and installation | Accepted | M1, M3 |
| 6 | 006 Feasibility measurements | Accepted | Round 2 |
| 7 | 007 Threat model | Accepted | M1 |
| 8 | 009 Vocabulary amendment | Accepted | Everything |
| 9 | 010 Footprint budgets | Accepted | M1 |
| 10 | 011 Configuration model and transactions | Accepted | M2 |
| 11 | 012 Guest network modes and protection status | Accepted | M2 |
| 12 | 013 Build, release, update, and upgrade | Accepted | M3 |
| 13 | 014 Platform findings that Enkrnett must handle itself | Accepted | M2 |
| 14 | 015 Management API | Accepted | M1 |
| 15 | 016 Amendments from the review of 001–015 | Accepted | Everything |
| 16 | 017 Operator UI | Accepted | M1 |

## Release cycle

| Subject | Rule |
|---|---|
| Version numbers | Semantic versioning. Tags carry no "v" prefix. |
| Before 1.0.0 | Each milestone ends with one release. These are for development and testing, not for operators. |
| OpenWrt baseline | One per Enkrnett release. The first is 25.12. |
| Every release | Security review. Threat model updated when data flows, integrations, or authorization changed. Documentation checked against the accepted RFCs. `CHANGELOG.md` updated. |
| After 1.0.0 | Cadence is decided with the update strategy in Round 2. |

## Release checklist for 1.0.0

| # | Item |
|---|---|
| 1 | Every requirement allocated to the first release is verified by the method its RFC names. |
| 2 | At least one Tier 1 device meets RFC 005 §2. |
| 3 | The threat model is current. |
| 4 | Install, setup, update, and reset guides exist in English and Japanese, and non-experts completed them. |
| 5 | Every published file can be rebuilt from the repository and its pinned inputs. |
| 6 | The legal basis for each market in which images are published is documented. |
