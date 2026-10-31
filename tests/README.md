# tests

| Folder | Content | Arrives |
|---|---|---|
| `browser/` | The browser harness: iOS Simulator and Android emulator load the UI from a private address against the mock daemon (chapter 14 §2) | M1 stage 2 |
| `vm/` | x86-64 and malta VM scripts and integration tests (the measurement scripts of M-5, M-6, M-8 are their seed) | M2 |

The API contract tests live with the daemon: `crates/enkrnettd/tests/`.
