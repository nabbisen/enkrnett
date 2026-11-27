# Install Enkrnett on an ELECOM WRC-1167GST2

About 15 minutes. You put one file onto the router through the router's own update screen. The
router then becomes an Enkrnett device and offers its setup Wi-Fi; [Set up](./set-up.md) follows.

> **Not yet tested on a unit.** The project has not installed Enkrnett on this router from its
> original firmware yet, and no non-expert has tried this guide. The steps that depend on what the
> router's own screens show are marked *to be confirmed*; they are checked in the usability test
> (chapter 5, M-7) and this guide is corrected from it. Until then the images for this device are
> built and compared on every change but not published (chapter 5 §5.4 and the release notes say why).

## Before you start

- **Check the label on the bottom of the router before you change anything.** It must say the
  model **WRC-1167GST2** exactly. A different model, even one that looks the same (WRC-1167GST2A,
  WRC-1750GST2, WRC-2533GST2), needs its own file; this file would be refused, or worse.
- The label also shows a code of twelve letters and digits in pairs, such as `A1:B2:C3:D4:E5:F6`.
  **Write down its last four characters** (here `E5F6`). The device's setup Wi-Fi will end in them, so
  you can tell your device from a neighbour's. *To be confirmed:* which of the label's codes the
  device uses; the guide is corrected after the first unit.
- Versions of the router's original firmware this installation was tested from: **none yet.**
  The list fills in with the usability test.
- Have ready: the router and its power supply, the network cable, your phone or a computer, and
  the Enkrnett file (next step).

## 1. Get the file

From the release page of Enkrnett 0.3.0, download the one file for this router:

```text
enkrnett-0.3.0-25.12-elecom_wrc-1167gst2-factory.bin
```

Its name ends in `factory.bin`: that is the one the router's original firmware accepts. (The
other file for this router, ending in `sysupgrade.bin`, is for a device that already runs
Enkrnett; the device fetches it by itself when you press *Update*.) Do not rename it. The release
page also lists a long checksum beside the file; you do not need it.

## 2. Connect the router

1. Plug the network cable into the router's Internet port (the one set apart from the others,
   marked WAN or INTERNET on the router) and into a free port of your existing router.
2. Connect the power. Wait about a minute until the router's light is steady.
3. On your phone or computer, join the router's own Wi-Fi (its name and password are on the label)
   or connect a second cable to one of the numbered ports.
4. Open the router's settings page in the browser, at the address printed in the router's manual
   or on the label. *To be confirmed:* the address and the login the original firmware asks for.

## 3. Put the file onto the router

1. In the router's settings, find the page for a firmware update. *To be confirmed:* the exact
   menu names of the original firmware; expect a page about updating or upgrading the firmware
   from a file.
2. Choose the file from step 1 and start the update.
3. **Wait. Do not unplug the router and do not close the page for three minutes.** The router's
   light blinks while it writes the file and then the router restarts.

If the router's screen refuses the file, nothing was changed: the router still runs its original
firmware. Check the model on the label again, and that the file's name ends in
`elecom_wrc-1167gst2-factory.bin`. If the power was lost while the light was blinking, go to
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

*To be confirmed:* the colours of this router's light in each state; OpenWrt's description of the
device names a green power light for starting, running and updating.

## Recovery

Everything that can go wrong during or after an installation, and what to do, is in
[Recover](./recover.md). In short: a refused file changes nothing; an interrupted write is
retried through the router's own recovery screen; a device that was set up and lost is erased by
holding the small button on the back for five seconds.
