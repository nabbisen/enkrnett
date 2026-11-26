# Release tooling and the index (M3 stage 1)

What exists of chapter 11 after the first stage of M3: the key and signature formats, the
release index, the daemon's check, and the tool that makes keys, signatures and indexes. The
installation of an update is stage 2; the reproducible build and the publication steps are
stage 3 (`ROADMAP.md`, M3 in detail).

## 1. Keys and signatures

The formats are OpenBSD signify's (`enkrnett_model::signify`), so a key made by either side
verifies the other's signatures:

| File | Content |
|---|---|
| `NAME.pub` | `untrusted comment: …`, then Base64 of `Ed` + 8-byte key number + 32-byte Ed25519 public key |
| `FILE.sig` | `untrusted comment: verify with NAME.pub`, then Base64 of `Ed` + key number + 64-byte signature over the file's bytes |
| `NAME.sec` | `Ed` + `BK` + rounds (0: unencrypted, as `signify -G -n` writes) + salt + checksum (SHA-512 of the secret, first 8 bytes) + key number + 64-byte secret |

The daemon reads only `.pub` and `.sig`. A passphrase-protected secret key is refused by the
tool (`Encrypted`); the signing host keeps the key offline instead (RFC 008, ENK-RLS-007).

The image carries at most two keys in `/usr/share/enkrnett/keys/` (ENK-RLS-005): the current key
and the designated next key. A release signed by the next key makes it current and ships a
further next key; the key that was current leaves in the same release. The repository holds the
public keys in `openwrt/release-keys/`; its README records that **no key exists yet** (the
owner's act on the signing host).

## 2. The index

One small JSON document (ENK-RLS-006, RFC 016 §9), published beside its signature as
`index.json` and `index.json.sig`:

```json
{ "format": 1, "sequence": 7,
  "releases": [ { "profile": "elecom_wrc-1167gst2", "version": "0.3.0",
                  "baseline": "25.12", "from_baseline": "25.12",
                  "file": "enkrnett-0.3.0-25.12-elecom_wrc-1167gst2-sysupgrade.bin",
                  "size": 6291456, "sha512": "…", "min_free_kib": 24576,
                  "notes_url": "https://…/releases/0.3.0", "install_seconds": 180 } ] }
```

| Field | Rule (`enkrnett_model::release`) |
|---|---|
| `format` | 1; another value is `bad_index` |
| `sequence` | Increases with every publication (ENK-UPD-009). A device keeps the highest it accepted and refuses a lower one (`stale_index`); an equal one is accepted |
| `profile` | The device profile identifier (`openwrt/profiles/<id>`); `[a-z0-9_-]`, at most 64 |
| `version` | `major.minor.patch`, no prefix, no suffix, no leading zeros (ENK-RLS-001) |
| `baseline`, `from_baseline` | The OpenWrt baseline the image is built on, and the one a device must run to take it: equal within a baseline, different for an upgrade, which the UI announces (ENK-UPD-007) |
| `file` | `enkrnett-<version>-<baseline>-<profile>-<kind>.bin` (ENK-RLS-002); anything else is refused |
| `size`, `sha512` | Of the image file; the hash is lower-case hex |
| `min_free_kib` | Free memory the installation needs beyond the image (ENK-UPD-003) |
| `notes_url` | `https://` only |
| `install_seconds` | What the confirmation screen tells the operator (RFC 017 §5.6) |

Limits: 64 KiB, 64 releases, an image of at most 256 MiB. Unknown fields are accepted, so a
later tool can add some without leaving older devices unable to learn about releases; the
`format` number changes only when a device would misread the document.

The device offers the highest version listed for its profile whose `from_baseline` is its
baseline and whose version is greater than the installed one; of a version offered both within
the baseline and as an upgrade, the one within wins. Nothing newer means "up to date".

## 3. The tool

`cargo build --release -p enkrnett-release` gives `target/release/enkrnett-release`:

| Command | Does |
|---|---|
| `keygen [-c COMMENT] NAME` | `NAME.sec` (mode 0600, unencrypted) and `NAME.pub` from the operating system's randomness |
| `sign KEY.sec FILE [SIGNATURE]` | A detached signature, `FILE.sig` by default |
| `verify FILE [SIGNATURE] --key KEY.pub ...` | Exit 0 when one of the keys verifies it; the device's rule |
| `sha512 FILE` | The hash the index carries |
| `index --sequence N [--out FILE] --release PROFILE VERSION BASELINE FROM_BASELINE IMAGE MIN_FREE_KIB NOTES_URL INSTALL_SECONDS ...` | The index, with `size` and `sha512` read from each image; refused on the spot if a device would refuse it |

The tool runs on the signing host for `keygen` and `sign`, and anywhere for the rest; CI holds
no key (ENK-RLS-007). It depends on the model crate, `serde_json` and `rand_core` only.

## 4. Where the index lives

`/etc/config/enkrnett`, `option update_index`, passed to the daemon by the init script. The
shipped default is provisional, a download path of the repository's GitHub releases; the
project's host is the owner's decision (RFC 013 open question 2, RFC 008) and one line to
change. With no key in the image and nothing published yet, a check from an image build answers
`bad_signature` or `unreachable`, honestly.

## 5. What the device does

`docs/src/design/daemon.md` §6b: the daily check, the request, the switch of ENK-UPD-008, the
outcomes, the log lines. `docs/src/design/platform.md`: `fetch`, `release_keys`,
`update_index_url`.

## 6. Tests

| Where | Count | Covers |
|---|---|---|
| `crates/enkrnett-model/src/signify/tests.rs` | 6 | Verify under the right key and no other, same key number with another key, round trips of the three files, an Ed25519 vector from OpenSSL in signify's layout, hostile files, passphrase-protected and corrupt secret keys, SHA-512 |
| `crates/enkrnett-model/src/release/tests.rs` | 5 | Strict versions, round trip with unknown fields, every malformed field refused by name, the offer, the upgrade |
| `crates/enkrnettd/tests/update.rs` | 9 | The daemon against signed indexes through the mock platform (§6b) |
| `crates/enkrnettd/src/platform/spawn/tests.rs` | 2 | Output captured and bounded, exit status, the fixed environment, empty input |
| `tests/vm/update-check.sh` | see `docs/src/verification/m3.md` | The real platform: `uclient-fetch` over HTTPS with a test authority, the offer, every refusal, the sequence on flash |
