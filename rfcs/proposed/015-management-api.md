# RFC 015 — Management API

**Status.** Proposed
**Tracks.** The interface between the management UI and the daemon. RFC 003 open question 2
(request limits); RFC 004 §7 (authorization); RFC 007 threat 23; RFC 011 consequence "the API RFC
names the save operation"; gap analysis GAP-007, GAP-015, GAP-050, GAP-092.
**Touches.** The daemon (HTTP service, every resource); the management UI (its only data source);
the browser harness; the consolidated specification.

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
something is `POST`. Three reads carry the whole UI (`status`, `settings`, `device`); one write
saves the desired document and starts the transaction of RFC 011; a handful of named actions do
the rest (claim, login, keep, restore, management key, password, update, restart, erase). Every
response has one shape per resource, every error has one shape with a stable code, every duration
is a number of seconds relative to now, and nothing is sent that the operator's screen does not
need. The limits that RFC 007 threat 23 asked for are fixed here.

## Motivation

| Fact | Source |
|---|---|
| The drafts named sixteen routes under two different naming schemes, including pairing, diagnostics, and operation polling that the accepted RFCs have since withdrawn. | ED §12; ARCH §10.1; GAP-092 |
| Which resources answer without a session was not stated; device and capability resources would have disclosed firmware details to anyone on the network. | GAP-015; RFC 004 ENK-AUTH-007 |
| "One step or two" for saving configuration was undecided; RFC 011 decided one step and left the name to this RFC. | GAP-050; RFC 011 |
| Limits on request size, rate, and time were deferred to this RFC; until they exist, threat 23 (exhaustion) has no remaining-risk entry. | RFC 007 §7 open question 1 |
| The UI and the daemon ship as one unit; a separate client is allowed but not required. | RFC 003 ENK-MGMT-004, 006 |
| The daemon has no asynchronous runtime, no framework, no TLS, and a bounded number of threads. | RFC 010 ENK-FOOT-012, 013 |
| The device's clock is unreliable; durations are measured monotonically. | RFC 004 ENK-AUTH-004; RFC 011 ENK-CFG-012 |

## Design

### 1. Conventions

| ID | Requirement | Verification |
|---|---|---|
| ENK-API-001 | The API shall live under the path prefix `/api/1/`. The integer is the API version; it changes only when an existing response or request shape changes incompatibly, and the greeting states it. | Inspection |
| ENK-API-002 | Every request body and every response body shall be JSON, `application/json; charset=utf-8`. A request with another content type shall be refused with 415. | Test |
| ENK-API-003 | Reads shall use `GET` and shall change nothing. Everything that changes state shall use `POST`. No other method is used. | Inspection; test with `PUT`, `PATCH`, `DELETE` (405) |
| ENK-API-004 | Field names shall be `snake_case`. A field of a response shape shall always be present; `null` means "not known" or "not applicable" and is used only where this RFC says so. | Schema test |
| ENK-API-005 | A request with a field this RFC does not define, or with a value of the wrong type, shall be refused with 400 and the field named. Nothing is ignored silently. | Test |
| ENK-API-006 | Durations shall be integers in seconds relative to the moment of the response: `*_in` for a remaining time, `*_ago` for an elapsed time. The API shall never send a date or a clock time. | Inspection |
| ENK-API-007 | The API shall send no text meant for the operator's screen. Every state is an enumerated value and every error a code; the UI holds the wording in each language. The optional `message` of an error is English, for developers, and the UI never displays it. | Inspection |
| ENK-API-008 | The session token shall be sent in the request header `Enkrnett-Session`. Neither cookies nor the `Authorization` header are read (RFC 003 ENK-SEC-010). | Test |

