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
# its SHA-256 from the lock (ENK-BUILD-007).
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
VM=${1:?VM_DIR}
LOCK="$ROOT/openwrt/lock/inputs.json"
CACHE="${ENKRNETT_IB_DIR_X86:-$ROOT/openwrt/.cache/x86_64-ib}"
lock() { python3 -c "import json,sys; d=json.load(open('$LOCK')); print(eval('d'+sys.argv[1]))" "$1"; }
for t in qemu-img ssh-keygen gunzip curl sha256sum zstd; do command -v $t >/dev/null || { echo "missing tool: $t"; exit 1; }; done

IB_URL=$(lock "['openwrt']['targets']['x86/64']['image_builder']['url']")
IB_SHA=$(lock "['openwrt']['targets']['x86/64']['image_builder']['sha256']")
mkdir -p "$CACHE"
IB_DIR=$(ls -d "$CACHE"/openwrt-imagebuilder-*/ 2>/dev/null | head -1 || true)
if [ -z "$IB_DIR" ]; then
  TAR="$CACHE/$(basename "$IB_URL")"
  [ -f "$TAR" ] || { echo "fetching Image Builder: $IB_URL"; curl -fsSL -o "$TAR" "$IB_URL"; }
  echo "$IB_SHA  $TAR" | sha256sum -c - || { echo "Image Builder checksum mismatch (ENK-BUILD-007)"; exit 1; }
  tar -C "$CACHE" --zstd -xf "$TAR"
  IB_DIR=$(ls -d "$CACHE"/openwrt-imagebuilder-*/ | head -1)
fi
IB_DIR=${IB_DIR%/}
echo "Image Builder: $IB_DIR"

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
