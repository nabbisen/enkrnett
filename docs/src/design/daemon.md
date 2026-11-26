# Daemon internal design (milestone M1, stage 1)

The internal design of `enkrnettd` as built in the first implementation stage: what exists, how
it is put together, and what is deliberately left for the next stages. The external design is
the specification (chapters 3, 4, 6, 9); this document follows it and adds nothing to it.

## 1. What this stage delivers

| Delivered | Not yet (stage, milestone) |
|---|---|
| The workspace of chapter 11 §4: `enkrnett-model`, `enkrnett-ubus`, `enkrnettd` | The UI (`ui/`, M1 stage 2) and the browser harness (`tests/browser/`) |
| Every resource of chapter 9 §3, against the mock platform; since M1 stage 4 also against OpenWrt (`docs/src/design/platform.md`); since M2 stage 1 the transaction of chapter 6 §4 (§6 below) | Drift detection (ENK-CFG-014), the isolation read-backs and the port scan in the health check (ENK-CFG-023 to 025; M2 stage 2) |
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
| Owner record (password hash, management name and key, settings) | `state.json` in `/etc/enkrnett` (the overlay) | Yes: a device with an owner starts `CLAIMED` |
| Pending claim (confirmed, not final) | Memory | No: a restart in `PENDING` returns to `SETUP` (ENK-OWN-008) |
| Held proposal (checked, not confirmed) | Memory, with the claim session | No (ENK-OWN-007: nothing persists before confirm) |
| Sessions, login delay | Memory | No (ENK-AUTH-003, ENK-OWN-012) |
| Setup window | Memory, from the monotonic clock | No: power opens a new window (ENK-OWN-001) |

At start and after `do_restart`, the application tells the platform what the state requires:
`enter_setup` without an owner, `apply_settings` with one.

## 6. The transaction (`app/change.rs`, M2 stage 1)

Chapter 6 §4, as the daemon runs it:

| Step | Where | What |
|---|---|---|
| Validate | `settings_post` | Version check (409 `version_mismatch` with the current document), then `settings::apply_update` (400 `invalid` with every field); nothing persistent before this passes (ENK-CFG-005) |
| Refuse a second change | `start_change` | 409 `change_pending` with the state of the one in progress (ENK-CFG-011) |
| Marker on flash | `start_change` | `pending.json`: id, kind, versions, `needs_confirmation`, and the previous document, written by temp-file-and-rename before the device is touched (ENK-CFG-006) |
| 202 | `start_change` | The `applying` shape; the rest runs on its own thread |
| Apply and verify | `perform_change` | `Platform::apply_settings`: owned sections written, reload, health check up to 60 s (ENK-CFG-007) |
| Settle | `settle` | The candidate document becomes the stored one, the marker goes, `settled` (ENK-CFG-010) |
| Confirmation needed | `perform_change` | After a passed health check: `disconnect_management_stations` (hostapd `del_client` with deauthentication), *then* `waiting_confirmation` with a 600 s deadline (ENK-API-021, ENK-CFG-008) |
| Confirmed | `note_owner_request` | Called by the API layer for every request with a session header, before the request is handled: a valid owner session on a connection accepted after the disconnection settles the change, and that same response already says `settled` (ENK-API-022). No separate action. |
| Not confirmed | `tick_change` | At the deadline the decision is taken under the lock, so no later request can confirm; the undo runs on its own thread |
| Undo | `undo` | The previous document is re-applied; the state reads `applying` meanwhile, which is what the device is doing; then `undone` with `reason` and `detail`; the marker goes (ENK-CFG-009) |
| Restart | `App::new` | A marker found at start-up becomes an `undone` change with reason `restart`; the stored document, which is the one from before the change, is applied as at every start; the numbering continues from the marker's id |

Two interpretations to carry into the next amendment RFC:

- **The snapshot is the previous desired document.** ENK-CFG-001 makes the owned sections a pure
  function of that document, so storing the document restores the sections exactly, and the store
  already holds it: the stored document only changes when a change settles. A second copy of the
  rendered sections would be dead data.
- **`validating` is never observable.** Validation is synchronous and complete before the 202.
  The state stays in the shape for the contract's sake.

The accept instant of ENK-API-022 is taken on the daemon's monotonic clock when the connection is
accepted (`Request::accepted`), before the request is read; the disconnection instant is taken
after `del_client` returned. Strictly later wins. Every request already closes its connection
(`Connection: close`), so "a connection accepted after" is simply "a request that began after".

## 6a. Readings and drift (`app/readings.rs`, M2 stage 3)

| Rule | How |
|---|---|
| `status` answers from readings at most 2 s old and queries ubus at most once per 2 s (ENK-API-027) | One `Readings` value in the application state with its instant; `checked_ago` is its age. A settled or undone change discards it. |
| The owned sections are compared with what the daemon last wrote at start-up and at least every 10 minutes (ENK-CFG-014) | `check_drift_if_due`: the tick thread every 600 s; `status` at most every 30 s while the UI is open; never while a change runs. The platform regenerates the sections from the current plan and compares them section by section, format-independent (`uci::owned_equal`), radios included. |
| Drift is reported and never overwritten by the daemon itself | `status.drift`; the UI's banner offers *restore*, an ordinary change of kind `restore` that keeps the version |
| Start-up without a marker adopts the device as the boot left it | `Platform::adopt_settings`: compare first; equal means verify only, nothing written; different means drift, reported and left alone. With a marker the stored document is applied instead, which undoes the cut-short change, or completes it when it was a restore (reported `settled`). |
| A hand edit through `uci commit` hands the file back to root, mode 0600 | The daemon's next read or write fails with `PlatformError::NotOurs`; it logs `files_not_ours` and asks `procd` to restart its service (`ubus call rc init`), because a plain exit would only be respawned with the same command line and the same unreadable file. The init script's `start_service` restores ownership; the new process adopts the device and reports the drift. Sessions are lost with the restart, so the owner logs in again. Were a write to fail the same way during a restore, the start-up apply with the restore marker on flash completes it. If the restart request fails, the daemon stays up and logs it. |
| The uplink's subnet collides with a Wi-Fi network (ENK-TOPO-003, 006) | `subnet::renumber` moves only the colliding network to the next free candidate (management: 192.168.207 to 254, then 172.31.x; guest: 10.207.0 to 254, then 172.16.x) and never moves a network back. Checked at start-up and every 10 s (one ubus call), because the uplink's lease often arrives after the daemon started, and until the colliding network has moved the device has two interfaces on one subnet. The platform re-applies the stored document with the new addresses; no owner-visible change. The daemon binds the management address the platform reports (`App::listen_address`; the configured `--listen` gives the port and the address before any network exists), so when that address moved it asks for a service restart and the new process binds the new one. The `Host` check accepts the bound address. Two paths, both seen on the VM: the lease was there before the listener (start-up check), so the process binds the new address at once; or it came later, then the order is networks moved, health check of that apply, `listen_address_changed`, restart, and between the move and the new process the UI answers on neither address, the same gap as after any restart. The health check does not count the daemon's own listener on the address it is leaving as exposed (before that, the move waited the full 60 s). |

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