Why `GET` and `POST` only: a non-expert's UI has reads and actions; "replace", "patch", and
"delete" are distinctions the operator never makes, and fewer methods are fewer things a client
can get wrong.

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
| ENK-API-009 | A resource that the current state does not offer shall answer 403 with code `wrong_state` and the current state, whether or not a session is presented. A resource that needs a session and gets none, or an ended one, shall answer 401 with code `no_session`. | Test of every resource in every state |
| ENK-API-010 | The greeting shall be exactly: `product`, `api`, `state`, `languages`, `development` (RFC 002 ENK-HARD-007), and `connection` with `network` (`setup` or `management`) and `encrypted` (RFC 007 ENK-OWN-022). Nothing else is readable without a session. | Test (RFC 004 ENK-AUTH-007) |

`connection.encrypted` is determined by the daemon from the requesting address: neighbour table
to station address, station address to the BSS it is associated with (`hostapd.* get_clients`).
On the management network it is always `true`. When the daemon cannot determine it, it is
`null` and the UI says so rather than guessing.

### 3. Resources

Paths are relative to `/api/1/`. "Shape" names the response body described in §4.

| Path | Method | Who | Does | Returns |
|---|---|---|---|---|
| `` (empty) | GET | anyone | Greeting | `greeting` |
| `claim/start` | POST | anyone, in `SETUP` | Grants the claim session to this client if none exists (RFC 004 step 3) | 200 `session`; 409 `setup_in_progress_elsewhere` |
| `claim` | GET | claim | What setup needs to know: country choice, limits, defaults, whether identify is available, remaining window | `claim` |
| `claim/identify` | POST | claim | Blinks the light (RFC 004 step 4) | 200 empty; 403 `not_available` |
| `claim/check` | POST | claim | Validates country, guest name and mode, owner password; generates the management network name and key in memory; persists nothing (ENK-OWN-007) | 200 `proposal`; 400 `invalid` |
| `claim/confirm` | POST | claim | Makes the proposal real (RFC 004 steps 7–8): state `PENDING`, setup network ends, management network starts | 200 `proposal` with `state`; 409 `no_proposal` |
| `login` | POST | anyone | Creates an owner session from the owner password; in `PENDING` it also makes the claim final (step 10) | 200 `session`; 401 `wrong_password`; 429 `try_later` |
| `logout` | POST | owner | Ends this session | 200 empty |
| `status` | GET | owner | What the status screen shows: effective state of every network, the current change, drift, version, update summary | `status` |
| `settings` | GET | owner | The desired document, without the management key | `settings` |
| `settings` | POST | owner | **Save.** Validates and starts the transaction of RFC 011 §4 with the given document | 202 `change`; 400 `invalid`; 409 `version_mismatch`, `change_pending`, `update_in_progress` |
| `settings/restore` | POST | owner | The one action for drift (ENK-CFG-014): rewrite the owned sections from the desired document | 202 `change`; 409 as above |
| `change` | GET | owner | The current or the last transaction | `change` |
| `change/keep` | POST | owner | The owner's confirmation (ENK-CFG-008); the UI's word is *Keep* | 200 `change`; 409 `nothing_to_keep` |
| `management-key` | GET | owner | Shows the management key (ENK-OWN-016) | `management_key` |
| `management-key` | POST | owner | Replaces the key: a transaction that needs confirmation | 202 `change`; 409 as for save |
| `password` | POST | owner | Changes the owner password; ends every session (ENK-OWN-013) | 200 empty; 401 `wrong_password`; 400 `invalid` |
| `device` | GET | owner | The details screen: model, tier, device identifier, versions, uptime, hardening report (ENK-HARD-006), memory and storage figures | `device` |
| `update` | GET | owner | Installed and available release, last check, progress of a running update | `update` |
| `update/check` | POST | owner | Fetches the release index now (ENK-UPD-001) | 200 `update` |
| `update/install` | POST | owner | Starts the update (ENK-UPD-002, 003) | 202 `update`; 409 `change_pending`, `nothing_to_install`, `update_in_progress`; 400 `cannot_update` with the reason |
| `restart` | POST | owner | Restarts the device after answering | 202 empty |
| `erase` | POST | owner | *Erase* (RFC 004 §8): requires the owner password again; erases and restarts | 202 empty; 401 `wrong_password` |

