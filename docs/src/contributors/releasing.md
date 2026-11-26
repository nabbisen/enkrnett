# Releasing

The release steps of chapter 11 §6, in the order they are done and with who does each: CI, a
maintainer's workstation, the signing host, the project site. Nothing in this page is for the
operator. The design is `docs/src/design/build.md` §7 and `release.md`; this page is the procedure.

| Where | Holds | Never holds |
|---|---|---|
| CI (GitHub Actions) | The lock, the toolchains it fetches by hash, the built daemon, UI, packages and images | The release key (ENK-RLS-007) |
| A workstation | The same as CI, to rebuild and compare | The release key |
| The signing host (RFC 008) | The release key; `enkrnett-release` or OpenBSD `signify` | The build; network access it does not need |
| The project site | The published index, images, signatures, lock, hashes, source offer | Anything unsigned that a device would act on |

## 1. Before the tag

1. Every requirement allocated to the release is verified by the method the book names (checklist
   item 1): `python3 tests/traceability.py --release VERSION` passes.
2. The version is written in four places and must agree: `Cargo.toml` (`[workspace.package]`),
   `openwrt/package/enkrnett/Makefile` (`PKG_VERSION`), `ui/package.json`,
   `tests/browser/package.json`. `openwrt/release-assemble.sh` refuses to assemble when they do not.
3. The lock is current (ENK-BUILD-007). It changes only on purpose:
   - `openwrt/lock/inputs.json`: the Image Builder and SDK archives with their SHA-256, the Rust
     nightly, the Node major, the squashfs settings, the budgets; edited by hand.
   - `openwrt/lock/packages/<profile>.json`: the OpenWrt packages the image of each profile
     installs, with version, path and SHA-256. Regenerated with
     `openwrt/lock-packages.sh PROFILE --apk ENKRNETT.apk` when the baseline moves or a profile's
     package list changes; the diff is reviewed like code. The script reads the live feeds this
     once; every build afterwards reads only the lock.
   - `Cargo.lock`, `ui/package-lock.json`, `openwrt/patches/*.patch`: committed with the tree.
4. The carried patches are reviewed for removal (checklist item 8): each header names its upstream
   commit and removal condition. A patch that no longer applies to the locked package stops the
   image build with that message.
5. The stored-data fixture of the previous release exists
   (`crates/enkrnettd/tests/fixtures/stored/<previous>/`, ENK-BASE-005); the test
   `stored_data.rs` fails when a release page has no fixture.
6. `CHANGELOG.md`, the release page `docs/src/releases/VERSION.md`, the threat model and the
   guides are current (items 3, 4, 8).

## 2. What CI builds on the release commit

The `ci` workflow, on every push:

| Job | Builds | Compares |
|---|---|---|
| `ui` | The UI bundle with `npm ci` from the lock, Node 24 | A second `vite build` into another directory, `diff -r` (ENK-BUILD-009) |
| `mips` | The daemon for `mipsel_24kc` with the pinned nightly and SDK, the budgets (ENK-BUILD-011), the `.apk` | A second build into another target directory, `cmp` (ENK-BUILD-009); a difference fails the job |
| `images` | Every Tier 1 profile from the lock: `openwrt/build-image.sh PROFILE --apk … --twice` | The two images of each kind byte for byte (ENK-BUILD-010); the result goes into the report, a difference does not fail the job |
| `images` | `openwrt/release-assemble.sh VERSION`: the release directory | The four version fields |

The artifact `enkrnett-release-unsigned` is the release directory without signatures:

| File | Content |
|---|---|
| `enkrnett-VERSION-BASELINE-PROFILE-sysupgrade.bin`, `…-factory.bin` | The images, named per ENK-RLS-002; the vendor kind from the profile |
| `enkrnett-VERSION-BASELINE-PROFILE.manifest`, `….build.json` | The package list of each image; sizes, hashes, the comparison result |
| `SHA256SUMS`, `SHA512SUMS` | Of every image |
| `lock/` | `inputs.json`, `packages/*.json`, `Cargo.lock`, `ui-package-lock.json`, `patches/`, with their own `SHA256SUMS` |
| `build-report.md` | The table the release notes copy: image, size, hash, built twice identical or not |
| `source-offer.md` | The written offer of source (ENK-BUILD-012): OpenWrt's tag and revision, the feed commits, where the packages came from, the carried patches, Enkrnett's own source |

