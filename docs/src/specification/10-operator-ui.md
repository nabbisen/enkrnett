# 10. Operator UI

From RFC 017. The sentence catalogue (every screen's English and Japanese text) is RFC 017 §5 and
becomes the two catalogue files under `ui/` in milestone M1; this chapter restates the rules and
the words, and points to the catalogue for the sentences.

## 10.1 Principles of the screens

| # | Rule |
|---|---|
| 1 | The status screen is the first screen after login and the one the operator returns to. |
| 2 | One decision per screen in setup; one action per screen for everything that disturbs the Wi-Fi or cannot be undone, with the consequence beside the button. |
| 3 | Words come from the display-form table (chapter 1 §4). The phone's Wi-Fi screen and Enkrnett's screen call the same thing by the same name. |
| 4 | A number is explained before it is typed. |
| 5 | Colour means running or not running, nothing else. |
| 6 | The screen never waits silently: countdowns from the device's seconds; a failed poll shows "not connected" and continues by itself. |
| 7 | No technical names except the two allowed in the guest notice. |

## 10.2 Screens

| # | Screen | State | Reads | Acts |
|---|---|---|---|---|
| S-1 | Start (connection line; *Start*) | `SETUP` | greeting | `claim/start` |
| S-2 | Is this the device? (*Blink*) | `SETUP` | `claim` | `claim/identify` |
| S-3 | Country (only when not fixed) | `SETUP` | `claim` | — |
| S-4 | Guest Wi-Fi: name with room counter; the two modes | `SETUP` | `claim` | — |
| S-5 | Owner password | `SETUP` | `claim` | `claim/check` |
| S-6 | Keep these: management Wi-Fi name and password, Wi-Fi QR, address and QR; *Set up* | `SETUP` | — | `claim/confirm` |
| S-7 | Join the management Wi-Fi: three numbered steps, 10-minute countdown; continues by itself | after confirm | greeting | — |
| S-8 | Log in | `PENDING`, `CLAIMED` | greeting | `login` |
| S-9 | Status | `CLAIMED` | `status` | — |
| S-10 | Settings (guest on/off, name, mode; management Wi-Fi name; *Save*; drift banner; link to Erase) | `CLAIMED` | `status` | `settings`, `settings/restore` |
| S-11 | Change in progress (a state over S-9/S-10) | `CLAIMED` | `change` | — |
| S-12 | Management Wi-Fi (name, password on request, Wi-Fi QR, address; *Replace the password*) | `CLAIMED` | `management-key` | `management-key` |
| S-13 | Erase (owner password; one button) | `CLAIMED` | — | `erase` |
| S-14 | Details (version, update, model, identifier, Internet connection, hardening report, change owner password, Restart, Log out) | `CLAIMED` | `device`, `update` | `update/check`, `update/install`, `password`, `restart`, `logout` |
| S-15 | What this protects / Guest notice (print view) | `CLAIMED` | `status` | — |

```text
S-9 Status ──► S-15 What this protects / Guest notice
   │  menu ──► S-10 Settings ──► (bottom) S-13 Erase
   │        ──► S-12 Management Wi-Fi
   │        ──► S-14 Details ──► Change owner password · Restart · Log out · Update
   └── S-11 Change in progress (shown over S-9 / S-10 while a change runs)
```

## 10.3 The catalogue

Every sentence the operator reads, in English and Japanese, keyed by the RFC 015 value or the
screen element that selects it, is RFC 017 §5: status lines (§5.1), names (§5.2), passwords
(§5.3), the connection line for `connection.security` (§5.4), change states, reasons, and details
(§5.5), update (§5.6), Erase and Restart (§5.7), "What this protects" and the guest notice (§5.8),
channel, connection, and every error code (§5.9). Each entry of §5 is one catalogue key; an API
value that no entry names renders the last row of §5.9.

## 10.4 The word check

The UI build fails when a catalogue string contains any word of chapter 1 §4 "Words that never
appear in operator-facing text", in either language. `OWE` and `Enhanced Open` are allowed once,
in the guest notice entry only.

## 10.5 Requirements

{{#include requirements/ui.md}}

## 10.6 Open

| # | Question | Closed by |
|---|---|---|
| 1 | The client-compatibility sentence (ENK-OWE-019) | M-4a, M-4 |
| 2 | Whether the phones' "stay connected without Internet?" dialogs need a sentence in S-7 and the guides | M-1 rows 13 to 15 |
| 3 | Whether a name is shown beside the address in S-6 and S-12 (ENK-MGMT-008) | M-1 |
| 4 | Wording changes found with non-experts | M-7 |
| 5 | Native-reader review of the Japanese catalogue (ENK-UI-011) | Release checklist |