| ID | Requirement | Verification |
|---|---|---|
| ENK-API-011 | These shall be the only resources. Adding one requires an RFC. | Test: every other path under `/api/` answers 404 `unknown_resource` |
| ENK-API-012 | `POST settings` shall be the one and only way to change the desired document, and shall carry the whole document except `country` and `management.key`, with the `version` the client last read. A `version` that is not the current one shall be refused with 409 `version_mismatch` and the current document. | Test |
| ENK-API-013 | `claim/confirm`, `restart`, and `erase` shall send their response before the network change they cause, and the daemon shall wait at least 2 seconds after sending before it acts. | Test with a slow client |
| ENK-API-014 | A response that contains the management key (`proposal`, `management_key`) or a session token (`session`) shall carry `Cache-Control: no-store` (RFC 007 ENK-SEC-015). For simplicity every `/api/` response carries it. | Inspection of headers |
| ENK-API-015 | The device shall accept at most one claim proposal at a time, kept in memory and discarded when the claim session ends. A second `claim/check` replaces it. | Test |

### 4. Shapes

Only the fields are listed; types are JSON's. Enumerations are closed: the UI treats an unknown
value as an error, never as "probably fine".

**`greeting`**

| Field | Value |
|---|---|
| `product` | `"Enkrnett"` |
| `api` | `1` |
| `state` | `UNCLAIMED` \| `SETUP` \| `PENDING` \| `CLAIMED` |
| `languages` | list of BCP 47 tags the UI ships, e.g. `["en", "ja"]` |
| `development` | boolean |
| `connection` | `{ "network": "setup" \| "management", "encrypted": true \| false \| null }` |

**`session`**: `token` (at least 128 bits, base64url), `kind` (`claim` \| `owner`), `idle_ends_in`,
`ends_in`.

**`claim`**

| Field | Value |
|---|---|
| `window_ends_in` | seconds left of the setup window (ENK-OWN-002) |
| `country` | `{ "fixed": "JP" \| null, "choices": ["JP", "DE", …] }`; when `fixed` is set, `choices` is empty and the UI shows the fixed value |
| `guest` | `{ "name_max_bytes": { "open_to_all": 29, "encrypted_only": 32 }, "mode_default": "open_to_all" }` |
| `password` | `{ "min": 10, "max": 64 }` |
| `identify_available` | boolean (ENK-OWN-006) |
| `connection` | as in the greeting |

**`claim/check` request**: `country` (two letters; omitted when fixed), `guest` `{ "name",
"mode" }`, `owner_password`.

**`proposal`**: `management` `{ "name", "key" }`, `address` (the management UI's IPv4 address as
text, ENK-MGMT-007), `guest` `{ "name", "mode" }`, `country`, `state` (`SETUP` after check,
`PENDING` after confirm), `final_in` (`null` after check; 600 after confirm, ENK-OWN-008).

**`settings`** (the desired document, RFC 011 §2, less the key)

```json
{
  "version": 7,
  "country": "JP",
  "guest": { "name": "Cafe Example", "mode": "open_to_all", "enabled": true },
  "management": { "name": "Enkrnett-3F2A" }
}
```

`POST settings` sends the same object without `country`.

**`change`**

| Field | Value |
|---|---|
| `id` | integer, increases per attempt |
| `state` | `none` \| `validating` \| `applying` \| `waiting_confirmation` \| `settled` \| `undone` |
| `from_version`, `to_version` | integers, `null` when `state` is `none` |
| `needs_confirmation` | boolean (ENK-CFG-008) |
| `confirm_in` | seconds left to press *Keep*, else `null` |
| `reason` | when `undone`: `health_check_failed` \| `not_confirmed` \| `restart` ; otherwise `null` |
| `detail` | when `health_check_failed`: which read-back failed, as a code from the health check (`guest_down`, `management_down`, `isolation_missing`, `dhcp_missing`, `listener_exposed`, `pmf_missing`, `mode_differs`); otherwise `null` |
| `started_ago` | seconds |
| `kind` | `settings` \| `restore` \| `management_key` |

