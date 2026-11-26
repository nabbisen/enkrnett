# tests

| Folder | Content |
|---|---|
| `browser/` | The browser harness: Playwright drives the built UI from a private address against `enkrnettd --mock`, at phone size in Chromium and WebKit (chapter 14 §2) |
| `vm/` | `image.sh` builds the x86-64 OpenWrt test VM from the pinned Image Builder (`launcher/`: `vm.sh`, `vmssh`, `vmscp`, `vmsh.py`; `files/`: the first-boot overlay). `run-all.sh` runs the chain: `prepare.sh`, `package.sh`, `claim.sh`, `isolation.sh`, `drift.sh`, `update-check.sh`, `renumber.sh`, `contract.sh`: the package through `procd`, the claim and the changes, chapter 2 with simulated stations, drift and its restore, the update check against a signed index served over HTTPS, renumbering on colliding uplinks, the RFC 003 and 004 checks, on the real platform (`docs/src/design/platform.md`). CI runs it in `.github/workflows/vm.yml`. |

The API contract tests live with the daemon: `crates/enkrnettd/tests/`.
