# M-7 — Usability test plan: install, set up, change, update, erase, recover with the guides alone

| | |
|---|---|
| **Measurement** | M-7 (RFC 006; RFC 005 criteria 2, 3 and 10; chapter 14 §1 *Usability test*) |
| **Prepared by** | The architect, 2026-11-27 (M3 stage 4) |
| **Carried out by** | The owner, with participants who are not IT specialists |
| **Needs** | A Tier 1 candidate router in the state in which it is sold; the lawful basis to operate it with replaced firmware at the place of the test (chapter 5 §5.4); three or more participants; the English guides of `docs/src/guides/` |
| **State** | Written; blocked by the router and the legal answer (handoff README) |
| **Report** | `docs/src/measurements/m-7-usability.md` on the report template |

This plan extends the M-7 section of `implementation-handoff.md` from "install and recover" to
every task the guides cover, as chapter 14 §1 requires before 0.3.0: install, set up, change,
update, erase, with the guides alone. It also lists the facts the guides mark *to be confirmed*,
which the first unit settles before any participant sits down.

## 1. Before the participants: facts the first unit settles

The guides were written from OpenWrt's device support and the vendors' public specifications,
not from a unit. The owner, with the first router at hand, checks each item below and the
architect corrects the guide **before** the first session; a participant must never be the one
who finds a wrong menu name.

| # | Fact | Where it goes |
|---|---|---|
| F-1 | Which code on the label the device's setup Wi-Fi name ends in (the profile says the WAN address) | `openwrt/profiles/<device>/profile.json` `label_address`; the install guide, *Before you start* |
| F-2 | The vendor firmware's address, login and the menu path to the firmware update page | Install guide, steps 2 and 3 |
| F-3 | The vendor firmware version the unit shipped with, and that it accepted the `factory.bin` | Install guide, *Before you start* (ENK-INST-003); the profile's `install.tested_vendor_firmware` |
| F-4 | The colour and behaviour of the power light while writing, starting, running, and when *Blink* is pressed | Install guide, *What the light does*; the profile's `light.led` |
| F-5 | The vendor's recovery procedure after an interrupted write (button, address, file) | `recover.md`, *An installation that was interrupted* (ENK-INST-004) |
| F-6 | Whether a phone alone suffices for the vendor's update page (file upload from a phone browser) | `index.md`, *What you need* |
| F-7 | The Buffalo mode switch's positions (ROUTER / AP / WB) and which one the install needs | Buffalo install guide, step 2 |
| F-8 | How long the install, the first boot, an update and Erase take on the device | The guides' *about n minutes*; the profile's `update.install_seconds` |

## 2. Participants

Three or more people who fit the operator of chapter 1 §3: they run a place and use a phone, and
have no IT training. Nobody who took part in the project. Record occupation only (handoff rule);
no other personal data. Each participant does every session alone, with the device, the guide on
paper or on a second phone, and their own phone. **No help is given**; a question is written down
and answered with "what does the guide say?" only.

## 3. Sessions

Each session is one task with one guide. Between sessions the owner resets the device to the
state the next session starts from (column *Start state*). Time every session.

