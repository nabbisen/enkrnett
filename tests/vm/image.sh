#!/bin/bash
# Builds the x86-64 OpenWrt test VM from the pinned Image Builder (openwrt/lock/inputs.json) and
# writes a VM directory the other scripts take as VM_DIR: disk.img, the launcher (vm.sh, vmssh,
# vmscp, vmsh.py) and a fresh ssh key. Chapter 14 §2: the VM is the test vehicle for chapters 2,
# 6 and 7; it is not a product target.
#
#   tests/vm/image.sh VM_DIR
#
# The image is the generic x86/64 ext4 image of the release plus what the tests need:
# wpad-basic-mbedtls (the product's hostapd), five mac80211_hwsim radios (the device and four
# stations), wpa_supplicant's wpa-cli, nmap and tcpdump-mini for the separation checks, patch for
# the carried wifi-scripts fix. The Image Builder fetches the release's packages over the network
# (its `repositories` file names the 25.12.5 feeds); the Image Builder itself is checked against
# its SHA-256 from the lock (ENK-BUILD-007). A release image is built differently, from the
# locked packages: openwrt/build-image.sh, which tests/vm/successor.sh uses.
set -euo pipefail
. "$(dirname "$0")/../../openwrt/lib.sh"
VM=${1:?VM_DIR}
for t in qemu-img ssh-keygen gunzip curl sha256sum zstd; do command -v $t >/dev/null || { echo "missing tool: $t"; exit 1; }; done

IB_DIR=$(image_builder x86/64)
echo "Image Builder: $IB_DIR"
# This is the test VM of the package path, not a release image: the live feeds and the test
# tooling. openwrt/build-image.sh may have left the Image Builder on the lock; put the feeds back.
[ -f "$IB_DIR/repositories.feeds" ] && cp "$IB_DIR/repositories.feeds" "$IB_DIR/repositories"
for f in "$IB_DIR"/packages/*.apk; do
  case "$(basename "$f")" in base-files-*|kernel-*|libc-*) ;; *) rm -f "$f" ;; esac
done
rm -f "$IB_DIR/packages/packages.adb"

mkdir -p "$VM"
VM=$(cd "$VM" && pwd)
[ -f "$VM/id_ed25519" ] || ssh-keygen -q -t ed25519 -N "" -C "enkrnett test vm" -f "$VM/id_ed25519"

# The overlay: tests/vm/files, the ssh key, the carried patch.
FILES=$(mktemp -d)
trap 'rm -rf "$FILES"' EXIT
cp -r "$ROOT/tests/vm/files/." "$FILES/"
mkdir -p "$FILES/etc/dropbear" "$FILES/usr/share/enkrnett-vm"
cp "$VM/id_ed25519.pub" "$FILES/etc/dropbear/authorized_keys"
cp "$ROOT"/openwrt/patches/0001-wifi-scripts-owe-transition-bss-mac.patch "$FILES/usr/share/enkrnett-vm/"
chmod 0700 "$FILES/etc/dropbear"; chmod 0600 "$FILES/etc/dropbear/authorized_keys"

PACKAGES="wpad-basic-mbedtls hostapd-utils wpa-cli iw ip-full kmod-mac80211-hwsim nmap tcpdump-mini patch shadow-su"
rm -rf "$IB_DIR/bin"
( cd "$IB_DIR" && make image PROFILE=generic PACKAGES="$PACKAGES" FILES="$FILES" >"$VM/image-build.log" 2>&1 ) \
  || { echo "image build failed (see $VM/image-build.log)"; tail -30 "$VM/image-build.log"; exit 1; }
IMG=$(find "$IB_DIR/bin" -name '*generic-ext4-combined.img.gz' | head -1)
[ -n "$IMG" ] || { echo "no ext4 combined image in $IB_DIR/bin"; exit 1; }
gunzip -c "$IMG" > "$VM/disk.img"
qemu-img resize -f raw "$VM/disk.img" 512M >/dev/null
cp "$ROOT"/tests/vm/launcher/{vm.sh,vmssh,vmscp,vmsh.py} "$VM/"
echo "VM directory: $VM ($(stat -c %s "$VM/disk.img") bytes disk; image $(basename "$IMG"))"
echo "start it with: $VM/vm.sh start"
