# Daemon internal design (milestone M1, stage 1)

The internal design of `enkrnettd` as built in the first implementation stage: what exists, how
it is put together, and what is deliberately left for the next stages. The external design is
the specification (chapters 3, 4, 6, 9); this document follows it and adds nothing to it.

## 1. What this stage delivers

| Delivered | Not yet (stage, milestone) |
|---|---|
| The workspace of chapter 11 §4: `enkrnett-model`, `enkrnett-ubus`, `enkrnettd` | The UI (`ui/`, M1 stage 2) and the browser harness (`tests/browser/`) |
| Every resource of chapter 9 §3, against the mock platform | The OpenWrt platform: UCI, ubus, sysfs, the transaction engine with the pending marker and confirmation by join (chapter 6; M2) |
| HTTP/1.1 server with the limits and headers of chapter 9 | Update download, verification, install (chapter 11; M3) |
| Ownership state machine with timers; sessions; login delay; argon2id | The OpenWrt package, init script, ACL (`openwrt/package`; M1 stage 3) |
| Persistence with format version; Erase | MIPS build and the squashfs size check in CI (M1 stage 3) |
| Unit tests per module; API contract tests over TCP; mock platform | Device tests; measurements that need hardware |

## 2. Crates

```text
crates/
  enkrnett-model   settings document, validation, password rules, key and id forms, API shapes,
                   error codes. Pure: no I/O, no time, no randomness.
  enkrnett-ubus    the libubus wire format (codec, pure, tested against vectors) and a blocking
                   client (connect, lookup, call). Unused by the daemon until the OpenWrt
                   platform exists.
  enkrnettd        the daemon: HTTP server, API, application, ownership, sessions, auth, store,
                   platform trait and mock, static files, logging.
```

`enkrnettd` is a library plus a thin binary, so that the integration tests start it in-process on
an ephemeral port.

## 3. Modules of `enkrnettd`

| Module | Responsibility | Specification |
|---|---|---|
| `http` | Accept loop, four workers over a rendezvous channel, listen backlog 16, `Connection: close` | ENK-FOOT-012, ENK-SEC-019 |
| `http::request` | Reads one request within the deadline; 8 KiB head, 16 KiB body; HTTP/1.1 only; no chunking, `Expect`, or compression | ENK-SEC-017, 018, 020 |
| `http::response` | Status, headers, body; the six security headers on every response; `no-store` on JSON; body ≤ 64 KiB | ENK-API-016, 025; ENK-SEC-022 |
| `api` | `Host` check (421), path prefix `/api/1/`, exact routes, `GET`/`POST` only, content type (415), strict JSON, typed body with serde errors mapped to field codes, dispatch | ENK-SEC-009; ENK-API-001 to 005, 010, 013, 018 |
| `api::json` | A serde visitor that refuses duplicate keys, nesting over 8, non-integers | ENK-SEC-020 |
| `app` | Every operation of chapter 9 §3; authorization order (state, then session); the change runner; deferred actions | chapters 4, 9 |
| `ownership` | `UNCLAIMED → SETUP → PENDING → CLAIMED`, the 30-minute window with one 15-minute extension, the 10-minute pending limit, Erase | ENK-OWN-001 to 003, 008 |
| `session` | Four sessions, 15 min idle (5 for a claim session), 12 h life, constant-time token compare | ENK-AUTH-002 to 006; ENK-OWN-005 |
| `auth` | argon2id (profile parameters) and the login delay: 5 failures, 30 s doubling, 15 min cap | ENK-OWN-011, 012; ENK-FOOT-017 |
| `store` | `state.json` with `format`, `device_id`, `owner`, written by temp-and-rename; erase | ENK-BASE-005; ENK-CFG-006; ENK-OWN-018, 020 |
| `platform` | The trait and `MockPlatform` | chapter 6 (the implementation), RFC 014 |
| `static_files` | Entry page `no-store`; content-hashed files immutable; client routes fall back to the entry page; traversal refused | ENK-API-024, 026; ENK-MGMT-005 |
| `random`, `clock`, `log`, `config` | OS randomness; monotonic clock with a mock; one-line events without secrets; configuration | ENK-AUTH-004; ENK-API-028 to 030 |