The wording for each `state` and `reason` is RFC 011 §8; the UI maps one to one.

**`status`**

```json
{
  "state": "CLAIMED",
  "version": { "enkrnett": "0.2.0", "baseline": "25.12" },
  "guest": {
    "enabled": true, "up": true,
    "name": "Cafe Example", "mode": "open_to_all",
    "clients_encrypted": 10, "clients_unencrypted": 2,
    "checked_ago": 3
  },
  "management": { "up": true, "name": "Enkrnett-3F2A", "address": "192.168.207.1", "clients": 1 },
  "uplink": { "up": true, "address": "192.168.1.57" },
  "change": { "state": "none", "...": "the change shape" },
  "drift": false,
  "update": { "available": null, "state": "idle", "last_result": null },
  "hardening_met": true
}
```

| Field | Rule |
|---|---|
| `guest.*` | Effective values (RFC 011 §1, RFC 012 §2). In `encrypted_only`, `clients_unencrypted` is `0`, never `null`: the UI shows "all encrypted" from the mode, not from a missing number (ENK-OWE-013). |
| `guest.name` | The effective name; the UI shows the desired one beside it only while `change.state` is not `none` or `settled`. |
| `management.clients` | Count only; no addresses. |
| `uplink.address` | The device's own address on the uplink, for the operator's helper; `null` while there is none. |
| `update` | A subset of the `update` shape: `available` is `null` or `{ "version", "baseline" }`. |
| `hardening_met` | `true` when every item of the hardening report is met; the items are in `device`. |

**`device`**

| Field | Value |
|---|---|
| `model`, `revision`, `tier`, `device_class` | from the device profile (RFC 005) |
| `device_id` | the random identifier (ENK-OWN-020) |
| `version` | `{ "enkrnett", "baseline", "openwrt": "25.12.5", "api": 1 }` |
| `uptime` | seconds |
| `memory` | `{ "total_kib", "available_kib" }` |
| `storage` | `{ "overlay_total_kib", "overlay_free_kib" }` |
| `hardening` | list of `{ "id": "ENK-HARD-001", "met": true }` for 001–005 and 008 |
| `installed_as` | `image` \| `package` (RFC 005 ENK-INST-005) |
| `radios` | list of `{ "band": "2g" \| "5g" \| "6g", "in_use": true }` |

**`update`**

| Field | Value |
|---|---|
| `installed` | `{ "version", "baseline" }` |
| `checks_enabled` | boolean (ENK-UPD-008) |
| `last_check` | `{ "ago", "result": "ok" \| "unreachable" \| "bad_signature" \| "bad_index" }` or `null` |
| `available` | `null` or `{ "version", "baseline", "changes_baseline", "size_kib", "notes_url", "takes_about": 120 }` |
| `state` | `idle` \| `downloading` \| `verifying` \| `installing` \| `failed` |
| `failed_reason` | `bad_signature` \| `wrong_device` \| `wrong_baseline` \| `not_enough_memory` \| `change_pending` \| `download_failed` \| `health_check_failed` \| `null` |
| `last_result` | `done` \| `failed` \| `null` (the outcome of the previous update, ENK-UPD-005) |
| `can_undo` | `false` on single-slot devices (ENK-UPD-006); stated so the UI can say "this cannot be undone from here" |

**`management_key`**: `name`, `key`.

### 5. Errors

One shape, one code per cause.

```json
{ "error": { "code": "invalid", "fields": [ { "field": "guest.name", "code": "too_long", "limit": 29 } ], "message": "guest.name is 31 bytes; at most 29 in mode open_to_all" } }
```

