#!/bin/bash
# Builds a successor release for the update test on the VM (chapter 11 §3): the same source as a
# newer version, packaged with the pinned x86/64 SDK, and put into a complete x86-64 image the way
# a release is built (openwrt/build-image.sh: the pinned Image Builder, the locked packages of
# openwrt/lock/packages/x86_64_generic.json and no live feed, the carried patch applied, the
# from-image witness). The image carries the overlay the VM tests need (the ssh key, the test key,
# the test authority) so that the chain goes on after the update.
#
#   tests/vm/successor.sh VM_DIR VERSION OUT_DIR [broken]
#
#   VERSION  the Enkrnett version the daemon reports (ENKRNETT_VERSION at build time)
#   OUT_DIR  receives enkrnett-VERSION-25.12-x86_64_generic-sysupgrade.bin (the ext4 combined
#            image, gzip, as sysupgrade takes it), its build report and successor.log
#   broken   leaves out the wireless driver of the test VM (kmod-mac80211-hwsim): a release whose
#            health check cannot pass on this device (ENK-UPD-005's deliberately broken successor)
#
# Needs the VM directory of tests/vm/image.sh (id_ed25519.pub) and the files
# tests/vm/update-install.sh puts in VM_DIR/successor-files (the test key and authority), if any.
set -euo pipefail
. "$(dirname "$0")/../../openwrt/lib.sh"
VM=$(cd "${1:?VM_DIR}" && pwd)
VERSION=${2:?VERSION}
OUT=${3:?OUT_DIR}
BROKEN=${4:-}
mkdir -p "$OUT"; OUT=$(cd "$OUT" && pwd)
SDK_CACHE="${ENKRNETT_SDK_DIR_X86:-$CACHE/x86_64}"
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

echo "## the image as a release builds it (openwrt/build-image.sh, from the lock)"
# The overlay: the VM tests' first-boot script and modules, the ssh key, whatever the update test
# adds (the test key, the authority). The carried patch is applied by the build, so the first-boot
# script finds it in place; the from-image witness is the build's too.
FILES=$(mktemp -d); trap 'rm -rf "$FILES"' EXIT
cp -r "$ROOT/tests/vm/files/." "$FILES/"
mkdir -p "$FILES/etc/dropbear"
cp "$VM/id_ed25519.pub" "$FILES/etc/dropbear/authorized_keys"
chmod 0700 "$FILES/etc/dropbear"; chmod 0600 "$FILES/etc/dropbear/authorized_keys"
[ -d "$VM/successor-files" ] && cp -r "$VM/successor-files/." "$FILES/"
DROP=""; [ "$BROKEN" = broken ] && DROP="kmod-mac80211-hwsim"
"$ROOT/openwrt/build-image.sh" x86_64_generic --apk "$APK" --out "$OUT" --files "$FILES" --version "$VERSION" ${DROP:+--drop "$DROP"} \
  | sed 's/^/  /'
NAME="enkrnett-$VERSION-25.12-x86_64_generic-sysupgrade.bin"
[ -f "$OUT/$NAME" ] || { echo "no image built"; exit 1; }
echo "image: $OUT/$NAME ($(stat -c %s "$OUT/$NAME") bytes)${BROKEN:+ [$BROKEN: without kmod-mac80211-hwsim]}"
