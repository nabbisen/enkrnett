# Update

About five minutes. The device fetches the newer Enkrnett, checks that it is the project's and
made for this device, writes it, and restarts with your settings kept. **The Wi-Fi stops for about
two minutes**; choose a quiet moment.

## Before you start

- The device has an Internet connection (*Status* says *connected*).
- Your phone is on the management Wi-Fi and you are logged in.
- You have the owner password; the device does not ask for it again, but you need it to log in
  afterwards.

## The steps

1. Open the menu and *Details*. Under the line *Enkrnett 0.3.0 on OpenWrt 25.12* the device says
   either *Up to date* or *Update available: Enkrnett x.y.z*. The device looks once a day by
   itself; press *Check for updates* to look now.
2. Press *Update*. The screen states what will happen: the version, how many minutes it takes,
   that the Wi-Fi stops meanwhile, and that it cannot be undone from here. When the newer
   Enkrnett is built on a newer OpenWrt, the screen also says that this one takes longer and
   restarts the Wi-Fi twice.
3. Press *Update* again to confirm, or *Cancel*.
4. The screen says *Downloading…*, *Verifying…*, then *Installing. The Wi-Fi stops for about two
   minutes. Come back to this address afterwards.* The light blinks while the device writes.
5. When the light is steady again, join the management Wi-Fi (your phone may do it by itself),
   open the settings screen and log in. *Details* says *Updated to Enkrnett x.y.z*.

Everything you set up is still there: the guest Wi-Fi's name and mode, the management Wi-Fi and
its password, the owner password, the country. Phones that were on the guest Wi-Fi reconnect by
themselves.

## If it does not go as planned

The device checks the update before it writes anything. When a check fails, it says why in one
sentence, and **nothing was changed**:

| The screen says | What it means | What to do |
|---|---|---|
| Could not reach the update server. Check the Internet connection. | The device could not fetch the update information. | Check the cable and your Internet router; try later. |
| The update information could not be verified and was not used. | What the device received was not signed by the project. | Nothing to do on your side; the device keeps running as it is. Try again another day. |
| The update file could not be verified. Nothing was changed. | The file did not match what the project signed. | The same. |
| This update is not for this device. | The file was built for another router. | The same. |
| This update cannot follow the installed version directly. | A step in between is needed. | Install the update the device offers first; check again after it. |
| Not enough free memory. Restart the device and try again. | The device needs a moment of free memory to hold the file. | Menu, *Details*, *Restart*; then update again. |
| The update could not be downloaded. Check the Internet connection and try again. | The download broke off. | Try again. |
| The update could not be installed. Nothing was changed. | The device refused to write the file at the last step. | Try again; if it repeats, the release notes say whom to tell. |

One case is different. If the screen says **The update was installed, but the device could not
start your Wi-Fi with it and is using your previous settings**, the newer Enkrnett is on the
device but could not run your Wi-Fi the way you set it up. Your guest Wi-Fi and management Wi-Fi
keep working as before. Tell the project (the release notes say how) and wait for the next update;
nothing else is needed from you.

## What an update never does

- It never changes your settings or your passwords.
- It never starts by itself: only your *Update* starts it.
- It never installs anything the project did not sign, and never a file made for another router.
