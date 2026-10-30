# RFC 009 — Vocabulary amendment: Erase, Restart, and operator-facing words

**Status.** Accepted (2026-10-29)
**Tracks.** Amendments A-1 and A-2 of the self-review of 2026-10-29, accepted by the owner.
**Touches.** [RFC 001](./001-scope-people-vocabulary.md) §4 (vocabulary);
[RFC 004](./004-ownership-credentials-authorization.md) §8 (the action formerly called
Reset); every operator-facing text: UI, guides, status.

## Summary

Two words in the accepted vocabulary can be misread by the people the product is for. **Reset**
becomes **Erase**, **Restart** is added, and a short list of words that never appear in
operator-facing text is fixed. Identifiers and requirements keep their numbers; only the words
change.

## Motivation

| Problem | Evidence |
|---|---|
| RFC 001 defines *Reset* as "erase everything the operator configured". The physical button is also called "reset", and on OpenWrt a short press **restarts** while only a 5-second hold erases ([`rc.button/reset`][reset]). An operator who reads "reset the device" and taps the button gets a restart and concludes the device is broken. | OpenWrt source; self-review §2.7 |
| "Transition Mode" is a Wi-Fi Alliance term; its everyday meaning ("temporary") is wrong. | Self-review §2.5 |
| "Secure" and "not secure" are the browser's words and the words the product must not borrow (RFC 001 principle 4). | RFC 001; RFC 003 §5 |

## Amendments

### A. RFC 001 §4.2 and §4.3

| Term | Meaning | Replaces |
|---|---|---|
| **Erase** | Remove everything the operator configured. The device becomes unclaimed. | Reset |
| **Restart** | Power the device off and on again, or let it reboot. Nothing changes. | (new) |

Everywhere an accepted RFC says "Reset" or "reset" in the sense of erasing, read "Erase" /
"erase". The requirements ENK-OWN-017, 018, 019 and ENK-OWN-021 keep their identifiers; their
wording is updated when the consolidated specification is written (RFC 001 §5.1). The physical
button keeps the platform's name "reset button" in device profiles, because that is what the
label on the device says.

### B. Fixed Japanese terms for these two words

| English | Japanese |
|---|---|
| Erase (the action) | 初期化 |
| Restart | 再起動 |

The remaining Japanese terms are fixed in the Operator UI RFC, as RFC 001 open question 1 says.

### C. Words that never appear in operator-facing text

Operator-facing text means: every screen of the management UI, every guide, every status and
error message, the guest notice. "Details" views for experts are exempt when they are clearly
marked as such.

| Never | Use instead |
|---|---|
| secure, insecure, not secure, security (as a label) | the specific property: *encrypted*, *separated*, *protected on the radio link* |
| Transition Mode, transition | *Open to every device* (the mode) |
| OWE, Enhanced Open, WPA2, WPA3, PMF, SAE | *encrypted* / *not encrypted*; the standard's name may appear once in the guest notice, in brackets, because guests' phones show it |
| SSID, BSS, BSSID | *network name*, *network* |
| reset (as an action) | *erase*; for the button: "hold the button for five seconds until the light changes" |
| press reset | never |
| apply, commit, rollback | *save*, *saved*, *undone* |
| pairing, pair | *set up*, *claim* |

### D. Rule for guides

A guide never asks the operator to "press reset". It describes the physical act and the sign that
it worked: "Hold the small button on the back for five seconds. The light starts blinking. Let go."

## Requirements

| ID | Requirement | Verification |
|---|---|---|
| ENK-WORD-001 | Operator-facing text shall use the vocabulary of RFC 001 as amended by this RFC and shall contain none of the words in table C. | A word check in the UI and documentation build; review |
| ENK-WORD-002 | The action that removes everything the operator configured shall be called *Erase* in English and 初期化 in Japanese, and the UI shall always state its consequence beside it. | Inspection |
| ENK-WORD-003 | The UI shall offer *Restart* and *Erase* on different screens, never side by side. | Inspection |

## Consequences

| Affected | Change |
|---|---|
| RFC 001 §4 | Amended as above. |
| RFC 004 §8 | "Reset" reads "Erase". The flow is unchanged. |
| Draft documents and the gap analysis | Use the old word; no change needed, they are inputs. |
| The planned Operator UI RFC | Inherits table C as a hard rule; adds the remaining Japanese terms. |

## Alternatives considered

| Alternative | Why not |
|---|---|
| Keep "Reset" and explain the button in the guide | The confusion arises at the moment the operator is stressed; a word that cannot be misread is cheaper than an explanation. |
| "Factory reset" | Still contains "reset"; and the device has no factory state of Enkrnett's making. |
| Allow "Transition Mode" with a tooltip | A tooltip is read once; the word is read every day. |

## References

[reset]: https://github.com/openwrt/openwrt/blob/openwrt-25.12/package/base-files/files/etc/rc.button/reset

- OpenWrt, reset button handler: [source][reset]