The `vm` workflow builds the x86-64 successor images the same way (from the lock, through
`tests/vm/successor.sh`) and installs them on the test VM, so the release build path is
exercised end to end on every push, on the test vehicle.

## 3. Rebuilding elsewhere

Anyone with the repository at the tag can rebuild and compare against CI's hashes:

```sh
openwrt/build-mips.sh --package --twice                    # the daemon and the .apk
openwrt/build-image.sh elecom_wrc-1167gst2 --apk openwrt/out/enkrnett-VERSION-r1.apk --twice
openwrt/build-image.sh buffalo_wcr-1166ds   --apk openwrt/out/enkrnett-VERSION-r1.apk --twice
openwrt/release-assemble.sh VERSION
diff openwrt/out/release/VERSION/SHA256SUMS <the published SHA256SUMS>
```

Compare only builds of the same commit: on 2026-11-27 the workstation and the runner gave the
same bytes for the daemon, the package and every Tier 1 image, and the two comparisons that
said otherwise had a stale build on one side. The packages come from the release's download tree by the paths the lock records, or from the
project's pinned mirror when `ENKRNETT_MIRROR` names it (RFC 013 §6); the SHA-256 of every file
is checked against the lock either way, and the Image Builder is given the files and an empty
feeds list, so it cannot read a live feed (ENK-BUILD-008). The mirror itself is the tree
`openwrt/.cache/mirror/` that a build leaves behind: upload it as it is.

## 4. On the signing host

With the release directory copied over (a USB stick is fine; the host needs no network):

```sh
enkrnett-release verify index.json --key release.pub     # only if an index was signed before: nothing stale
for f in *.bin; do enkrnett-release sign release.sec "$f"; done     # -> IMAGE.sig (ENK-RLS-004)
enkrnett-release index --sequence N --out index.json \
  --release elecom_wrc-1167gst2 VERSION 25.12 25.12 enkrnett-VERSION-25.12-elecom_wrc-1167gst2-sysupgrade.bin 32768 https://…/releases/VERSION 180 \
  --release buffalo_wcr-1166ds   VERSION 25.12 25.12 enkrnett-VERSION-25.12-buffalo_wcr-1166ds-sysupgrade.bin   16384 https://…/releases/VERSION 240
enkrnett-release sign release.sec index.json             # -> index.json.sig
```

`openwrt/release-assemble.sh VERSION --sequence N --notes-url URL` writes the same `index.json`
on the workstation from the profiles' `update` figures, so that the signing host only signs; the
sequence is one more than the last published index (ENK-UPD-009), and an upgrade that changes the
baseline sets `from_baseline` to the previous one. OpenBSD `signify -S -s release.sec -m FILE`
produces the same signatures.

The keys (ENK-RLS-005): the image carries the public keys of `openwrt/release-keys/`, at most two.
To rotate, the release before the switch ships the new key as the designated next one; the
release that switches is signed by it and ships a further next key, and the old key leaves in
the same release. The key is made on this host (`enkrnett-release keygen`) and only its `.pub`
is committed. **No key exists yet (2026-11-27)**: the first key is the owner's act, and until
then nothing can be published that a device would accept.

## 5. Publishing

1. The signed files go to the project site (checklist item 6): the whole release directory plus
   every `.sig`, under a path the devices' `update_index` setting reaches; `index.json` and
   `index.json.sig` beside the images, since a device fetches `<index directory>/<file>`. The
   baseline's end of life is at least six months away (ENK-BASE-002).
2. The release notes (`docs/src/releases/VERSION.md`) state the build report's result: whether
   the images were bit-identical between the two builds (ENK-BUILD-010), the figures of chapter 8,
   and what is open.
3. The tag `VERSION` (no prefix, ENK-RLS-001) is made on the owner's authorization at the commit
   CI built.
4. After the tag: the stored-data fixture of this release is made for the next one
   (`crates/enkrnettd/tests/fixtures/stored/README.md` says how).

## 6. What exists and what waits for the owner (2026-11-27)

| Step | State |
|---|---|
| The lock, the mirror path, the image build, the double builds, the report, the release directory, the index draft | In the repository and in CI |
| The release key | Not made; the owner's act on the signing host |
| The project site, the mirror host, the index URL in `/etc/config/enkrnett` | Open question 2 of RFC 013; the shipped URL is provisional |
| The Tier 1 images on a unit | Built from the lock in CI and bit-identical; never flashed (the owner's routers) |
