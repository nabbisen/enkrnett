# RFC 013 — Build, release, update, and upgrade

**Status.** Accepted (2026-10-31)
**Tracks.** Decisions D-11, D-13, D-14 as reviewed in `11-round2-review.md` and accepted by the
owner on 2026-10-29; RFC 005 ENK-BASE-003 to 007; RFC 008 (infrastructure platform).
**Touches.** Repository layout; CI; the release checklist; the daemon (update check, update
flow, first boot after an update); the management UI; the project's hosting.

Terms are defined in [RFC 001](./001-scope-people-vocabulary.md) as amended by
[RFC 009](./009-vocabulary-amendment-operator-words.md). Devices and tiers:
[RFC 005](./005-devices-baseline-installation.md). Budgets:
[RFC 010](./010-footprint-budgets.md). Transactions:
[RFC 011](./011-configuration-model-and-transactions.md).

## Summary

| Subject | Decision |
|---|---|
| Update and upgrade | One path: a full, signed firmware image, installed from the management UI by the owner after the device has checked the signature, the device match, and its own resources; settings survive. The operator sees one word, *Update*. |
| Build | Every published file can be rebuilt from the repository and a lock of pinned, hash-recorded inputs; the daemon and the UI are bit-identical between builds; images are compared and the result is stated. |
| Repository | One Cargo workspace; `crates/`, `ui/`, `openwrt/`, `tests/`, `docs/`, `rfcs/`. |
| Infrastructure | A pinned package mirror and the signing key, on OpenBSD by preference (RFC 008). |

## Motivation

| Fact | Source |
|---|---|
| For a non-expert, "reflash manually" means "never"; the regulator recommends keeping firmware current. | Gap analysis GAP-061; F-REG-04 |
| OpenWrt's release package feeds change in place under the same release number; an Image Builder run today differs from one next month. | F-OWRT-19 |
| Most target devices have one firmware slot: there is no second image to fall back on. | Device profiles (RFC 005 §7 candidates) |
| `sysupgrade` writes a new image and can keep configuration files; the process needs the image in RAM. | OpenWrt |
| The pending marker of RFC 011 already protects the first boot after a change. | RFC 011 §4 |
| Images contain GPL-licensed components; distributing them carries a source obligation that the daemon alone does not. | Gap analysis GAP-065 |

## Design

### 1. Versions and names

| ID | Requirement | Verification |
|---|---|---|
| ENK-REL-001 | Releases shall use semantic versioning; tags carry no prefix. Each release names exactly one OpenWrt baseline (ENK-BASE-001). | Inspection |
| ENK-REL-002 | Image files shall be named `enkrnett-<version>-<baseline>-<device>-<kind>.bin`, where `<kind>` is `sysupgrade` or the vendor-install kind of the device profile. | Inspection |
| ENK-REL-003 | The status screen shall show the installed Enkrnett version and the baseline in one line, in words the operator can read back to a helper: "Enkrnett 1.2 on OpenWrt 25.12". | Inspection |

### 2. Signing and the release index

| ID | Requirement | Verification |
|---|---|---|
| ENK-REL-004 | Every published image shall be accompanied by a detached signature made with the project's release key; the release index shall be signed with the same key. | Release checklist |
| ENK-REL-005 | The image shall embed the public keys that may sign its successor: the current key and one designated next key. A release signed by the next key makes it current and designates a further next key. | Test: rotate once in CI |
| ENK-REL-006 | The release index shall be one small signed document listing, per device profile and baseline, the latest release: version, file name, size, hash, minimum free memory, release notes URL, and whether it changes the baseline. | Inspection |
| ENK-REL-007 | The signing key shall never be on a CI runner. Signing is a manual step on the project's signing host (RFC 008). | Release checklist |

Tool: OpenBSD `signify` (Ed25519, one-line format, portable implementations exist for Linux and in
Rust). Its format is small enough to verify on an 8 MB device.

### 3. Update, as the operator experiences it

```text
 status screen ──"Check for updates"──► device fetches the index (uplink) ──► "Up to date" | "Update available: Enkrnett 1.3"
                                                                                   │
                                        "Update" ──► download to RAM ──► verify signature, device, baseline, free memory
                                                                         │ ok: "Installing. The Wi-Fi stops for about two minutes." ──► sysupgrade ──► first boot: health check
                                                                         │ not ok: one sentence why; nothing written
```

| ID | Requirement | Verification |
|---|---|---|
| ENK-UPD-001 | The device shall check the release index at most once a day by itself and on request. The check shall send no identifier of the device, the operator, or the configuration; the request is the same for every device of the same profile. | Network capture |
| ENK-UPD-002 | The device shall start an update only when the owner presses *Update* in the management UI, after a screen that states the version, the baseline change if any, the duration, and that the Wi-Fi stops meanwhile. | Inspection; M-7 |
| ENK-UPD-003 | Before writing, the device shall verify: the signature of the index and of the image against an embedded key; that the image is for this device profile; that the baseline is the same or the designated next; that free memory exceeds the image size plus 16 MB; and that no transaction (RFC 011) is pending. Any failure shall stop the update with one sentence of reason, writing nothing. | Test for each condition |
| ENK-UPD-004 | The update shall carry over the owner, the management key, the desired document, and the device identifier (ENK-BASE-004), and shall not carry over logs, sessions, or snapshots. | Test |
| ENK-UPD-005 | On the first boot after an update the daemon shall verify the carried state (format versions, ENK-BASE-005), run the health check of RFC 011, and then report the update as done. If the health check fails, the device shall keep serving the last known configuration, mark the update as failed, and say so on the status screen. | Test with a deliberately broken successor |
| ENK-UPD-006 | The device shall not promise a firmware rollback it cannot perform. On single-slot devices the only way back is a new update or the recovery of the device profile; the UI shall say "this cannot be undone from here" before the update starts. | Inspection |
| ENK-UPD-007 | *Update* and *Upgrade* are the project's words (RFC 001 §4.2). The operator sees *Update* for both; when the baseline changes, the confirmation screen adds "This one takes longer and restarts the Wi-Fi twice." | Inspection |
| ENK-UPD-008 | A development build or an image installed by other means shall not be offered updates from the project's index unless the owner enables it in the details screen. | Test |