| Session | Guide | Start state | Done when | Pass (per participant) |
|---|---|---|---|---|
| S1 Install | `install-<device>.md` | Vendor firmware, as sold, cabled to the Internet router | The setup Wi-Fi `Enkrnett Setup XXXX` is seen on the phone | Finished without help (RFC 005 criterion 2); the label step was done before anything else (ENK-INST-002); the right four characters were noted (criterion 10) |
| S2 Set up | `set-up.md` | Just installed, setup Wi-Fi open | The Status screen shows the guest Wi-Fi running | Finished without help in at most 10 minutes excluding the Wi-Fi joins (ENK-UI-012); the two passwords told apart when asked (handoff step 7); the management Wi-Fi joined within the ten minutes (handoff step 8 timing) |
| S3 Change | `set-up.md`, *Afterwards* | Set up | The guest Wi-Fi's name changed and saved; then the mode changed to *Encrypted only* and back | Finished without help; the room counter understood (ENK-CFG-017: ask what "{n} left" means) |
| S4 Update | `update.md` | Set up; a signed test release served as in `tests/vm/update-install.sh` (a test key in the image for the trial; never the project's key on the test server) | *Details* says *Updated to Enkrnett x.y.z* and the guest Wi-Fi is back | Finished without help; the participant could say beforehand what will happen (the Wi-Fi stops, cannot be undone from here: ENK-UPD-002) |
| S5 Erase from the screen | `erase.md` | Set up | The setup Wi-Fi is back | Finished without help; the participant could say what is removed |
| S6 Erase with the button | `erase.md`, `recover.md` *I lost the owner password* | Set up; the owner password withheld from the participant | The setup Wi-Fi is back | Finished without help; a short press was not mistaken for Erase (ENK-OWN-021) |
| S7 Recover an interrupted install | `recover.md`, *An installation that was interrupted* | Vendor firmware; the owner interrupts the write by cutting power at a moment chosen in advance (after the upload is accepted, during the blinking) | Either Wi-Fi is back, by the guide's steps 1 to 4 | Finished without help, **or** the device kept its previous firmware by itself (RFC 005 criterion 3 allows either) |
| S8 Words | none | After S2 | — | Handoff step 6: the browser's warning beside the address; what the participant thinks it means. The notice *What this protects* read aloud: what the participant believes is encrypted and what is not (ENK-OWE-015 wording) |

S7 is destructive and is run last, once per device, by the owner's choice; if the device has no
documented recovery path, S7 is recorded as "not run: no recovery path" and criterion 3 rests on
the device keeping its firmware when the vendor screen refuses a file (S1 with a wrong file,
which is also recorded).

## 4. What is recorded

Per participant and session: finished without help or not; time; every hesitation, error and
question, word for word; what the participant said when asked. Per device: F-1 to F-8 of §1;
whether a phone alone sufficed; whether the vendor firmware accepted the image. Per guide: the
sentences that were misread, and the proposed rewording.

Timing, in seconds, for ENK-FOOT-011 and the guides' minutes: first screen of the settings screen
on the participant's phone; install from the vendor screen's *start* to the setup Wi-Fi; set up
from *Start* to the Status screen; update from *Update* to the guest Wi-Fi back.

## 5. Pass criteria for the milestone

| Criterion | Met when |
|---|---|
| RFC 005 criterion 2 | Every participant finished S1 without help on the device |
| RFC 005 criterion 3 | Every participant finished S7 without help, or the device kept its firmware |
| RFC 005 criterion 10 | The four characters each participant noted match the setup Wi-Fi seen |
| ENK-UI-012 | Every participant finished S2 within 10 minutes excluding the Wi-Fi joins |
| ENK-INST-002, 004 | Observed in S1 and S7 |
| Chapter 14 §1 usability test | S1 to S7 run with the guides alone; the wording changes listed |

A session that one participant could not finish does not fail the milestone by itself: the guide
is corrected and that session is run again with one new participant. Two failures of the same
session after a correction do.

## 6. What changes afterwards

- The guides: every misread sentence, every confirmed fact F-1 to F-8; the *to be confirmed*
  markers removed as they are settled; `install.tested_vendor_firmware` filled.
- The UI wording found with non-experts (RFC 017 §9 item 4): recorded for the next amendment RFC.
- The durations of ENK-OWN-002, 005, 008 reviewed (RFC 004 open item).
- The device profile: `verified.on_hardware`, `label_address`, `light.led` confirmed; a Tier 1
  promotion needs the rest of chapter 5 §5.1 (M-3, M-4) and the owner's decision.
- The report on the template, `docs/src/measurements/m-7-usability.md`; the measurement index
  row; the 0.3.0 release notes' *Open* table.
