# tests

| Folder | Content |
|---|---|
| `browser/` | The browser harness: Playwright drives the built UI from a private address against `enkrnettd --mock`, at phone size in Chromium and WebKit (chapter 14 §2) |
| `vm/` | x86-64 and malta VM scripts and integration tests (M2; the measurement scripts of M-5, M-6, M-8 are their seed) |

The API contract tests live with the daemon: `crates/enkrnettd/tests/`.
