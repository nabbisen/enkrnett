# UI internal design (milestone M1, stage 2)

The management UI as built in `ui/`, and the browser harness in `tests/browser/`. The external
design is specification chapter 10 (RFC 017); this document says how it is put together.

## 1. What this stage delivers

| Delivered | Not yet |
|---|---|
| Every screen of chapter 10 §2 (S-1 to S-15) in English and Japanese | Native-reader review of the Japanese catalogue (ENK-UI-011; release checklist) |
| The two catalogues as JSON, one key per sentence of RFC 017 §5, with named placeholders and the languages' plural rules | The compatibility sentence after M-4a and M-4 (ENK-OWE-019) |
| The word check that fails the build on any forbidden word (ENK-WORD-001, ENK-UI-008) | The real iOS Simulator and Android emulator in CI; WebKit and Chromium at phone size stand in |
| Name fields that normalise to NFC and count bytes against the device's limits (ENK-UI-015) | The sentence for the phones' "stay connected without Internet?" dialogs (M-1) |
| QR codes for the management Wi-Fi and the address, drawn in the page (ENK-UI-025) | |
| Countdowns from the device's seconds and a monotonic clock (ENK-UI-018) | |
| The "not connected" path that continues by itself (ENK-UI-019) | |
| The browser harness: 14 tests against `enkrnettd --mock` with the built UI | |

## 2. Stack and size

Svelte 5 (runes) with TypeScript, built by Vite into `ui/dist/`: one script, one stylesheet,
one entry page, one icon. No UI library, no icon font, no web font, one dependency at run time
(`uqr`, the QR encoder, about 4 KiB). The daemon serves `dist/` with `--ui-dir`.

| Measure | Value | Budget |
|---|---|---|
| Bundle, both languages, xz-compressed (the squashfs stand-in) | 44.3 KiB | 150 KiB (ENK-FOOT-002) |
| Raw | 149.5 KiB | — |

`npm run build` runs the word check, the build, and the size check in that order and fails on any
of them. Vite writes hex content hashes (`index-f2ec8e19.js`) because the daemon treats a hex run
of eight or more as content-hashed and immutable (ENK-API-024).

## 3. Layout of `ui/src`

