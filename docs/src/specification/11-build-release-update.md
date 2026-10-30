# 11. Build, release, update

From RFC 013, as amended by RFC 016 (ENK-UPD-001, ENK-UPD-009, ENK-RLS-006). Infrastructure
platform: RFC 008 (OpenBSD preferred for the mirror and the signing host).

| Subject | Decision |
|---|---|
| Update and upgrade | One path: a full, signed firmware image, installed from the management UI by the owner after the device has checked the signature, the device match, the baseline, the sequence, and its own resources; settings survive. The operator sees one word, *Update*. |
| Build | Every published file can be rebuilt from the repository and a lock of pinned, hash-recorded inputs; the daemon and the UI are bit-identical between builds; images are compared and the result is stated. |
| Repository | One Cargo workspace: `crates/`, `ui/`, `openwrt/`, `tests/`, `docs/`, `rfcs/`. |
| Infrastructure | A pinned package mirror and the signing key; the key never on CI. |

## 11.1 Versions, signing, the index

{{#include requirements/rls.md}}

Tool: OpenBSD `signify` (Ed25519). The index fields: per device profile and baseline, the latest
release's version, file name, size, hash, minimum free memory, release notes URL, whether it
changes the baseline, and the publication **sequence** (ENK-UPD-009).

## 11.2 Update, as the operator experiences it

```text
 status ──"Check for updates"──► index fetched (platform downloader, HTTPS) ──► "Up to date" | "Update available: Enkrnett 1.3"
                                                                                       │
                         "Update" ──► download to RAM ──► verify signature, device, baseline, sequence and version, free memory, no change pending
                                                          │ ok: "Installing. The Wi-Fi stops for about two minutes." ──► sysupgrade ──► first boot: health check
                                                          │ not ok: one sentence why; nothing written
```

{{#include requirements/upd.md}}

Automatic updates (option C of decision D-11) are not specified; they need their own RFC with a
failure analysis for unattended devices.

## 11.3 Build and reproducibility

{{#include requirements/build.md}}

## 11.4 Repository layout

```text
enkrnett/
  Cargo.toml              workspace
  crates/
    enkrnettd/            the daemon
    enkrnett-ubus/        the ubus client (own crate, tested alone)
    enkrnett-model/       the settings document, validation, and the UCI translation (no I/O)
  ui/                     Svelte 5 + TypeScript; built into the daemon's static files
  openwrt/
    package/enkrnett/     OpenWrt package: Makefile, init script, ACL, defaults, conffiles
    profiles/<device>/    device profile and Image Builder definition
    patches/              carried fixes, each with its upstream reference and removal condition
    lock/                 the pinned inputs of ENK-BUILD-007
  tests/
    browser/              the browser harness (GitHub Actions)
    vm/                   x86-64 and malta VM scripts; integration tests
  docs/                   mdBook (this book)
  rfcs/
  ROADMAP.md  CHANGELOG.md  README.md  NOTICE  LICENSE
```

{{#include requirements/repo.md}}

## 11.5 Infrastructure

| Item | Decision |
|---|---|
| Pinned package mirror | A static file tree, served over HTTPS, holding exactly the packages of each lock. Preferred platform: OpenBSD `httpd`. |
| Signing host | Offline or single-purpose; OpenBSD `signify`. Keys backed up on paper or hardware token. |
| Release index | Static files on the project site, signed, with a sequence number. |
| CI | GitHub Actions for builds, budgets, double builds, and the browser harness; it never holds the release key. |

## 11.6 Release checklist

| # | Item |
|---|---|
| 1 | Every requirement allocated to the release is verified by the method this book names. |
| 2 | Each Tier 1 device: installed from the vendor firmware by the guide; updated from the previous release with settings kept (ENK-UPD-004); budgets measured (chapter 8). |
| 3 | The threat model (chapter 12) is current for this release. |
| 4 | Documentation and guides match this book, in every UI language; the Japanese catalogue reviewed by a native reader (ENK-UI-011). |
| 5 | The lock is committed; CI's double build passed; the image comparison result is in the notes. |
| 6 | Images, signatures, index (with the next sequence), lock, and the source offer are published; the baseline's end of life is at least six months away (ENK-BASE-002). |
| 7 | For devices sold in Japan: the legal basis is documented (ENK-RF-005). |
| 8 | `CHANGELOG.md` updated; the carried patches reviewed for removal. |

## 11.7 Open

| # | Question | Closed by |
|---|---|---|
| 1 | Exact `sysupgrade` conffile list and the state-migration test | M1 internal design |
| 2 | Where the project site and the mirror are hosted | Owner, before the first release |
