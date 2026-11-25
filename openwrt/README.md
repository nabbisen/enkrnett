# openwrt

| Path | Content |
|---|---|
| `build-mips.sh` | Builds `enkrnettd` for `mipsel_24kc` with the pinned SDK toolchain and Rust nightly, measures the budgets of chapter 8, and (with `--package`) builds the OpenWrt package |
| `lock/inputs.json` | The pinned inputs: SDK and Image Builder URLs with SHA-256 per target, the Rust nightly, the squashfs settings, the budgets (ENK-BUILD-007) |
| `package/enkrnett/` | The OpenWrt package: Makefile, `procd` init script, `ubusd` ACL, capabilities, UCI config, first-boot script |
| `profiles/<device>/profile.json` | Device profiles (chapter 5); schema and rules in `enkrnett_model::profile` |
| `patches/` | Carried fixes, each with its upstream reference and removal condition (ENK-REPO-002); the first is commit 1789118599 for Transition Mode (ENK-CFG-004), arriving with M2 |

Design: `docs/src/design/build.md`.