Automatic updates (option C of D-11) are not in this RFC; they need their own RFC with a
failure analysis for unattended devices.

### 4. Build and reproducibility

| ID | Requirement | Verification |
|---|---|---|
| ENK-BUILD-007 | The repository shall contain a lock of every input of a release: the Image Builder archive and its hash; the list of OpenWrt packages with versions and hashes; the Rust toolchain and the Cargo lock; the UI package lock; the carried patches (RFC 011 ENK-CFG-004). | Inspection |
| ENK-BUILD-008 | OpenWrt packages shall be taken from the project's pinned mirror, never from the live feeds, when a release is built. | CI configuration |
| ENK-BUILD-009 | Two builds of the daemon and of the UI bundle from the same lock shall be bit-identical; CI shall build twice and compare. | CI |
| ENK-BUILD-010 | Two builds of an image from the same lock should be bit-identical; CI shall compare and the release notes shall state the result. | CI; release notes |
| ENK-BUILD-011 | CI shall enforce the budgets of RFC 010 on every build. | CI |
| ENK-BUILD-012 | Every release shall publish, beside the images: the lock, the hashes, the signatures, and the written offer of source for the GPL components (the OpenWrt tag and feed revisions of the lock, and where to obtain them). | Release checklist |

### 5. Repository layout

```text
enkrnett/
  Cargo.toml              workspace
  crates/
    enkrnettd/            the daemon
    enkrnett-ubus/        the ubus client (own crate, tested alone)
    enkrnett-model/       the desired document, validation, and the UCI translation (no I/O)
  ui/                     Svelte 5 + TypeScript; built into the daemon's static files
  openwrt/
    package/enkrnett/     OpenWrt package: Makefile, init script, ACL, defaults, conffiles
    profiles/<device>/    device profile and Image Builder definition
    patches/              carried fixes, each with its upstream reference and removal condition
    lock/                 the pinned inputs of ENK-BUILD-007
  tests/
    browser/              the browser harness (GitHub Actions)
    vm/                   x86-64 and malta VM scripts; integration tests
  docs/                   mdBook
  rfcs/
  ROADMAP.md  CHANGELOG.md  README.md  NOTICE  LICENSE
```

| ID | Requirement | Verification |
|---|---|---|
| ENK-REPO-001 | The repository shall follow the layout above. The present root crate is replaced by the workspace in the first structural commit of milestone M1. | Inspection |
| ENK-REPO-002 | `openwrt/patches/` shall hold only patches with a header naming the upstream change and the condition under which the patch is removed. | Review |

### 6. Infrastructure

| Item | Decision |
|---|---|
| Pinned package mirror | A static file tree, served over HTTPS, holding exactly the packages of each lock. Preferred platform: OpenBSD `httpd` (RFC 008). |
| Signing host | Offline or single-purpose; OpenBSD `signify`. Keys backed up on paper or hardware token. |
| Release index | Static files on the project site, signed. |
| CI | GitHub Actions for builds, budgets, double builds, and the browser harness; it never holds the release key. |

### 7. Release checklist

Consolidates the lists of `ROADMAP.md`, RFC 005 §2, and RFC 010.

| # | Item |
|---|---|
| 1 | Every requirement allocated to the release is verified by the method its RFC names. |
| 2 | Each Tier 1 device: installed from the vendor firmware by the guide; updated from the previous release with settings kept (ENK-UPD-004); budgets measured (RFC 010). |
| 3 | The threat model (RFC 007) is current for this release. |
| 4 | Documentation and guides match the accepted RFCs, in every UI language. |
| 5 | The lock is committed; CI's double build passed; the image comparison result is in the notes. |
| 6 | Images, signatures, index, lock, and the source offer are published; the baseline's end of life is at least six months away (ENK-BASE-002). |
| 7 | For devices sold in Japan: the legal basis is documented (ENK-RF-005). |
| 8 | `CHANGELOG.md` updated; the carried patches reviewed for removal. |

## Consequences

| Affected | Change |
|---|---|
| D-11, D-13, D-14 | Decided by this RFC. |
| RFC 005 ENK-BASE-003 | Satisfied by §3. |
| Draft future item "automatic update system" | Option C deferred to a later RFC. |
| Repository | Restructured at the start of M1. |
| Gap analysis GAP-060, 061, 063, 065 | Closed. |

## Alternatives considered

| Alternative | Why not |
|---|---|
| Package-only updates (`apk`) for the daemon | Leaves the baseline, the kernel, and `hostapd` unpatched, which is where the security updates are; two mechanisms to explain. |
| Automatic updates by default | Unattended failure on a single-slot device (D-11 review). |
| A/B firmware slots | The hardware class does not have them. |
| Signing in CI | The key would live on a shared runner. |
| GPG for signing | Larger verifier; `signify` does one thing. |

## Open questions

| # | Question | Closed by |
|---|---|---|
| 1 | Exact `sysupgrade` conffile list and the state-migration test | M1 internal design |
| 2 | Where the project site and the mirror are hosted | Owner, before the first release |