## 4. Data flow of a request

```text
accept ─► worker ─► read_request (limits, deadline)
                 ─► Api::handle: tick timers · Host check · /api/ prefix · route · method
                    ─► POST: Content-Type · strict JSON · update-in-progress rule · typed body
                    ─► App::<operation>: lock → state check → session check → work → unlock
                        (argon2 hashing and verification happen outside the lock)
                 ─► Response (security headers, Connection: close) ─► shutdown
```

Actions that end the client's own network (confirm, restart, erase) answer first and act after
`act_delay` (2 s in a release, 0 in tests) on a short-lived thread (ENK-API-015).

## 5. The ownership state and where it lives

| Fact | Where | Survives a restart |
|---|---|---|
| Owner record (password hash, management name and key, settings) | `state.json` | Yes: a device with an owner starts `CLAIMED` |
| Pending claim (confirmed, not final) | Memory | No: a restart in `PENDING` returns to `SETUP` (ENK-OWN-008) |
| Held proposal (checked, not confirmed) | Memory, with the claim session | No (ENK-OWN-007: nothing persists before confirm) |
| Sessions, login delay | Memory | No (ENK-AUTH-003, ENK-OWN-012) |
| Setup window | Memory, from the monotonic clock | No: power opens a new window (ENK-OWN-001) |

At start and after `do_restart`, the application tells the platform what the state requires:
`enter_setup` without an owner, `apply_settings` with one.

## 6. The change runner in this stage

`App::run_change` is the synchronous stand-in for the transaction of chapter 6 §4. It numbers the
change, asks the platform to apply and verify, and either settles (version + 1, store saved) or
undoes (the previous settings are re-applied; `reason` and `detail` are reported). The pending
marker on flash, the 60-second health-check window, the disconnection of management stations and
the confirmation by the first request on a new connection are the OpenWrt platform's part and
arrive with it. The `change` shape already carries `needs_confirmation` truthfully, so the UI can
be built against the final contract.

## 7. Dependencies (ENK-FOOT-016)

| Crate | Purpose | Why not written here |
|---|---|---|
| `serde`, `serde_json` | Shapes to and from JSON | Correctness and size are better than anything hand-written; measured in M-2 |
| `argon2` (brings `password-hash`, `blake2`) | argon2id | A password hash is not something to write oneself |
| `rand_core` with `getrandom` | OS randomness | Thin wrapper over `getrandom(2)` |
| `libc` | `listen(2)` to set the backlog | Already in the tree through `getrandom` |

Everything else is `std`: the HTTP server, base64url, hex, the ubus codec. The x86-64 release
binary is recorded by CI for the trend; the budget is the MIPS squashfs figure (ENK-FOOT-001),
measured when the MIPS build joins CI.

## 8. Tests

| Where | What |
|---|---|
| `crates/*/src/**/tests.rs` | Unit tests beside each module (project rule: separate files) |
| `crates/enkrnettd/tests/api_contract.rs` | Conventions, headers, `Host`, strict JSON, authorization order, error shape |
| `crates/enkrnettd/tests/claim_flow.rs` | The claim end to end, timers with a mock clock, sessions, settings changes, key replacement, password change, Erase, restart |
| `crates/enkrnettd/tests/limits.rs` | Oversized requests, HTTP version and features, connection close, slow client, a burst of 24 connections |

Every test names the requirements it verifies in its file header or function name (chapter 14
rule 1).

## 9. Running it

```sh
cargo run -q -p enkrnettd -- --mock --listen 127.0.0.1:8080 --state-dir /tmp/enkrnett-demo --act-delay-ms 0
sh crates/enkrnettd/examples/claim-flow.sh
```
