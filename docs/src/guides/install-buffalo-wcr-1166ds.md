# Install Enkrnett on a Buffalo WCR-1166DS

About 15 minutes. You put one file onto the router through the router's own update screen. The
router then becomes an Enkrnett device and offers its setup Wi-Fi; [Set up](./set-up.md) follows.

> **Not yet tested on a unit.** The project has not installed Enkrnett on this router from its
> original firmware yet, and no non-expert has tried this guide. The steps that depend on what the
> router's own screens show are marked *to be confirmed*; they are checked in the usability test
> (chapter 5, M-7) and this guide is corrected from it. This router is of the small class (8 MB of
> storage, 64 MB of memory): the project builds and measures for it from the first day and
> promotes it to the supported list when it passes (chapter 5). Until then its images are built and
> compared on every change but not published.

## Before you start

- **Check the label on the router before you change anything.** It must say the model
  **WCR-1166DS** exactly. A different Buffalo model needs its own file; this file would be
  refused, or worse.
- The label also shows a code of twelve letters and digits in pairs, such as `A1:B2:C3:D4:E5:F6`.
  **Write down its last four characters** (here `E5F6`). The device's setup Wi-Fi will end in them.
  *To be confirmed:* which of the label's codes the device uses.
- Versions of the router's original firmware this installation was tested from: **none yet.**
  The list fills in with the usability test.
- Have ready: the router and its power supply, the network cable, your phone or a computer, and
  the Enkrnett file (next step).

## 1. Get the file

From the release page of Enkrnett 0.3.0, download the one file for this router:

```text
enkrnett-0.3.0-25.12-buffalo_wcr-1166ds-factory.bin
```

Its name ends in `factory.bin`: that is the one the router's original firmware accepts. Do not
rename it. The release page also lists a long checksum beside the file; you do not need it.

## 2. Connect the router

1. If the router has a switch marked ROUTER / AP / WB (or AUTO), set it to ROUTER. *To be
   confirmed:* the switch's positions on this model; OpenWrt's description of the device lists an
   AP switch.
2. Plug the network cable into the router's Internet port (marked INTERNET or WAN) and into a free
   port of your existing router.
3. Connect the power. Wait about a minute until the router's lights are steady.
4. On your phone or computer, join the router's own Wi-Fi (its name and password are on the label
   or the card in the box) or connect a second cable to the LAN port.
5. Open the router's settings page in the browser, at the address printed in the router's manual.
   *To be confirmed:* the address and the login the original firmware asks for.

## 3. Put the file onto the router

1. In the router's settings, find the page for a firmware update. *To be confirmed:* the exact
   menu names of the original firmware; expect a page about updating the firmware from a file.
2. Choose the file from step 1 and start the update.
3. **Wait. Do not unplug the router and do not close the page for three minutes.** The router's
   light blinks while it writes the file and then the router restarts.

If the router's screen refuses the file, nothing was changed: the router still runs its original
firmware. Check the model on the label again, and that the file's name ends in
`buffalo_wcr-1166ds-factory.bin`. If the power was lost while the light was blinking, go to
[Recover](./recover.md), *An installation that was interrupted*.

## 4. Find the setup Wi-Fi

Within about two minutes the light becomes steady and your phone sees a new Wi-Fi named

```text
Enkrnett Setup E5F6
```

with the last four characters from the label in place of `E5F6`. The router's original Wi-Fi is
gone. Join the setup Wi-Fi with your phone; it has no password. Then open the browser and go to
`http://192.168.207.1/`. The screen *Set up this device* appears: continue with
[Set up](./set-up.md).

The setup Wi-Fi stays open for 30 minutes. If it has closed, unplug the power and plug it in
again: a device that has not been set up opens it at every start.

## What the light does

| The light | Meaning |
|---|---|
| Blinking, during step 3 | Writing the file. Do not unplug. |
| Off, then blinking slowly | Starting. |
| Steady | Running. |
| Blinking when you press *Blink* in Set up | The device you are looking at is the one you are setting up. |

This router has several lights (ROUTER, INTERNET, WIRELESS, DIAG, POWER). Enkrnett uses the
**POWER** light. *To be confirmed:* the colours in each state; OpenWrt's description of the device
names a green power light for starting, running and updating.

## Recovery

Everything that can go wrong during or after an installation, and what to do, is in
[Recover](./recover.md).
