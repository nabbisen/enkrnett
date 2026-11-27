# Guides

For the person who runs a shop, a clinic, a classroom or any other place and wants to give
visitors a Wi-Fi of their own, without IT training. Each guide is a short list of steps for one
task. Read the one you need; nothing in the rest of this book is required.

| I want to… | Guide | Time |
|---|---|---|
| Put Enkrnett on a new router | [Install: ELECOM WRC-1167GST2](./install-elecom-wrc-1167gst2.md) · [Install: Buffalo WCR-1166DS](./install-buffalo-wcr-1166ds.md) | about 15 minutes |
| Give the device its name, its guest Wi-Fi and my password | [Set up](./set-up.md) | about 5 minutes |
| Install a newer Enkrnett | [Update](./update.md) | about 5 minutes; the Wi-Fi stops for 2 |
| Start over, or hand the device to someone else | [Erase](./erase.md) | about 2 minutes |
| Something went wrong | [Recover](./recover.md) | — |
| I run OpenWrt already and want Enkrnett as a package | [Expert install](./expert-install.md) | for experts |

## What you need

- The device named in the install guide, its power supply, and the network cable in its box.
- Your existing Internet router, with one free port.
- A phone. Everything is done on the phone; no computer and no app are needed. For the
  installation itself the router's own update screen may be easier on a computer.
- A pen, or the phone's screenshot button, for the two things you must keep (the management
  Wi-Fi password and the owner password).

## Words used in the guides

| Word | Meaning |
|---|---|
| **this device** | The router that runs Enkrnett. |
| **guest Wi-Fi** | The Wi-Fi your visitors join. It is separate from your own network. |
| **management Wi-Fi** | The Wi-Fi you join to change settings. Only you know its password. |
| **setup Wi-Fi** | The temporary Wi-Fi the device offers before it is set up. Its name ends in the four characters printed on the label. |
| **settings screen** | The pages you open on your phone to see and change what the device does. |
| **owner password** | What you enter to change anything. Keep it; if it is lost, the only way back is Erase. |
| **the light** | The device's power light. The guides say what it does at each step. |
| **the small button on the back** | The one button the guides use: a short press restarts the device, holding it for five seconds erases it. |

## Which release these guides are for

Enkrnett 0.3.0 on OpenWrt 25.12. The devices named here are the two the project builds for.
**Neither has been installed by a non-expert yet**: the usability test of chapter 5 (M-7) is
still to be run, and the install guides mark every step that waits for it. A guide is revised
after each test; the version of the guide is the version of Enkrnett.

## Other languages

These guides exist in English. The settings screen itself is in English and Japanese. The
Japanese guides are a recorded plan (`ROADMAP.md`, *Future plans*): they are written once the
English ones have been through the usability test, so that the translation follows text that
non-experts have read, and they are reviewed by a native reader before they are published.
