# tests

| Folder | Content |
|---|---|
| `browser/` | The browser harness: Playwright drives the built UI from a private address against `enkrnettd --mock`, at phone size in Chromium and WebKit (chapter 14 §2) |
| `vm/` | `prepare.sh` and `claim.sh`: the claim end to end on the x86-64 OpenWrt VM with the real platform (`docs/src/design/platform.md` §6). The VM image is private measurement material. |

The API contract tests live with the daemon: `crates/enkrnettd/tests/`.
