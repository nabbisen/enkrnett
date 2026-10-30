# RFC 015 — Management API

**Status.** Proposed (amended 2026-10-31 after the architect's self-review 12)
**Tracks.** The interface between the management UI and the daemon. RFC 003 open question 2
(request limits); RFC 004 §7 (authorization); RFC 007 threat 23; RFC 011 consequence "the API RFC
names the save operation"; gap analysis GAP-007, GAP-015, GAP-050, GAP-092.
**Touches.** The daemon (HTTP service, every resource); the management UI (its only data source);
the browser harness; the consolidated specification; RFC 004 §7 and RFC 011 ENK-CFG-008 / §8
(amended, see Consequences).

Terms are defined in [RFC 001](../accepted/001-scope-people-vocabulary.md) as amended by
[RFC 009](../accepted/009-vocabulary-amendment-operator-words.md). Channel and request security:
[RFC 003](../accepted/003-management-channel-and-ui-delivery.md). States, sessions, and
authorization: [RFC 004](../accepted/004-ownership-credentials-authorization.md). Configuration
and transactions: [RFC 011](../accepted/011-configuration-model-and-transactions.md). Guest modes
and counts: [RFC 012](../accepted/012-owe-modes-and-status.md). Update:
[RFC 013](../accepted/013-build-release-update.md). Health check:
[RFC 014](../accepted/014-platform-findings-carried.md).

## Summary

One small HTTP/1.1 API, JSON only, under `/api/1/`. Reads are `GET`; everything that changes
something is `POST`. Two reads carry the whole UI: `status` (the first screen, including the
desired settings) and `device` (the details screen). One write, `POST settings`, saves the desired
document and starts the transaction of RFC 011; a few named actions do the rest (claim, login,
restore, management key, password, update, restart, erase). A change that needs the owner's
confirmation is confirmed by the owner **joining the new management Wi-Fi**: the first
authenticated request through it settles the change; there is no button to press. Every shape is
defined once and embedded whole, every error has one shape with a stable code, every duration is
a number of seconds relative to now, and no sentence meant for an operator travels in the API.
The limits that RFC 007 threat 23 asked for are fixed here.

## Motivation

| Fact | Source |
|---|---|
| The drafts named sixteen routes under two naming schemes, including pairing, diagnostics, and operation polling that the accepted RFCs have withdrawn. | ED §12; ARCH §10.1; GAP-092 |
| Which resources answer without a session was not stated; device and capability resources would have disclosed firmware details to anyone on the network. | GAP-015; RFC 004 ENK-AUTH-007 |
| "One step or two" for saving was undecided; RFC 011 decided one step and left the name to this RFC. | GAP-050; RFC 011 |
| Limits on request size, rate, and time were deferred to this RFC. | RFC 007 §7 open question 1 |
| The UI and the daemon ship as one unit; a separate client is allowed but not required. | RFC 003 ENK-MGMT-004, 006 |
| The daemon has no asynchronous runtime, no framework, no TLS, and a bounded number of threads. | RFC 010 ENK-FOOT-012, 013 |
| The device's clock is unreliable; durations are measured monotonically. | RFC 004 ENK-AUTH-004; RFC 011 ENK-CFG-012 |
| After a key change, hostapd may keep an already associated station connected; only a new association proves that a client holds the new key. | hostapd behaviour; self-review 12 finding 1 |
| A `POST` without body or custom header is a browser "simple request" that a page on another origin can send without preflight. | Fetch standard, CORS |

## Design

### 1. Conventions

| ID | Requirement | Verification |
|---|---|---|
| ENK-API-001 | The API shall live under the path prefix `/api/1/`. The integer is the API version; it changes only when an existing shape changes incompatibly, and the greeting states it. Paths are matched exactly. | Inspection |
| ENK-API-002 | Every request to a `POST` resource shall carry a JSON body, `{}` when the action has no parameters, with `Content-Type: application/json`; every response body shall be a JSON object, `{}` when there is nothing to say, as `application/json; charset=utf-8`. A `POST` without a JSON content type is refused with 415. | Test; cross-origin test (the preflight is refused) |
| ENK-API-003 | Reads shall use `GET` and change nothing. Everything that changes state shall use `POST`. Any other method answers 405. | Test with `PUT`, `PATCH`, `DELETE`, `OPTIONS` |
| ENK-API-004 | Field names shall be `snake_case`. Every field of a shape shall always be present; `null` means "not known" or "not applicable" and appears only where this RFC says so. | Schema test |
| ENK-API-005 | A request with a field this RFC does not define, a value of the wrong type, or a body on a `GET` shall be refused with 400 and the field named. Nothing is ignored silently. | Test |
| ENK-API-006 | Durations shall be integers in seconds relative to the moment of the response: `*_in` for a remaining time, `*_ago` for an elapsed time, `*_seconds` for an estimate. The API shall never send a date or a clock time. | Inspection |
| ENK-API-007 | The API shall send no text meant for the operator's screen. Every state is an enumerated value and every error a code; the UI holds the wording in each language. The optional `message` of an error is English, for developers, and the UI never displays it. | Inspection |
| ENK-API-008 | The session token shall be sent in the request header `Enkrnett-Session`. Cookies and the `Authorization` header are never read (RFC 003 ENK-SEC-010). | Test |
| ENK-API-009 | A shape shall be defined once (§4) and, where it appears inside another shape, embedded whole. | Schema test |

Why `GET` and `POST` only: a non-expert's UI has reads and actions. "Replace", "patch", and
"delete" are distinctions the operator never makes, and fewer methods are fewer things a client
can get wrong. Why a body on every `POST`: it makes every action a request that browsers will not
send across origins without a preflight, which the device refuses.

### 2. Authorization and states

The matrix of RFC 004 §7, made concrete.

| Resource | `UNCLAIMED` | `SETUP` | `PENDING` | `CLAIMED` |
|---|---|---|---|---|
| UI files, greeting | no network | anyone | anyone | anyone |
| `claim/*` | — | claim session | — | — |
| `login` | — | — | anyone (makes the claim final) | anyone |
| everything else | — | — | — | owner session |

| ID | Requirement | Verification |
|---|---|---|
| ENK-API-010 | A resource that the current state does not offer shall answer 403 `wrong_state` with the current state, whether or not a session is presented. A resource that needs a session and gets none, or an ended one, shall answer 401 `no_session`. | Test of every resource in every state |
| ENK-API-011 | The greeting shall be exactly the `greeting` shape: `product`, `api`, `state`, `languages`, `development` (RFC 002 ENK-HARD-007), and `connection` (RFC 007 ENK-OWN-022). Nothing else is readable without a session. | Test (RFC 004 ENK-AUTH-007) |
| ENK-API-012 | While `update.state` is not `idle`, every `POST` except `logout` shall answer 409 `update_in_progress`. | Test |

`connection.encrypted` is read from the requesting address: neighbour table to station address,
station address to the BSS it is associated with (`hostapd.* get_clients`). On the management
network it is `true`. When the daemon cannot determine it, it is `null`, and the UI says that it
could not tell rather than guessing.

### 3. Resources

Paths are relative to `/api/1/`. "Returns" names a shape of §4 or an error code of §5.

| Path | Method | Who | Does | Returns |
|---|---|---|---|---|
| `` (the prefix itself) | GET | anyone | Greeting | `greeting` |
| `claim/start` | POST `{}` | anyone, in `SETUP` | Grants the claim session to this client if none exists (RFC 004 step 3) | 200 `session`; 409 `setup_in_progress_elsewhere` |
| `claim` | GET | claim | What setup needs: country choice, limits, defaults, whether identify is available, remaining window | `claim` |
| `claim/identify` | POST `{}` | claim | Blinks the light (RFC 004 step 4) | 200 `{}`; 403 `not_available` |
| `claim/check` | POST | claim | Validates country, guest name and mode, owner password; generates the management name and key in memory; persists nothing (ENK-OWN-007) | 200 `proposal`; 400 `invalid` |
| `claim/confirm` | POST `{}` | claim | Makes the proposal real (RFC 004 steps 7–8): state `PENDING`, setup network ends, management network starts | 200 `proposal`; 409 `no_proposal` |
| `login` | POST | anyone | Creates an owner session from the owner password; in `PENDING` it also makes the claim final (step 10) | 200 `session`; 401 `wrong_password`; 429 `try_later` |
| `logout` | POST `{}` | owner | Ends this session | 200 `{}` |
| `status` | GET | owner | The first screen: desired settings, effective state of every network, the current change, drift, version, update | `status` |
| `settings` | POST | owner | **Save.** Validates the document and starts the transaction of RFC 011 §4 | 202 `change`; 400 `invalid`; 409 `version_mismatch`, `change_pending` |
| `settings/restore` | POST `{}` | owner | The one action for drift (ENK-CFG-014): rewrite the owned sections from the desired document | 202 `change`; 409 `change_pending` |
| `change` | GET | owner | The current or the last transaction | `change` |
| `management-key` | GET | owner | Shows the management key (ENK-OWN-016) | `management_key` |
| `management-key` | POST `{}` | owner | Replaces the key: a transaction that needs confirmation | 202 `change` **plus** `management_key` (see §7); 409 `change_pending` |
| `password` | POST | owner | Changes the owner password; ends every session (ENK-OWN-013) | 200 `{}`; 400 `invalid`; 401 `wrong_password` |
| `device` | GET | owner | The details screen | `device` |
| `update` | GET | owner | Installed and available release, last check, a running update, the last outcome | `update` |
| `update/check` | POST `{}` | owner | Fetches the release index now (ENK-UPD-001) | 200 `update` |
| `update/install` | POST `{}` | owner | Starts the update (ENK-UPD-002, 003) | 202 `update`; 400 `cannot_update`; 409 `change_pending`, `nothing_to_install` |
| `restart` | POST `{}` | owner | Restarts the device after answering | 202 `{}` |
| `erase` | POST | owner | *Erase* (RFC 004 §8): the owner password again; erases and restarts | 202 `{}`; 401 `wrong_password` |

| ID | Requirement | Verification |
|---|---|---|
| ENK-API-013 | These shall be the only resources. Adding one requires an RFC. | Test: every other path under `/api/` answers 404 `unknown_resource` |
| ENK-API-014 | `POST settings` shall be the only way to change the desired document. It carries the whole document except `country` and `management.key`, with the `version` the client last read. A `version` that is not the current one is refused with 409 `version_mismatch` and the current `settings`. A retried save therefore meets `change_pending` (with the change) or `version_mismatch` (with the saved document), and no change is applied twice. | Test, including a retry after a lost response |
| ENK-API-015 | `claim/confirm`, `restart`, and `erase` shall send their response before the network change they cause, and the daemon shall wait at least 2 seconds after sending before it acts. | Test with a slow client |
| ENK-API-016 | Every `/api/` response shall carry `Cache-Control: no-store` (RFC 007 ENK-SEC-015). | Header inspection |
| ENK-API-017 | At most one claim proposal shall exist, kept in memory and discarded when the claim session ends. A second `claim/check` replaces it. | Test |

### 4. Shapes

Only fields are listed; types are JSON's; integers only, no fractions. Enumerations are closed:
the UI treats an unknown value as an error, never as "probably fine".

**`version`**: `enkrnett` ("0.2.0"), `baseline` ("25.12"), `openwrt` ("25.12.5"), `api` (1).
The status screen shows the first two as "Enkrnett 0.2 on OpenWrt 25.12" (RFC 013 ENK-RLS-003).

**`connection`**: `network` (`setup` | `management`), `encrypted` (`true` | `false` | `null`).

**`limits`**: `guest_name_max_bytes` `{ "open_to_all": 29, "encrypted_only": 32 }`,
`management_name_max_bytes` (32), `password` `{ "min": 10, "max": 64 }`. One source for every
figure the UI shows while a name or a password is typed (RFC 011 ENK-CFG-017).

**`greeting`**: `product` ("Enkrnett"), `api` (1), `state` (`UNCLAIMED` | `SETUP` | `PENDING` |
`CLAIMED`), `languages` (BCP 47 tags the UI ships, e.g. `["en", "ja"]`), `development` (boolean),
`connection`.

**`session`**: `token` (at least 128 bits, base64url), `kind` (`claim` | `owner`), `state` (the
state after this request), `idle_ends_in`, `ends_in`.

**`claim`**: `window_ends_in` (ENK-OWN-002), `country` `{ "fixed": "JP" | null, "choices": [...] }`
(when `fixed` is set, `choices` is empty), `guest_mode_default` (`open_to_all`),
`identify_available` (boolean, ENK-OWN-006), `limits`, `connection`.

**`claim/check` request**: `country` (two letters; omitted when fixed), `guest` `{ "name", "mode" }`,
`owner_password`.

**`proposal`**: `management` `{ "name", "key" }`, `address` (the management UI's IPv4 address,
ENK-MGMT-007), `guest` `{ "name", "mode" }`, `country`, `state` (`SETUP` after check, `PENDING`
after confirm), `final_in` (`null` after check; 600 after confirm, ENK-OWN-008).

**`settings`** — the desired document of RFC 011 §2 without the key:

```json
{
  "version": 7,
  "country": "JP",
  "guest": { "name": "Cafe Example", "mode": "open_to_all", "enabled": true },
  "management": { "name": "Enkrnett-3F2A" },
  "limits": { "guest_name_max_bytes": { "open_to_all": 29, "encrypted_only": 32 },
              "management_name_max_bytes": 32, "password": { "min": 10, "max": 64 } }
}
```

`POST settings` sends `version`, `guest`, `management` only.

**`change`**

| Field | Value |
|---|---|
| `id` | integer, increases per attempt; `0` when `state` is `none` |
| `state` | `none` \| `validating` \| `applying` \| `waiting_confirmation` \| `settled` \| `undone` |
| `kind` | `settings` \| `restore` \| `management_key` |
| `from_version`, `to_version` | integers |
| `needs_confirmation` | boolean (ENK-CFG-008) |
| `confirm_in` | seconds left for the owner to join the new management Wi-Fi; `null` unless `waiting_confirmation` |
| `reason` | when `undone`: `health_check_failed` \| `not_confirmed` \| `restart`; otherwise `null` |
| `detail` | when `health_check_failed`: `guest_down` \| `management_down` \| `isolation_missing` \| `dhcp_missing` \| `listener_exposed` \| `pmf_missing` \| `mode_differs`; otherwise `null` |
| `started_ago` | seconds |

When `state` is `none`, every field other than `id` and `state` is `null`. The wording for each
`state` and `reason` is RFC 011 §8 as amended by this RFC; the UI maps one to one.

**`status`**

```json
{
  "state": "CLAIMED",
  "version": { "enkrnett": "0.2.0", "baseline": "25.12", "openwrt": "25.12.5", "api": 1 },
  "settings": { "...": "the settings shape" },
  "guest": { "up": true, "name": "Cafe Example", "mode": "open_to_all",
             "clients_encrypted": 10, "clients_unencrypted": 2, "checked_ago": 3 },
  "management": { "up": true, "name": "Enkrnett-3F2A", "address": "192.168.207.1", "clients": 1 },
  "uplink": { "up": true, "address": "192.168.1.57" },
  "change": { "...": "the change shape" },
  "drift": false,
  "update": { "...": "the update shape" },
  "hardening_met": true
}
```

| Field | Rule |
|---|---|
| `settings` | The desired document. The UI shows it beside the effective values only where they differ (RFC 011 §1). |
| `guest.*`, `management.*` | Effective values (RFC 011 §1, RFC 012 §2). When `up` is `false`, `name` and `mode` are `null` and the counts `0`. In `encrypted_only`, `clients_unencrypted` is `0`, never `null` (ENK-OWE-013). |
| `management.clients` | A count; no addresses. |
| `uplink.address` | The device's own address on the uplink, for the operator's helper; `null` while there is none. |
| `hardening_met` | `true` when every item of `device.hardening` is met. |

**`device`**

| Field | Value |
|---|---|
| `model`, `revision`, `tier`, `device_class` | from the device profile (RFC 005) |
| `device_id` | the random identifier (ENK-OWN-020) |
| `version` | the `version` shape |
| `uptime_seconds` | integer |
| `memory` | `{ "total_kib", "available_kib" }` |
| `storage` | `{ "overlay_total_kib", "overlay_free_kib" }` |
| `hardening` | list of `{ "id": "ENK-HARD-001", "met": true }` for 001 to 005 and 008 |
| `installed_as` | `image` \| `package` (RFC 005 ENK-INST-005) |
| `radios` | list of `{ "band": "2g" \| "5g" \| "6g", "in_use": true }` |

**`update`**

| Field | Value |
|---|---|
| `installed` | `{ "version", "baseline" }` |
| `checks_enabled` | boolean (ENK-UPD-008) |
| `last_check` | `{ "checked_ago", "result": "ok" \| "unreachable" \| "bad_signature" \| "bad_index" }` or `null` |
| `available` | `null` or `{ "version", "baseline", "changes_baseline", "size_kib", "notes_url", "install_seconds" }` |
| `state` | `idle` \| `downloading` \| `verifying` \| `installing` — an attempt in progress, nothing else |
| `last_result` | `null` or `{ "result": "done" \| "failed", "reason": null \| "bad_signature" \| "wrong_device" \| "wrong_baseline" \| "not_enough_memory" \| "download_failed" \| "health_check_failed" }` — the outcome of the latest attempt, kept across the restart (ENK-UPD-005) |
| `can_undo` | `false` on single-slot devices (ENK-UPD-006), so the UI can say "this cannot be undone from here" |

**`management_key`**: `name`, `key`.

### 5. Errors

One shape, one code per cause.

```json
{ "error": { "code": "invalid",
             "fields": [ { "field": "guest.name", "code": "too_long", "limit": 29 } ],
             "message": "guest.name is 31 bytes; at most 29 in mode open_to_all" } }
```

| HTTP | Code | When |
|---|---|---|
| 400 | `invalid` | A value fails validation; `fields` lists each with its own code: `required`, `unknown_field`, `wrong_type`, `too_short`, `too_long`, `not_allowed`, `forbidden_characters`, `common_password`, `equals_network_name`, `equals_key`, `equals_device_id`, `not_in_choices` |
| 400 | `cannot_update` | `update/install` refused by ENK-UPD-003; `reason` carries one of the `last_result.reason` values |
| 401 | `no_session` | No token, unknown token, or an ended session |
| 401 | `wrong_password` | `login`, `password`, `erase` |
| 403 | `wrong_state` | The resource is not offered in the current state; `state` is included |
| 403 | `not_available` | The device profile lacks what the action needs (identify) |
| 404 | `unknown_resource` | Any other path under `/api/` |
| 405 | `method_not_allowed` | Wrong method for a known path, including `OPTIONS` |
| 409 | `setup_in_progress_elsewhere`, `no_proposal`, `version_mismatch`, `change_pending`, `nothing_to_install`, `update_in_progress` | State conflicts; `version_mismatch` includes the current `settings`, `change_pending` the current `change` |
| 413 | `too_large` | Body over the limit of §6 |
| 415 | `wrong_content_type` | — |
| 421 | — | `Host` is not the device (RFC 003 ENK-SEC-009); empty body |
| 429 | `try_later` | Login delay (ENK-OWN-012); `Retry-After` header and `retry_in` in the body |
| 431 | `too_large` | Request line and headers over the limit of §6 |
| 500 | `internal` | A defect; `reference` is a short value that the log line also carries; never a stack trace |

| ID | Requirement | Verification |
|---|---|---|
| ENK-API-018 | Every error response shall use this shape and one of these codes. A new code requires an RFC. | Schema test |
| ENK-API-019 | Validation shall report every failing field at once, not the first only. | Test with two invalid fields |
| ENK-API-020 | `password` shall validate the new password first (400 `invalid`), then check the current one. A `wrong_password` answer shall take the same time whatever the wrong value was. | Timing test |

### 6. Limits (RFC 007 threat 23)

| ID | Requirement | Verification |
|---|---|---|
| ENK-SEC-017 | Request line and headers together shall not exceed 8 KiB (431) and a request body shall not exceed 16 KiB (413); the connection is then closed. | Test |
| ENK-SEC-018 | The daemon shall close a connection that has not delivered a complete request within 10 seconds, and shall finish or abandon any response within 30 seconds. | Test with a slow client |
| ENK-SEC-019 | The daemon shall answer every request with `Connection: close` and shall serve at most 4 connections at a time, one per worker thread, with a listen backlog of 16. | Test with 32 concurrent clients; memory within RFC 010 ENK-FOOT-007 |
| ENK-SEC-020 | The daemon shall speak HTTP/1.1 only, and shall accept no chunked request body, no `Expect`, no compression, and no `Transfer-Encoding`. Its JSON parser shall refuse duplicate keys, nesting deeper than 8, and numbers that are not integers within 64 bits. | Test |
| ENK-SEC-021 | The API shall accept no file upload. The update image is fetched by the device from the release index (RFC 013), never pushed by a client. | Inspection |
| ENK-SEC-022 | Response bodies shall not exceed 64 KiB. | Test |

Why these numbers: one operator on one phone produces a few requests per second at most; the
largest legitimate body is a `settings` document under 1 KiB; the largest response is `status`
with its embedded shapes. Without keep-alive, four workers serve two phones polling at once with
room to spare, and no idle connection can hold a thread. The numbers keep a hostile client that
holds the management key from holding memory.

### 7. Confirmation of a change

RFC 011 ENK-CFG-008 requires the owner's confirmation through the management network for a
change of the management network's name or key. This RFC defines what counts as confirmation.

| ID | Requirement | Verification |
|---|---|---|
| ENK-API-021 | Once the health check of such a change has passed, the daemon shall disconnect every station of the management network (`hostapd.* del_client` with deauthentication) and only then enter `waiting_confirmation`, so that any later request proves a new association with the new name or key. | Test |
| ENK-API-022 | The change shall be confirmed by the first request with a valid owner session that arrives on a connection accepted after the disconnection of ENK-API-021 completed, while the change is `waiting_confirmation`. There is no separate confirmation action. | Test: a request received before the disconnection does not settle the change; the first one after it does |
| ENK-API-023 | `POST management-key` shall answer with the `change` shape and, beside it, a `management_key` field holding the new key, because the operator needs the key before the reload and cannot read it afterwards. This is the only compound response. | Inspection |

What the operator sees: "Saved. Join the new management Wi-Fi within 3 minutes, or the change is
undone." with the new name and key and a countdown (`confirm_in`). When they join and the UI's
next poll arrives, the screen says "Saved." No button. The act that proves the owner can still
reach the device is joining the network; a button would add a step and prove nothing.

### 8. Static files and headers

| ID | Requirement | Verification |
|---|---|---|
| ENK-API-024 | `GET /` and `GET /index.html` shall serve the UI's entry page with `Cache-Control: no-store`; every other UI file shall be named with a hash of its content and served with `Cache-Control: max-age=31536000, immutable`. | Inspection (RFC 003 ENK-MGMT-005) |
| ENK-API-025 | Every response, API or file, shall carry: `Content-Security-Policy: default-src 'self'; img-src 'self' data:; frame-ancestors 'none'; base-uri 'none'; form-action 'none'`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`, `Cross-Origin-Opener-Policy: same-origin`, `Cross-Origin-Resource-Policy: same-origin`. No `Access-Control-*` header is ever sent. | Header inspection (RFC 003 ENK-SEC-011, 012) |
| ENK-API-026 | A path outside `/api/` that is not a UI file shall serve the entry page (the UI routes on the client) with the headers above. | Test |

`img-src data:` exists for the QR code of the address (ENK-MGMT-009), which the UI draws itself.
The guest notice (RFC 012 ENK-GUEST-001) is a page of the UI rendered from `status`; it needs no
resource.

### 9. Polling

There is no push channel. The UI polls.

| Screen | Resource | Interval |
|---|---|---|
| Status | `status` | 5 s |
| While `change.state` is `validating`, `applying`, or `waiting_confirmation` | `change` | 1 s |
| While `update.state` is not `idle` | `update` | 2 s |
| During setup, before confirm | `claim` | 30 s (for `window_ends_in`) |

| ID | Requirement | Verification |
|---|---|---|
| ENK-API-027 | The daemon shall answer `status` from readings at most 2 seconds old and shall not query ubus more than once per 2 seconds for it, whatever the number of clients. | Test with two clients polling |

A push channel would cost a held connection per client in a daemon with four threads; a poll
every few seconds is what the status screen needs.

### 10. Log lines

RFC 004 replaced audit events by log lines and left their content to a later RFC. This is it.

| ID | Requirement | Verification |
|---|---|---|
| ENK-API-028 | The daemon shall log to the platform's system log (`logd`, in memory) and shall write no log file of its own. | Inspection (D-15; RFC 010) |
| ENK-API-029 | It shall log, one line each: session created and ended (kind, cause); login refused (count of consecutive failures); claim started, confirmed, made final, abandoned; change started, settled, undone (id, kind, reason, detail); drift detected; update checked, started, failed, done; erase and restart requested; every 500 with its reference. | Inspection after exercising every function |
| ENK-API-030 | A log line shall contain no station address, no client address, no name entered by the operator, and nothing listed in RFC 007 ENK-SEC-016. | Search of the log after exercising every function |

The management UI does not show the log. An expert reads it with the platform's tools after an
expert install (RFC 005 ENK-INST-005); for the operator, the status and the change's `reason`
and `detail` are the diagnosis.

### 11. Versioning and clients

| ID | Requirement | Verification |
|---|---|---|
| ENK-API-031 | A field may be added to a shape, and a value to an enumeration, only together with the UI release that handles it (they ship together, ENK-MGMT-004). A client that is not the device's UI shall treat an unknown enumeration value as "unknown" and shall not act on it. | Inspection |
| ENK-API-032 | Removing a field or changing a meaning shall increase the API version; the previous prefix is retired in the same release. The device never serves two API versions. | Inspection |

## Worked flows

**Setup, from the phone.** `GET /api/1/` → `state: SETUP`; the UI shows `connection.encrypted`
first (ENK-OWN-022) → `POST claim/start` → `GET claim` → identify → `POST claim/check` with
country, name, mode, password → the UI shows the management name, key, and address and asks the
operator to keep them → `POST claim/confirm` → the phone loses the setup network within seconds →
the operator joins the management network and opens the same address → `GET /api/1/` →
`state: PENDING` → `POST login` → `session.state: CLAIMED` → `GET status`.

**Changing the guest name.** `GET status` (settings version 7) → `POST settings` with version 7
and the new name → 202 `change {state: applying, needs_confirmation: false}` → poll `change` each
second → `settled` → `status.guest.name` is the new name. On a health-check failure: `undone`,
`reason: health_check_failed`, `detail: guest_down`; the UI shows RFC 011 §8's sentence; the
previous settings are already back.

**Replacing the management key.** `POST management-key` → 202 `change {needs_confirmation: true}`
with `management_key` beside it → the UI shows the new key and the countdown → the health check
passes, the daemon disconnects the management stations → the operator joins again with the new
key; the session is still valid (sessions survive a reload) → the UI's next poll of `change`
arrives from the fresh association and settles the change → `settled`. If no such request arrives
within 180 s: `undone`, `reason: not_confirmed`, and the old key works again.

## Consequences

| Affected | Change |
|---|---|
| Draft routes `GET /api/v1/device`, `/state`, `/config`, `/capabilities` | Replaced by `device`, `status` (which carries `settings`), and `claim` |
| Draft `PUT /config` + `POST /config/apply`, `GET /operations/{id}` | One operation `POST settings`; progress through `change` (GAP-050 closed) |
| Draft `pairing/begin`, `/complete`, `/revoke`, `GET /pairing` | `claim/start`, `claim/check`, `claim/confirm`; no revoke (RFC 004 ENK-OWN-019) |
| Draft `POST /factory-reset`, `/restart`, `/recovery/prepare` | `erase`, `restart`; no recovery resource (RFC 004 §8: one path) |
| Draft `POST /diagnostics`, `GET /diagnostics/{id}` | Withdrawn; `change.detail`, `device.hardening`, and the log replace them |
| RFC 004 §7 greeting | Gains `development` and `connection` (needed by ENK-HARD-007 and ENK-OWN-022, both accepted after RFC 004) |
| RFC 004 ENK-OWN-016 | Satisfied as written: the replacement becomes final by an authenticated act through the new network (ENK-API-021) |
| RFC 011 ENK-CFG-008 | "The owner's confirmation" is defined by ENK-API-021 and 022. No separate action. |
| RFC 011 §8, row "waiting for confirmation" | Reads: "Saved. Join the new management Wi-Fi within 3 minutes, or the change is undone." with the new name and key and a countdown. The word *Keep* is not used. |
| RFC 007 threat 23 | Covered by ENK-SEC-017 to 022; remaining risk beyond the Wi-Fi layer (R-10): none known |
| RFC 007 open question 1; RFC 003 open question 2 | Closed |
| RFC 011 "the API RFC names it" | `POST settings` |
| Gap analysis GAP-007, GAP-015, GAP-050, GAP-092 | Closed |
| Operator UI RFC | Maps every enumeration and error code of this RFC to wording in each language; nothing else reaches the screen |
| Browser harness (M1) | Runs against a mock that implements exactly these shapes |

## Alternatives considered

| Alternative | Why not |
|---|---|
| A *Keep* button for confirmation (RFC 011 §8 as first written) | Proves nothing: after a key change the phone may still be associated with the old key. Joining the new network is the proof, and the device can observe it. |
| REST with `PUT`/`PATCH`/`DELETE` | Three more methods for distinctions the operator never makes; `PATCH` invites partial documents and silent merges (RFC 011 ENK-CFG-001 forbids merging). |
| `GET settings` beside `status` | The first screen needs the desired document anyway; two reads of one object are two ways. |
| Operator wording in API responses, with `Accept-Language` | Two places for one sentence; the UI already ships every language. |
| Absolute timestamps | The device's clock may be wrong by years at first boot. |
| Keep-alive connections | Idle connections would hold worker threads; a second phone would starve. |
| Server-sent events for change progress | A held connection per client; polling at 1 s is indistinguishable on a phone. |
| A log resource for the operator | Logs are the stock product's answer; Enkrnett's is a sentence with a reason. |
| Cookies with `SameSite=Strict` | A credential the browser attaches by itself; RFC 003 ENK-SEC-010 rules it out. |
| Repeating the `claim/check` body in `claim/confirm` | The password would travel twice; a held proposal is simpler and matches "check, then confirm". |

## Open questions

| # | Question | Closed by |
|---|---|---|
| 1 | Whether `Host` also accepts a fixed name (RFC 003 ENK-MGMT-008) | M-1 |
| 2 | The UI normalises names to Unicode NFC before sending, so that byte counts match what the operator sees | Operator UI RFC |
| 3 | Further `change.detail` codes from the internal design of ENK-CFG-007 | M1 internal design; added here by amendment |
| 4 | JSON Schema files generated from `enkrnett-model`, used by the UI and the harness | M1 handoff |
| 5 | `hostapd.<iface> del_client {addr, reason, deauth, ban_time}` exists on 25.12.5 (checked on the measurement VM, 2026-10-31). Its entry in the daemon's ubus ACL (ENK-CFG-020) is part of the internal design. | M1 internal design |