| HTTP | Code | When |
|---|---|---|
| 400 | `invalid` | A value fails validation; `fields` lists each with its own code: `required`, `unknown_field`, `wrong_type`, `too_short`, `too_long`, `not_allowed`, `forbidden_characters`, `common_password`, `equals_network_name`, `equals_key`, `equals_device_id`, `not_in_choices` |
| 400 | `cannot_update` | `update/install` refused by ENK-UPD-003; `reason` carries one of the `failed_reason` values |
| 401 | `no_session` | No token, unknown token, or an ended session |
| 401 | `wrong_password` | `login`, `password`, `erase` |
| 403 | `wrong_state` | The resource is not offered in the current state; `state` is included |
| 403 | `not_available` | The device profile lacks what the action needs (identify) |
| 404 | `unknown_resource` | Any other path under `/api/` |
| 405 | `method_not_allowed` | Wrong method for a known path |
| 409 | `setup_in_progress_elsewhere`, `no_proposal`, `version_mismatch`, `change_pending`, `nothing_to_keep`, `nothing_to_install`, `update_in_progress` | State conflicts; `version_mismatch` includes the current `settings`, `change_pending` the current `change` |
| 413 | `too_large` | Body over the limit of §6 |
| 415 | `wrong_content_type` | — |
| 421 | — | `Host` is not the device (RFC 003 ENK-SEC-009); empty body |
| 429 | `try_later` | Login delay (ENK-OWN-012); `Retry-After` header and `retry_in` in the body |
| 500 | `internal` | A defect; the body holds a short reference the log also carries, never a stack trace |
| 503 | `busy` | No worker available (§6); `Retry-After: 1` |

| ID | Requirement | Verification |
|---|---|---|
| ENK-API-016 | Every error response shall use this shape and one of these codes. A new code requires an RFC. | Schema test |
| ENK-API-017 | Validation shall report every failing field at once, not the first only. | Test with two invalid fields |
| ENK-API-018 | A `wrong_password` response shall take the same time whether the password is close or not, and shall not say which rule a refused new password broke beyond the field codes above. | Timing test |

### 6. Limits (RFC 007 threat 23)

| ID | Requirement | Verification |
|---|---|---|
| ENK-SEC-017 | Request line and headers together shall not exceed 8 KiB; a request body shall not exceed 16 KiB. Larger requests are refused with 431 or 413 and the connection closed. | Test |
| ENK-SEC-018 | The daemon shall close a connection that has not delivered a complete request within 10 seconds, and shall finish or abandon any response within 30 seconds. | Test with a slow client |
| ENK-SEC-019 | The daemon shall serve at most 8 connections at a time with at most 4 worker threads; further connections wait in the listen queue up to 16 and are then refused by the kernel. | Test with 32 concurrent clients; memory stays within RFC 010 ENK-FOOT-007 |
| ENK-SEC-020 | The daemon shall accept no request body on `GET`, no chunked request bodies, no `Expect: 100-continue`, no compression, and no HTTP version other than 1.1 and 1.0. | Test |
| ENK-SEC-021 | The API shall accept no file upload. The update image is fetched by the device from the release index (RFC 013), never pushed by a client. | Inspection |
| ENK-SEC-022 | Response bodies shall not exceed 64 KiB. | Test |

Why these numbers: one operator on one phone produces at most a few requests per second; the
largest legitimate body is a `settings` document under 1 KiB; the largest response is `device`.
The numbers leave room for a second client (RFC 003 ENK-MGMT-006) and keep a hostile client on the
management network from holding memory.

### 7. Static files and headers

