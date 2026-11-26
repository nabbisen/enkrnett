# tests

| Folder | Content |
|---|---|
| `browser/` | The browser harness: Playwright drives the built UI from a private address against `enkrnettd --mock`, at phone size in Chromium and WebKit (chapter 14 §2) |
| `vm/` | `package.sh`, `claim.sh`, `contract.sh`, `isolation.sh`: the package through `procd`, the claim and the changes, the RFC 003 and 004 checks, and chapter 2 with simulated stations, on the x86-64 OpenWrt VM with the real platform (`docs/src/design/platform.md`). The VM image is private measurement material. |

The API contract tests live with the daemon: `crates/enkrnettd/tests/`.