| Path | Responsibility |
|---|---|
| `main.ts`, `App.svelte` | Mounts; chooses the view from the device's state and the session: `setup`, `join`, `login`, `main`, or the loading and not-connected states |
| `lib/api.ts` | One function per resource of chapter 9 §3; the token in `Enkrnett-Session`; `ApiError` with the one error shape; `NotConnected` for a failed connection or a 421 |
| `lib/types.ts` | The shapes of chapter 9 §4, hand-written until the schema is generated from `enkrnett-model` |
| `lib/app.svelte.ts` | Shared state: greeting, status, route (by path, with the daemon's fallback to the entry page), the proposal during setup, the management name for the not-connected sentence |
| `lib/i18n/` | `en.json`, `ja.json`; `index.svelte.ts` with `t()`, `n()` (plural), the browser-language default, the remembered choice that is only a convenience |
| `lib/errors.ts` | Error code, field code, and change state to sentence (ENK-UI-017); unknown values to the generic sentence with a console warning |
| `lib/names.ts` | NFC, UTF-8 byte count, room left, sendability (ENK-UI-015) |
| `lib/countdown.svelte.ts` | `set(seconds)` on every poll; ticks on `performance.now()` |
| `lib/wifi.ts` | The `WIFI:` payload with escaping; the address URL |
| `components/` | `Header` (language switch, development banner), `NameField`, `PasswordField` (show/hide, autocomplete hints), `Qr`, `ConnectionLine`, `ChangeProgress` (S-11), `NotConnectedBar` |
| `screens/` | `Setup` (S-1 to S-6 as one component with one decision per step), `Join` (S-7 with the login form once the device answers from PENDING), `Login` (S-8), `Main` (menu, polling, the route switch), `Status`, `Settings`, `Management`, `Erase`, `Details`, `Protects` (screen and print view) |
| `app.css` | Tokens, system fonts, `rem` sizes, 44 px targets, dark appearance from the system, colour only on the running dot, print styles |

## 4. Behaviour worth knowing

- **State decides the screen.** The greeting is read at start and re-read on `wrong_state`; the
  view follows `state` and whether a session token exists. In PENDING with a proposal in memory
  the screen is S-7; without one (the page was reopened) it is the login screen.
- **Setup persists nothing on the device until "Set up".** The proposal from `claim/check` is
  held in memory; the claim session is kept alive by polling `claim` every 30 seconds.
- **Polling.** `status` every 5 seconds on the claimed screens; the greeting every 3 seconds
  while there is none or during S-7. A failed poll sets `notConnected`; the next success clears
  it; nothing else changes. While `update.state` is not `idle`, the details screen reads `update`
  every 2 seconds (chapter 9 §7) and shows Downloading…, Verifying…, or the installing sentence;
  when the device goes away to write the image the sentence stays and the not-connected bar
  appears; when it comes back, the ended session sends the owner to the login screen, and the
  details screen then shows "Updated to Enkrnett {v}." or the failure (`last_result`).
- **The update's own step (ENK-UI-013; RFC 017 §5.6).** The update block shows "Update
  available: Enkrnett {v}" and one *Update* button; pressing it opens the step: the full sentence
  (version, minutes, the Wi-Fi stops, cannot be undone from here), the baseline sentence when
  `changes_baseline`, the primary *Update*, and *Cancel*. `400 cannot_update` shows the sentence
  of its `reason` (the same catalogue as `last_result.reason`); `install_failed` is a reason the
  accepted API lacks (`daemon.md` §6c).
- **Errors.** A field error goes to its field (and, in setup, the wizard returns to that step);
  `version_mismatch` reloads the settings and says so; `no_session` returns to the login screen;
  `try_later` shows the countdown from `retry_in`.
- **Storage.** The token lives in memory and the tab's session storage; the language in local
  storage; both wrapped in try/catch and never required (ENK-UI-023).
- **Printing.** The notice route renders the four paragraphs alone with print styles; the button
  on "What this protects" navigates there and opens the print dialog.

## 5. One sentence differs from RFC 017

RFC 017 §5.6 wrote the Japanese for `update.failed.wrong_baseline` with 適用, a word on the
RFC's own forbidden list (§8, "apply"). The word check caught it; the catalogue reads
「このアップデートは現在のバージョンから直接にはインストールできません。」 The catalogue is the
source of truth for sentences from here on; the RFC sentence is corrected in the next amendment
RFC (ENK-SCOPE-006).

## 6. The browser harness

`tests/browser/`, Playwright. `global-setup.ts` starts `enkrnettd --mock --development` with the
built UI on port 18080 and waits for the greeting; it first makes a signed release index and a
pretend image with `enkrnett-release` and hands them to the mock as `file://` URLs (the key
beside the index is the one the mock's "image" carries), so that an update can be offered and
started. `update.spec.ts` runs last: once the owner confirms, the mock's root side answers
"flashing" and the daemon stays in the update, refusing every other POST, as a device does until
it restarts. The tests reset the device through the API
(erase when claimed) and must never leave a claim session behind, because the device rightly
refuses the next *Start* for five minutes.

| File | Covers |
|---|---|
| `setup.spec.ts` | The whole claim in the browser in English: the connection line, the development banner, the name counter with Japanese characters, a refused common password, the proposal with the key form and both QR codes, S-7 with the countdown, login, the status screen; a second phone refused; the Japanese setup screens |
| `claimed.spec.ts` | Status with no tap and no external request; Settings with the mode warning, a settled save, the management-name notice; Management Wi-Fi with the password on request and the replace step; Details with the hardening report and Restart on its own step; What this protects and the printable notice; Erase with the wrong and the right password |
| `resilience.spec.ts` | Not connected and recovery; storage blocked; no horizontal scroll and 44 px targets at 360 px; an axe audit (WCAG 2 A and AA) of the start, login, and status screens; the language from the browser |

Projects: `chromium-phone` (360 × 740) and `webkit-phone` (430 × 932). Every rendered screen is
also checked for the forbidden words. Run with `npm test` in `tests/browser/` after building the
UI; `ENKRNETTD` may point at a prebuilt daemon.
