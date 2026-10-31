# Building and testing

For contributors. The operator never does any of this.

## Requirements

| Tool | Version |
|---|---|
| Rust | 1.85 or later (edition 2024); the workspace pins nothing newer |
| mdBook | any recent, for this book |
| OpenWrt SDK and Image Builder | Milestone M1 stage 3 onwards, for MIPS builds and images |

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
| `ui/` | The management UI (M1 stage 2) |
| `openwrt/` | Package, device profiles, carried patches, the lock (M1 stage 3 onwards) |
| `tests/browser`, `tests/vm` | The browser harness and the VM integration tests (M1 stage 2 and M2) |
| `docs/src/specification` | What is built; `docs/src/design` how |
