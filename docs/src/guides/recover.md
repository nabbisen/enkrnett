# Recover

What to do when something went wrong. Find your situation; each has its own few steps. None of
them needs a computer, except where the router's original update screen is involved.

## I cannot find the setup Wi-Fi after installing

The setup Wi-Fi `Enkrnett Setup` plus four characters stays open for 30 minutes after the device
starts. If it has closed, or you never saw it:

1. Unplug the device's power, wait ten seconds, plug it in again.
2. Wait about two minutes until the light is steady.
3. Look for the setup Wi-Fi again on your phone. It has no password.

If after two tries there is no setup Wi-Fi and the device's original Wi-Fi is not back either,
go to *An installation that was interrupted* below.

## I was set up and lost the device during step 7

Step 7 of Set up asks you to join the management Wi-Fi within ten minutes. If you did not manage
to (the password was not kept, the phone did not join), wait: ten minutes after step 6 the device
forgets everything and offers the setup Wi-Fi again by itself. Set up again from the start.

## I lost the owner password

There is no way to read or recover it: the device keeps only a scrambled form that cannot be
turned back. The way back is Erase, then Set up again:

1. [Erase](./erase.md) with the button: hold the small button on the back for five seconds, until
   the light changes.
2. [Set up](./set-up.md) again. Your guests will have to join the guest Wi-Fi again if you give it
   a new name; with the same name they reconnect by themselves.

## I lost the management Wi-Fi password

If you can still log in from a phone that is on the management Wi-Fi: open the menu,
*Management Wi-Fi*, and *Show the password*. There is also a code to scan with another phone.

If no phone is on the management Wi-Fi any more: the owner password alone does not help, because
the settings screen is reachable only through the management Wi-Fi. Erase with the button and set
up again (above).

## I cannot open the settings screen

1. Check that the phone is joined to the management Wi-Fi, not to the guest Wi-Fi or to your own
   network. The settings screen is reachable from the management Wi-Fi only.
2. Open the address you kept at setup (`http://192.168.207.1/` unless the device showed another).
   Type it in the address bar; a search box will not find it.
3. If the page does not load, restart the device: a short press of the small button on the back,
   or unplug and plug the power. Wait a minute, join the management Wi-Fi, try again.

## The guest Wi-Fi is there but guests cannot use it

1. On *Status*, check the Internet connection. *Not connected* means the cable between the device's
   Internet port and your own router, or your own router, is the problem; the guest Wi-Fi itself
   is fine.
2. *Status* may say *Settings were changed outside Enkrnett. Restore my settings?* Press *Restore
   my settings*; the device puts back what you set.
3. Some older phones cannot join a guest Wi-Fi set to *Encrypted only*. In *Settings*, *Who can
   join*, choose *Open to every device*; the screen then tells you what that means for those phones.

## An update went wrong

Every message the update can show, and what to do, is in [Update](./update.md). In short: when
the device says nothing was changed, it is running as before; when it says it is using your
previous settings, it is running the new Enkrnett with your Wi-Fi as before, and the project needs
to hear about it.

## An installation that was interrupted

This is the installation from the router's original firmware ([Install](./index.md)), when the
power was lost or the browser was closed while the router's light was blinking.

1. Wait three minutes with the power connected. Routers finish writing even when the page is gone.
2. Look for a Wi-Fi: either the router's original Wi-Fi (the name from the label) or
   `Enkrnett Setup …`. If one of them is there, the router is fine: join it and continue either
   with the installation from step 2 or with [Set up](./set-up.md).
3. If neither appears after two more minutes, unplug the power, wait ten seconds, plug it in
   again, and wait two minutes. Then look again.
4. If still neither appears, the router's own recovery procedure is needed: the router's manual
   describes it under a heading about firmware recovery or an emergency update, and it usually
   involves holding a button while connecting the power and then offering the original firmware
   from a computer. *To be confirmed on a unit:* the exact procedure for each supported router;
   this section is completed from the usability test (chapter 5, M-7), which tries an interrupted
   installation on purpose.

A router whose screen **refused** the file was never changed; it still runs its original
firmware, and nothing needs recovering.

## Nothing above fits

The release notes (`docs/src/releases/`) say where to report a problem. Say which device, what you
did, and what the screen said, word for word.
