# 9. Management API

From RFC 015 (as accepted after self-review 12), as amended by RFC 016 (`connection.security`;
600 s; `stale_index`).

One HTTP/1.1 API, JSON only, under `/api/1/`. Reads are `GET`; everything that changes something
is `POST`. Two reads carry the whole UI: `status` and `device`. One write, `POST settings`, saves
the settings and starts the transaction of chapter 6. A change that needs the owner's
confirmation is confirmed by the owner joining the new management Wi-Fi. Every shape is defined
once and embedded whole; every error has one shape with a stable code; every duration is a
number of seconds relative to now; no sentence meant for an operator travels in the API.

## 9.1 Conventions

{{#include requirements/api.md}}

## 9.2 Authorization by state

| Resource | `UNCLAIMED` | `SETUP` | `PENDING` | `CLAIMED` |
|---|---|---|---|---|
| UI files, greeting | no network | anyone | anyone | anyone |
| `claim/*` | — | claim session | — | — |
| `login` | — | — | anyone (makes the claim final) | anyone |
| everything else | — | — | — | owner session |

`connection.security` is read from the requesting address: neighbour table to station address,
station address to the BSS and its security (`hostapd.* get_clients`, `get_status`). `null` when
the daemon cannot determine it.

## 9.3 Resources

Paths relative to `/api/1/`.

| Path | Method | Who | Does | Returns |
|---|---|---|---|---|
| `` | GET | anyone | Greeting | `greeting` |
| `claim/start` | POST `{}` | anyone, in `SETUP` | Grants the claim session to this client if none exists | 200 `session`; 409 `setup_in_progress_elsewhere` |
| `claim` | GET | claim | Country choice, limits, defaults, identify availability, remaining window | `claim` |
| `claim/identify` | POST `{}` | claim | Blinks the light | 200 `{}`; 403 `not_available` |
| `claim/check` | POST | claim | Validates country, guest name and mode, owner password; generates the management name and key in memory; persists nothing | 200 `proposal`; 400 `invalid` |
| `claim/confirm` | POST `{}` | claim | State `PENDING`; setup network ends; management network starts | 200 `proposal`; 409 `no_proposal` |
| `login` | POST | anyone | Creates an owner session; in `PENDING` makes the claim final | 200 `session`; 401 `wrong_password`; 429 `try_later` |
| `logout` | POST `{}` | owner | Ends this session | 200 `{}` |
| `status` | GET | owner | The first screen: settings, effective state, the current change, drift, version, update | `status` |
| `settings` | POST | owner | **Save**: validates and starts the transaction | 202 `change`; 400 `invalid`; 409 `version_mismatch`, `change_pending` |
| `settings/restore` | POST `{}` | owner | Rewrites the owned sections from the settings (drift) | 202 `change`; 409 `change_pending` |
| `change` | GET | owner | The current or the last transaction | `change` |
| `management-key` | GET | owner | Shows the management key | `management_key` |
| `management-key` | POST `{}` | owner | Replaces the key: a transaction that needs confirmation | 202 `change` plus `management_key`; 409 `change_pending` |
| `password` | POST | owner | Changes the owner password; ends every session | 200 `{}`; 400 `invalid`; 401 `wrong_password` |
| `device` | GET | owner | The details screen | `device` |
| `update` | GET | owner | Installed and available release, last check, a running update, the last outcome | `update` |
| `update/check` | POST `{}` | owner | Fetches the release index now | 200 `update` |
| `update/install` | POST `{}` | owner | Starts the update | 202 `update`; 400 `cannot_update`; 409 `change_pending`, `nothing_to_install` |
| `restart` | POST `{}` | owner | Restarts after answering | 202 `{}` |
| `erase` | POST | owner | Erase: the owner password again; erases and restarts | 202 `{}`; 401 `wrong_password` |

While `update.state` is not `idle`, every `POST` except `logout` answers 409 `update_in_progress`
(ENK-API-012).

## 9.4 Shapes

Integers only; enumerations are closed.

| Shape | Fields |
|---|---|
| `version` | `enkrnett`, `baseline`, `openwrt`, `api` |
| `connection` | `network` (`setup` \| `management`), `security` (`none` \| `owe` \| `wpa2` \| `wpa3` \| `null`) |
| `limits` | `guest_name_max_bytes { open_to_all: 29, encrypted_only: 32 }`, `management_name_max_bytes` (32), `password { min: 10, max: 64 }` |
| `greeting` | `product` ("Enkrnett"), `api` (1), `state`, `languages`, `development`, `connection` |
| `session` | `token` (≥ 128 bits, base64url), `kind` (`claim` \| `owner`), `state`, `idle_ends_in`, `ends_in` |
| `claim` | `window_ends_in`, `country { fixed, choices }`, `guest_mode_default`, `identify_available`, `limits`, `connection` |
| `claim/check` request | `country` (omitted when fixed), `guest { name, mode }`, `owner_password` |
| `proposal` | `management { name, key }`, `address`, `guest { name, mode }`, `country`, `state`, `final_in` (`null` after check; 600 after confirm) |
| `settings` | `version`, `country`, `guest { name, mode, enabled }`, `management { name }`, `limits`; `POST settings` sends `version`, `guest`, `management` |
| `change` | `id` (0 when `none`), `state` (`none` \| `validating` \| `applying` \| `waiting_confirmation` \| `settled` \| `undone`), `kind` (`settings` \| `restore` \| `management_key`), `from_version`, `to_version`, `needs_confirmation`, `confirm_in` (≤ 600), `reason` (`health_check_failed` \| `not_confirmed` \| `restart`), `detail` (`guest_down` \| `management_down` \| `isolation_missing` \| `dhcp_missing` \| `listener_exposed` \| `pmf_missing` \| `mode_differs`), `started_ago`. When `state` is `none`, every other field but `id` is `null`. |
| `status` | `state`, `version`, `settings`, `guest { up, name, mode, clients_encrypted, clients_unencrypted, checked_ago }`, `management { up, name, address, clients }`, `uplink { up, address }`, `change`, `drift`, `update`, `hardening_met` |
| `device` | `model`, `revision`, `tier`, `device_class`, `device_id`, `version`, `uptime_seconds`, `memory { total_kib, available_kib }`, `storage { overlay_total_kib, overlay_free_kib }`, `hardening [ { id, met } ]` (ENK-HARD-001 to 005, 008), `installed_as` (`image` \| `package`), `radios [ { band, in_use } ]` |
| `update` | `installed { version, baseline }`, `checks_enabled`, `last_check { checked_ago, result: ok \| unreachable \| bad_signature \| bad_index \| stale_index }`, `available` (`null` or `{ version, baseline, changes_baseline, size_kib, notes_url, install_seconds }`), `state` (`idle` \| `downloading` \| `verifying` \| `installing`), `last_result` (`null` or `{ result: done \| failed, reason }`), `can_undo` |
| `management_key` | `name`, `key` |

`status.guest`: when `up` is `false`, `name` and `mode` are `null` and the counts `0`; in
`encrypted_only`, `clients_unencrypted` is `0`, never `null`.

## 9.5 Errors

```json
{ "error": { "code": "invalid",
             "fields": [ { "field": "guest.name", "code": "too_long", "limit": 29 } ],
             "message": "developer text, never shown" } }
```

| HTTP | Code | When |
|---|---|---|
| 400 | `invalid` | Field codes: `required`, `unknown_field`, `wrong_type`, `too_short`, `too_long`, `not_allowed`, `forbidden_characters`, `common_password`, `equals_network_name`, `equals_key`, `equals_device_id`, `not_in_choices` |
| 400 | `cannot_update` | `update/install` refused by ENK-UPD-003 or 009; `reason` as in `last_result.reason` |
| 401 | `no_session`, `wrong_password` | |
| 403 | `wrong_state` (with `state`), `not_available` | |
| 404 | `unknown_resource` | Any other path under `/api/` |
| 405 | `method_not_allowed` | Including `OPTIONS` |
| 409 | `setup_in_progress_elsewhere`, `no_proposal`, `version_mismatch` (with `settings`), `change_pending` (with `change`), `nothing_to_install`, `update_in_progress` | |
| 413 / 431 | `too_large` | Body / headers over the limit |
| 415 | `wrong_content_type` | |
| 421 | — | `Host` is not the device; empty body |
| 429 | `try_later` | `Retry-After` and `retry_in` |
| 500 | `internal` | `reference` also in the log; never a stack trace |

## 9.6 Limits

Included in `requirements/sec.md` (chapter 3) as ENK-SEC-017 to 022: 8 KiB headers, 16 KiB body,
10 s to deliver a request and 30 s to answer, `Connection: close` on every response, 4 workers,
listen backlog 16, HTTP/1.1 only, strict JSON (no duplicate keys, nesting ≤ 8, 64-bit integers),
no file upload, responses ≤ 64 KiB.

## 9.7 Confirmation, headers, polling, logs, versioning

Stated as ENK-API-021 to 032 above. In short: after the health check of a change that needs
confirmation, the daemon disconnects every management station and then waits up to 600 s for the
first owner request on a connection accepted after that; every `/api/` response is `no-store`;
the entry page is `no-store` and every other UI file content-hashed and immutable; the security
headers of ENK-API-025 on every response; the UI polls (`status` 5 s, `change` 1 s, `update` 2 s,
`claim` 30 s) and the daemon answers `status` from readings at most 2 s old; log lines go to
`logd` with no addresses, names, or secrets; a field or value is added only with the UI that
handles it, and a breaking change raises the API version and retires the old prefix in the same
release.
