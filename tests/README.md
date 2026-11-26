# tests

| Folder | Content |
|---|---|
| `browser/` | The browser harness: Playwright drives the built UI from a private address against `enkrnettd --mock`, at phone size in Chromium and WebKit (chapter 14 §2). The mock is given a signed release index from disk; `update.spec.ts`, the last file, walks the update's step and the install |
| `vm/` | `image.sh` builds the x86-64 OpenWrt test VM from the pinned Image Builder (`launcher/`: `vm.sh`, `vmssh`, `vmscp`, `vmsh.py`; `files/`: the first-boot overlay). `run-all.sh` runs the chain: `prepare.sh`, `package.sh`, `claim.sh`, `isolation.sh`, `drift.sh`, `update-check.sh`, `update-install.sh`, `renumber.sh`, `contract.sh`: the package through `procd`, the claim and the changes, chapter 2 with simulated stations, drift and its restore, the update check against a signed index served over HTTPS, an update installed to a successor image built by `successor.sh` and a deliberately broken one, renumbering on colliding uplinks, the RFC 003 and 004 checks, on the real platform (`docs/src/design/platform.md`). CI runs it in `.github/workflows/vm.yml`. |

The API contract tests live with the daemon: `crates/enkrnettd/tests/`.
