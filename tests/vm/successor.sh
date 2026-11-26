#!/bin/bash
# Builds a successor release for the update test on the VM (chapter 11 §3): the same source as a
# newer version, packaged with the pinned x86/64 SDK and put into a complete x86-64 image by the
# pinned Image Builder, as a release would be. The image carries the overlay the VM tests need
# (the ssh key, the test key, the test authority) so that the chain goes on after the update.
#
#   tests/vm/successor.sh VM_DIR VERSION OUT_DIR [broken]
#
#   VERSION  the Enkrnett version the daemon reports (ENKRNETT_VERSION at build time)
#   OUT_DIR  receives enkrnett-VERSION-25.12-x86_64_generic-sysupgrade.bin (the ext4 combined
#            image, gzip, as sysupgrade takes it) and successor.log
#   broken   leaves out the wireless driver of the test VM (kmod-mac80211-hwsim): a release whose
#            health check cannot pass on this device (ENK-UPD-005's deliberately broken successor)
#
# Needs the VM directory of tests/vm/image.sh (id_ed25519.pub) and the files
# tests/vm/update-install.sh puts in VM_DIR/successor-files (the test key and authority), if any.
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
VM=$(cd "${1:?VM_DIR}" && pwd)
VERSION=${2:?VERSION}
OUT=${3:?OUT_DIR}
BROKEN=${4:-}
mkdir -p "$OUT"; OUT=$(cd "$OUT" && pwd)
LOCK="$ROOT/openwrt/lock/inputs.json"
SDK_CACHE="${ENKRNETT_SDK_DIR_X86:-$ROOT/openwrt/.cache/x86_64}"
IB_CACHE="${ENKRNETT_IB_DIR_X86:-$ROOT/openwrt/.cache/x86_64-ib}"
lock() { python3 -c "import json,sys; d=json.load(open('$LOCK')); print(eval('d'+sys.argv[1]))" "$1"; }
LOG="$OUT/successor.log"; : > "$LOG"

echo "## the daemon as version $VERSION"
TARGET_DIR="$ROOT/target/successor"
( cd "$ROOT" && ENKRNETT_VERSION="$VERSION" RUSTFLAGS="-C target-feature=+crt-static" CARGO_TARGET_DIR="$TARGET_DIR" \
    cargo build -q --release -p enkrnettd --target x86_64-unknown-linux-musl ) >>"$LOG" 2>&1 \
  || { echo "daemon build failed (see $LOG)"; tail -20 "$LOG"; exit 1; }
DAEMON="$TARGET_DIR/x86_64-unknown-linux-musl/release/enkrnettd"
[ -d "$ROOT/ui/dist" ] || { echo "build the UI first (ui/: npm run build)"; exit 1; }

echo "## the package with the pinned SDK"
SDK_DIR=$(ls -d "$SDK_CACHE"/openwrt-sdk-* 2>/dev/null | head -1 || true)
[ -n "$SDK_DIR" ] || { echo "no SDK in $SDK_CACHE: run tests/vm/package.sh first"; exit 1; }
PRE="$OUT/prebuilt"
rm -rf "$PRE"; mkdir -p "$PRE/ui" "$PRE/profiles" "$PRE/keys"
cp "$DAEMON" "$PRE/enkrnettd"
cp -r "$ROOT/ui/dist/." "$PRE/ui/"
for d in "$ROOT"/openwrt/profiles/*/; do cp "$d/profile.json" "$PRE/profiles/$(basename "$d").json"; done
for k in "$ROOT"/openwrt/release-keys/*.pub; do [ -f "$k" ] && cp "$k" "$PRE/keys/"; done
rm -rf "$SDK_DIR/package/enkrnett"
cp -r "$ROOT/openwrt/package/enkrnett" "$SDK_DIR/package/enkrnett"
sed -i "s/^PKG_VERSION:=.*/PKG_VERSION:=$VERSION/" "$SDK_DIR/package/enkrnett/Makefile"
( cd "$SDK_DIR" && { [ -f .config ] || make defconfig >/dev/null; } && make package/enkrnett/compile ENKRNETT_PREBUILT="$PRE" ) >>"$LOG" 2>&1 \
  || { echo "package build failed (see $LOG)"; tail -20 "$LOG"; exit 1; }
APK=$(find "$SDK_DIR/bin/packages" -name "enkrnett-$VERSION*.apk" | head -1)
[ -n "$APK" ] || { echo "no package built"; exit 1; }
echo "package: $(basename "$APK") ($(stat -c %s "$APK") bytes)"

echo "## the image with the pinned Image Builder"
IB_DIR=$(ls -d "$IB_CACHE"/openwrt-imagebuilder-*/ 2>/dev/null | head -1 || true)
[ -n "$IB_DIR" ] || { echo "no Image Builder in $IB_CACHE: run tests/vm/image.sh first"; exit 1; }
IB_DIR=${IB_DIR%/}
IB_URL=$(lock "['openwrt']['targets']['x86/64']['image_builder']['url']"); echo "Image Builder: $(basename "$IB_URL")"
# The package goes into the Image Builder's local package directory, which it indexes itself.
rm -f "$IB_DIR"/packages/enkrnett-*.apk "$IB_DIR/packages/packages.adb"
cp "$APK" "$IB_DIR/packages/"
FILES=$(mktemp -d); trap 'rm -rf "$FILES"' EXIT
cp -r "$ROOT/tests/vm/files/." "$FILES/"
mkdir -p "$FILES/etc/dropbear" "$FILES/usr/share/enkrnett-vm"
cp "$VM/id_ed25519.pub" "$FILES/etc/dropbear/authorized_keys"
cp "$ROOT"/openwrt/patches/0001-wifi-scripts-owe-transition-bss-mac.patch "$FILES/usr/share/enkrnett-vm/"
chmod 0700 "$FILES/etc/dropbear"; chmod 0600 "$FILES/etc/dropbear/authorized_keys"
[ -d "$VM/successor-files" ] && cp -r "$VM/successor-files/." "$FILES/"
# What the project's image build places and the package never does: the witness of an image
# install on a target without /rom (ENK-INST-005; crates/enkrnettd/src/platform/openwrt.rs).
mkdir -p "$FILES/usr/share/enkrnett"
echo "placed by the project's image build; never by the package" > "$FILES/usr/share/enkrnett/from-image"
PACKAGES="enkrnett wpad-basic-mbedtls hostapd-utils wpa-cli iw ip-full nmap tcpdump-mini patch shadow-su"
[ "$BROKEN" = broken ] || PACKAGES="$PACKAGES kmod-mac80211-hwsim"
rm -rf "$IB_DIR/bin"
( cd "$IB_DIR" && make image PROFILE=generic PACKAGES="$PACKAGES" FILES="$FILES" ) >>"$LOG" 2>&1 \
  || { echo "image build failed (see $LOG)"; tail -30 "$LOG"; exit 1; }
IMG=$(find "$IB_DIR/bin" -name '*generic-ext4-combined.img.gz' | head -1)
[ -n "$IMG" ] || { echo "no ext4 combined image"; exit 1; }
NAME="enkrnett-$VERSION-25.12-x86_64_generic-sysupgrade.bin"
cp "$IMG" "$OUT/$NAME"
echo "image: $OUT/$NAME ($(stat -c %s "$OUT/$NAME") bytes)${BROKEN:+ [$BROKEN: without kmod-mac80211-hwsim]}"
