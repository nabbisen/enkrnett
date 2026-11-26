# Building and testing

For contributors. The operator never does any of this.

## Requirements

| Tool | Version |
|---|---|
| Rust | 1.85 or later (edition 2024); the workspace pins nothing newer |
| mdBook | any recent, for this book |
| OpenWrt SDK 25.12.5 | Fetched and verified by `openwrt/build-mips.sh` from the lock; the Image Builder follows in stage 4 |

## Build and test

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

The integration tests start the daemon in-process on an ephemeral port against the mock platform
and drive it over TCP; they take about two seconds.

## Run the daemon against the mock platform

```sh
cargo run -q -p enkrnettd -- --mock --listen 127.0.0.1:8080 --state-dir /tmp/enkrnett-demo --act-delay-ms 0
curl -s http://127.0.0.1:8080/api/1/
```

`examples/claim-flow.sh` in the daemon crate walks the whole claim with `curl`. The state
directory holds `state.json`; delete it to start unclaimed again, or use the API's `erase`.

## The UI

```sh
cd ui
npm ci
npm run wordcheck   # the forbidden-word check on both catalogues
npm run check       # svelte-check
npm test            # unit tests (names, catalogues, word check)
npm run build       # word check, build, size check against ENK-FOOT-002
```

Then serve it from the daemon: `cargo run -q -p enkrnettd -- --mock --ui-dir ui/dist --act-delay-ms 0`
and open `http://127.0.0.1:8080/`. `npm run dev` in `ui/` proxies `/api` to a daemon on port
8080 for live editing.

## The browser harness

```sh
cd tests/browser
npm ci
npx playwright install chromium webkit   # once; CI adds --with-deps
npm test                                 # starts enkrnettd --mock itself on port 18080
```

The UI must be built first (`ui/dist`). `ENKRNETTD=/path/to/enkrnettd` points at a prebuilt
daemon; otherwise `cargo build -p enkrnettd` runs.

## Build for the device (MIPS)

```sh
openwrt/build-mips.sh             # fetch and verify the SDK, build enkrnettd for mipsel_24kc, check the budgets
openwrt/build-mips.sh --package   # also build the OpenWrt package (.apk) from the daemon and ui/dist
```

Needs the pinned nightly with `rust-src` (`rustup toolchain install nightly-2026-11-21 --component
rust-src`, the date is in `openwrt/lock/inputs.json`), `zstd`, `xz`, and `squashfs-tools` for
the real squashfs figure (without it the script estimates with xz and says so). The SDK (about
100 MB) is cached in `openwrt/.cache/`; `ENKRNETT_SDK_DIR` points at an existing one. Output:
`openwrt/out/enkrnettd`, `openwrt/out/sizes.json`, and the `.apk`.

## The claim on the OpenWrt virtual machine

```sh
rustup target add x86_64-unknown-linux-musl
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release -p enkrnettd --target x86_64-unknown-linux-musl
tests/vm/prepare.sh /path/to/vm    # user, ACL, ownership, base configuration; reboots the VM
tests/vm/claim.sh   /path/to/vm    # the claim end to end through an SSH tunnel
```

The VM directory is the measurement VM of RFC 006 (`vm.sh`, `vmssh`, `vmscp`, the disk image);
it is private material under `.git-exclude/tmp/m/vm/`. What the test checks is in
`docs/src/design/platform.md` §6.

## The package on the virtual machine

```sh
tests/vm/package.sh  /path/to/vm   # x86/64 SDK from the lock, enkrnett-*.apk, apk add on the VM, reboot
tests/vm/contract.sh /path/to/vm   # the RFC 003 and 004 checks against the procd-started service, Erase through a boot
tests/vm/isolation.sh /path/to/vm   # chapter 2 with four simulated stations in network namespaces (after claim.sh)
tests/vm/drift.sh /path/to/vm       # a hand edit with uci: drift reported, restored through a respawn (after claim.sh)
tests/vm/renumber.sh /path/to/vm    # colliding uplinks (VM_WAN_NET in the VM launcher): only the colliding network moves; ends with Erase
python3 tests/traceability.py --release 0.1.0   # the requirement-to-test table; fails on an uncovered 0.1.0 requirement
```

## Validate a settings document

```sh
echo '{"version":1,"country":"JP","guest":{"name":"カフェの無料ワイファイ","mode":"open_to_all","enabled":true},"management":{"name":"Enkrnett-3F2A"}}' \
  | cargo run -q -p enkrnett-model --example validate_settings
```

## The book

```sh
cd docs && mdbook serve
```

## Where things are

| Path | Content |
|---|---|
| `crates/enkrnett-model` | Settings, validation, API shapes (no I/O) |
| `crates/enkrnett-ubus` | ubus wire format and client |
| `crates/enkrnettd` | The daemon; `tests/` are the API contract tests |
| `ui/` | The management UI: Svelte 5 + TypeScript, two catalogues, the word check |
| `openwrt/` | `build-mips.sh`, the package, the device profiles, the lock; `patches/` arrives with M2 |
| `tests/browser` | The Playwright harness against `enkrnettd --mock` |
| `tests/vm` | The VM integration tests (M2) |
| `docs/src/specification` | What is built; `docs/src/design` how |
