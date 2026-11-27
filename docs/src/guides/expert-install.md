# Expert install: Enkrnett as a package on OpenWrt

**For experts.** This page is exempt from the word rules of chapter 1 §4; it names OpenWrt's own
things by their names. It is the expert install class of chapter 5 §5.2 (ENK-INST-005): Enkrnett
installed as a package on a device that already runs the OpenWrt baseline, 25.12. Nothing here is
needed for a device installed from an Enkrnett image.

## What you get, and what you do not

| | Image install | Package install |
|---|---|---|
| OpenWrt | The release's exact package set (the lock) | Whatever the device runs |
| Updates | The device checks and installs signed images | Off until the owner turns checks on; then the same mechanism (`sysupgrade` with the configuration kept) |
| Hardening (chapter 2 §6) | Complete on `standard` devices | The device reports which of ENK-HARD-001 to 005 and 008 are not met, in *Details* |
| Support | Tier 1 devices, when promoted | None; it is an expert install |

## Requirements

- OpenWrt **25.12** (the baseline of this release; another series is not supported).
- A device for which Enkrnett ships a profile: `elecom_wrc-1167gst2`, `buffalo_wcr-1166ds`, or the
  x86-64 test vehicle. The daemon selects the profile by the board name (`ubus call system board`)
  and does not start on a board without one.
- `wpad-basic-mbedtls` as the hostapd (the default of 25.12 images); the package depends on it,
  `dnsmasq`, `firewall4` and `usign`.
- Port 80 free on the management address: Enkrnett redirects it to the daemon. Disable `uhttpd`
  (LuCI) first, or the settings screen is unreachable.
- Root access over SSH, and the package file for the device's architecture from the release page:
  `enkrnett-0.3.0-r1-mipsel_24kc.apk` for both ramips devices. The release's `SHA256SUMS` covers it.

## Steps

```sh
scp enkrnett-0.3.0-r1-mipsel_24kc.apk root@192.168.1.1:/tmp/
ssh root@192.168.1.1
/etc/init.d/uhttpd disable; /etc/init.d/uhttpd stop     # port 80 is Enkrnett's
apk add --allow-untrusted /tmp/enkrnett-0.3.0-r1-mipsel_24kc.apk
reboot
```

`--allow-untrusted` is needed because the package is not signed with an OpenWrt feed key; verify
its SHA-256 against the release's `SHA256SUMS` before you install it. The reboot lets `ubusd` read
the package's ACL and `procd` start the service as the user `enkrnett`; the first-boot script
gives the daemon its state directory, `/etc/enkrnett`.

After the reboot the device behaves as an unclaimed Enkrnett device: it offers the setup Wi-Fi
`Enkrnett Setup XXXX` for 30 minutes and [Set up](./set-up.md) applies. Your previous wireless
configuration is replaced by Enkrnett's networks; the uplink is the profile's WAN port by DHCP.

## What the device reports

- *Details* says **Installed as a package**.
- *Details*, *Protections of this device*, lists the hardening requirements and whether each is
  met on this device (ENK-HARD-006): a package install on a device whose OpenWrt build lacks
  `ujail` or PIE reports them as not met.
- Update checks are **off**: a package install was not made from a published image, so the device
  does not look for one until the owner turns the switch on in *Details* (ENK-UPD-008). With checks
  on, an offered image is installed through `sysupgrade` like any update, keeping `/etc/config`
  and `/etc/enkrnett`; the device is then an image install.

## Removing it

`apk del enkrnett` removes the daemon, the UI and the scripts; `rm -rf /etc/enkrnett` removes the
owner and the settings; the owned sections in `/etc/config/wireless`, `network`, `dhcp` and
`firewall` carry the Enkrnett marker and can be removed by hand. Erase (the button, five seconds)
on a package install removes Enkrnett's state and sections and leaves the rest of the device as it
is, because `firstboot` would remove Enkrnett itself.

## How this is tested

`tests/vm/package.sh` installs the package on the x86-64 OpenWrt VM through `apk` and `procd`;
`tests/vm/contract.sh` and the rest of the chain run against it (`docs/src/verification/m1.md`,
`m2.md`). The hardening report is `crates/enkrnettd/tests/api_contract.rs` and the VM contract.
