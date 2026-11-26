# openwrt

| Path | Content |
|---|---|
| `lib.sh` | Shared by the scripts here and in `tests/vm`: the lock's values, archives fetched by URL and checked against the lock's SHA-256 (`openwrt/.cache/ib/<target>/`, `sdk/`, `mirror/`), the profiles |
| `build-mips.sh` | Builds `enkrnettd` for `mipsel_24kc` with the pinned SDK toolchain and Rust nightly, measures the budgets of chapter 8, with `--package` builds the OpenWrt package, with `--twice` builds the daemon again and compares (ENK-BUILD-009) |
| `lock-packages.sh` | Writes `lock/packages/<profile>.json`: every OpenWrt package the profile's image installs, with version, path and SHA-256 (ENK-BUILD-007); run on purpose, reviewed as a diff |
| `build-image.sh` | Builds a profile's images from the lock: the pinned Image Builder, the locked packages and no live feed (ENK-BUILD-008), the carried patches applied, the `from-image` witness, the names of ENK-RLS-002; `--twice` compares two builds (ENK-BUILD-010) |
| `release-assemble.sh` | The release directory beside the images: hashes, the lock, the build report, the source offer, the index to sign (ENK-BUILD-012); refuses disagreeing versions and live-feed images |
| `lock/inputs.json` | The pinned inputs: SDK and Image Builder URLs with SHA-256 per target, the release's download tree, the Rust nightly, Node, the squashfs settings, the budgets, where the other locks are (ENK-BUILD-007) |
| `lock/packages/` | One package lock per profile (`lock-packages.sh`) |
| `package/enkrnett/` | The OpenWrt package: Makefile, `procd` init script, `ubusd` ACL, UCI config, first-boot script |
| `profiles/<device>/profile.json` | Device profiles (chapter 5); schema and rules in `enkrnett_model::profile` |
| `patches/` | Carried fixes, each with its upstream reference and removal condition (ENK-REPO-002); the first is commit 1789118599 for Transition Mode (ENK-CFG-004), arriving with M2 |

Design: `docs/src/design/build.md` (§7 for the release build); the procedure: `docs/src/contributors/releasing.md`.