| ID | Requirement | Verification |
|---|---|---|
| ENK-API-019 | `GET /` and `GET /index.html` shall serve the UI's entry page with `Cache-Control: no-store`; every other UI file shall be named with a hash of its content and served with `Cache-Control: max-age=31536000, immutable`. | Inspection (RFC 003 ENK-MGMT-005) |
| ENK-API-020 | Every response, API or file, shall carry: `Content-Security-Policy: default-src 'self'; img-src 'self' data:; frame-ancestors 'none'; base-uri 'none'; form-action 'none'`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`, `Cross-Origin-Opener-Policy: same-origin`, `Cross-Origin-Resource-Policy: same-origin`. No `Access-Control-*` header is ever sent; an `OPTIONS` request is answered 405. | Header inspection (RFC 003 ENK-SEC-011, 012) |
| ENK-API-021 | A path under `/` that is not a UI file and not under `/api/` shall serve the entry page (the UI routes on the client) with the headers above; a path under `/api/` that is unknown shall answer 404 `unknown_resource`. | Test |
| ENK-API-022 | The guest notice (RFC 012 ENK-GUEST-001) is a page of the UI, rendered from `status.guest.name`, and needs no resource of its own. | Inspection |

`img-src data:` exists for the QR code of the address (ENK-MGMT-009), which the UI draws itself.

### 8. Polling

There is no push channel: no WebSocket, no server-sent events. The UI polls.

| Screen | Resource | Interval |
|---|---|---|
| Status | `status` | 5 s |
| While `change.state` is `validating`, `applying`, or `waiting_confirmation` | `change` | 1 s |
| While `update.state` is not `idle` | `update` | 2 s |
| During setup, before confirm | `claim` | 30 s (for `window_ends_in`) |

| ID | Requirement | Verification |
|---|---|---|
| ENK-API-023 | The daemon shall answer `status` from readings at most 2 seconds old and shall not query ubus more than once per 2 seconds for it, whatever the number of clients. | Test with two clients polling |

A push channel would cost a long-lived connection per client in a daemon with four threads; a
poll every few seconds is what the status screen needs.

### 9. Log lines

RFC 004 replaced audit events by log lines and left their content to a later RFC. This is it.

| ID | Requirement | Verification |
|---|---|---|
| ENK-API-024 | The daemon shall log to the platform's system log (`logd`, in memory) and shall write no log file of its own. | Inspection (D-15; RFC 010) |
| ENK-API-025 | It shall log, one line each: session created and ended (kind, cause); login refused (count of consecutive failures); claim started, confirmed, made final, abandoned; change started, settled, undone (id, kind, reason, detail); drift detected; update checked, started, failed, done; erase and restart requested; every 5xx with its reference. | Inspection after exercising every function |
| ENK-API-026 | A log line shall contain no station address, no client address, no name entered by the operator, and nothing listed in RFC 007 ENK-SEC-016. | Search of the log after exercising every function |

The management UI does not show the log. An expert reads it with the platform's tools after an
expert install (RFC 005 ENK-INST-005); for the operator, the status and the change's `reason`
and `detail` are the diagnosis.

### 10. Versioning and clients

| ID | Requirement | Verification |
|---|---|---|
| ENK-API-027 | A field may be added to a response shape, and a value may be added to an enumeration, only together with a UI release that handles it (they ship together, ENK-MGMT-004). A client that is not the device's UI shall treat an unknown enumeration value as "unknown" and shall not act on it. | Inspection |
| ENK-API-028 | Removing a field or changing a meaning shall increase the API version; `/api/1/` is then retired in the same release. The device never serves two API versions. | Inspection |

Two API versions on one device would be two ways to do one thing. The UI that belongs to the
daemon is always current (RFC 003); a separate client follows the greeting's `api`.

## Worked flows

**Setup, from the phone's point of view.** `GET /api/1/` → `state: SETUP`, `connection.encrypted`
shown first (ENK-OWN-022) → `POST claim/start` → `GET claim` → identify → `POST claim/check` with
country, name, mode, password → the UI shows the management name, key, and address and asks the
operator to keep them → `POST claim/confirm` → the phone loses the setup network within a few
seconds → the operator joins the management network, opens the same address → `GET /api/1/` →
`state: PENDING` → `POST login` → `state: CLAIMED`, `GET status`.

**Changing the guest name.** `GET settings` (version 7) → `POST settings` with version 7 and the
new name → 202 `change {state: applying, needs_confirmation: false}` → poll `change` every second →
`settled` → `status` shows the new effective name. On a health-check failure: `undone`,
`reason: health_check_failed`, `detail: guest_down`; the UI shows RFC 011 §8's sentence and the
previous settings are already back.

**Replacing the management key.** `POST management-key` → 202 `change {needs_confirmation: true}`
→ the Wi-Fi reloads; the phone drops → the UI shows the new key (it was in the `change`
response's companion `management_key`, see below) → the operator joins again, the session is
still valid (sessions survive a reload) → `POST change/keep` within `confirm_in` → `settled`.
If not: `undone`, `reason: not_confirmed`, the old key works again.

For the key case `POST management-key` answers with the `change` shape **and** a `management_key`
field holding the new key, because the operator needs it before the reload and `GET
management-key` after the reload may be unreachable. This is the only compound response.

## Consequences

| Affected | Change |
|---|---|
| Draft routes `GET /api/v1/device`, `/state`, `/config`, `/capabilities` | Replaced by `device`, `status`, `settings`, and the `claim` resource |
| Draft `PUT /config` + `POST /config/apply`, `GET /operations/{id}` | One operation `POST settings`; progress through `change` (GAP-050 closed) |
| Draft `pairing/begin`, `/complete`, `/revoke`, `GET /pairing` | `claim/start`, `claim/check`, `claim/confirm`; there is no revoke (RFC 004 ENK-OWN-019) |
| Draft `POST /factory-reset`, `/restart`, `/recovery/prepare` | `erase`, `restart`; no recovery resource (RFC 004 §8: one path) |
| Draft `POST /diagnostics`, `GET /diagnostics/{id}` | Withdrawn; `change.detail`, `device.hardening`, and the log replace them |
| RFC 004 §7 greeting | Gains `development` and `connection` (needed by ENK-HARD-007 and ENK-OWN-022, both accepted after RFC 004) |
| RFC 007 threat 23 | Covered by ENK-SEC-017 to 022; remaining risk: none known beyond the Wi-Fi layer (R-10) |
| RFC 007 open question 1 | Closed |
| RFC 003 open question 2 | Closed |
| RFC 011 "the API RFC names it" | `POST settings` |
| Gap analysis GAP-007, GAP-015, GAP-050, GAP-092 | Closed |
| Operator UI RFC | Maps every enumeration and error code of this RFC to wording in each language; nothing else may reach the screen |
| Browser harness (M1) | Runs against a mock that implements exactly these shapes |

## Alternatives considered

| Alternative | Why not |
|---|---|
| REST with `PUT`/`PATCH`/`DELETE` | Three more methods for distinctions the operator never makes; `PATCH` invites partial documents and silent merges (RFC 011 ENK-CFG-001 forbids merging). |
| Operator wording in API responses, with `Accept-Language` | Two places for one sentence; the UI already ships every language. |
| Absolute timestamps | The device's clock may be wrong by years at first boot. |
| Server-sent events for change progress | A held connection per client in a four-thread daemon; polling at 1 s is indistinguishable on a phone. |
| A log resource for the operator | Logs are the stock product's answer; Enkrnett's is a sentence with a reason. Experts have the platform's tools. |
| Cookies with `SameSite=Strict` | A cookie is a credential the browser attaches by itself; RFC 003 ENK-SEC-010 rules it out. |
| Separate `claim/check` body repeated in `claim/confirm` | The password would travel twice; a held proposal is simpler and matches "check, then keep". |

## Open questions

| # | Question | Closed by |
|---|---|---|
| 1 | Whether `Host` also accepts a fixed name (RFC 003 ENK-MGMT-008) | M-1 |
| 2 | The UI normalises names to Unicode NFC before sending, so that byte counts match what the operator sees | Operator UI RFC |
| 3 | The health-check `detail` codes beyond those listed, derived from the internal design of ENK-CFG-007 | M1 internal design; added here by amendment |
| 4 | The exact JSON Schema files, generated from `enkrnett-model` and used by the UI and the harness | M1 handoff |
