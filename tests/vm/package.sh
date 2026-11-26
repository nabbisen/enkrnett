#!/bin/bash
# Builds the Enkrnett package for the x86-64 test VM with the pinned x86/64 SDK, installs it on
# the VM with apk, and reboots the VM so that ubusd reads the ACL and procd starts the service.
# This is the package path of a device (USERID, conffiles, init script, capabilities), on the
# test vehicle.
#
#   tests/vm/package.sh VM_DIR [DAEMON]   DAEMON: a static x86_64-unknown-linux-musl enkrnettd
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
VM=${1:?VM_DIR}
DAEMON=${2:-$ROOT/target/x86_64-unknown-linux-musl/release/enkrnettd}
LOCK="$ROOT/openwrt/lock/inputs.json"
CACHE="${ENKRNETT_SDK_DIR_X86:-$ROOT/openwrt/.cache/x86_64}"
OUT="$ROOT/openwrt/out/x86_64"
lock() { python3 -c "import json,sys; d=json.load(open('$LOCK')); print(eval('d'+sys.argv[1]))" "$1"; }

[ -f "$DAEMON" ] || { echo "build the daemon first: RUSTFLAGS='-C target-feature=+crt-static' cargo build --release -p enkrnettd --target x86_64-unknown-linux-musl"; exit 1; }
[ -d "$ROOT/ui/dist" ] || { echo "build the UI first (ui/: npm run build)"; exit 1; }

SDK_URL=$(lock "['openwrt']['targets']['x86/64']['sdk']['url']")
SDK_SHA=$(lock "['openwrt']['targets']['x86/64']['sdk']['sha256']")
mkdir -p "$CACHE" "$OUT"
SDK_DIR=$(ls -d "$CACHE"/openwrt-sdk-* 2>/dev/null | head -1 || true)
if [ -z "$SDK_DIR" ]; then
  TAR="$CACHE/$(basename "$SDK_URL")"
  [ -f "$TAR" ] || { echo "fetching SDK: $SDK_URL"; curl -fsSL -o "$TAR" "$SDK_URL"; }
  echo "$SDK_SHA  $TAR" | sha256sum -c - || { echo "SDK checksum mismatch (ENK-BUILD-007)"; exit 1; }
  tar -C "$CACHE" --zstd -xf "$TAR"
  SDK_DIR=$(ls -d "$CACHE"/openwrt-sdk-* | head -1)
fi
echo "SDK: $SDK_DIR"

PRE="$OUT/prebuilt"
rm -rf "$PRE"; mkdir -p "$PRE/ui" "$PRE/profiles"
cp "$DAEMON" "$PRE/enkrnettd"
cp -r "$ROOT/ui/dist/." "$PRE/ui/"
for d in "$ROOT"/openwrt/profiles/*/; do cp "$d/profile.json" "$PRE/profiles/$(basename "$d").json"; done
rm -rf "$SDK_DIR/package/enkrnett"
cp -r "$ROOT/openwrt/package/enkrnett" "$SDK_DIR/package/enkrnett"
( cd "$SDK_DIR" && { [ -f .config ] || make defconfig >/dev/null; } && make package/enkrnett/compile ENKRNETT_PREBUILT="$PRE" >"$OUT/package.log" 2>&1 ) \
  || { echo "package build failed (see $OUT/package.log)"; tail -20 "$OUT/package.log"; exit 1; }
APK=$(find "$SDK_DIR/bin/packages" -name 'enkrnett-*.apk' | head -1)
cp "$APK" "$OUT/"
echo "package: $OUT/$(basename "$APK") ($(stat -c %s "$APK") bytes)"

cd "$VM"
./vm.sh start >/dev/null || true
python3 vmsh.py wait 120 >/dev/null 2>&1 || true
./vmssh 'killall enkrnettd 2>/dev/null; /etc/init.d/enkrnett stop 2>/dev/null; apk del enkrnett 2>/dev/null; rm -rf /etc/enkrnett /usr/share/enkrnett; true' >/dev/null 2>&1 || true
./vmscp "$OUT/$(basename "$APK")" /tmp/enkrnett.apk >/dev/null
./vmssh 'set -e
apk add --allow-untrusted /tmp/enkrnett.apk
echo "--- installed files"; apk info -L enkrnett | head -20
echo "--- user"; id enkrnett
echo "--- ownership the first-boot script will set"; ls -ld /etc/enkrnett
# Ownership of the four files is the first-boot script's and the init script's job; nothing by hand.
/etc/init.d/uhttpd disable 2>/dev/null; /etc/init.d/uhttpd stop 2>/dev/null; true
echo installed; (sleep 1; reboot) >/dev/null 2>&1 &'
echo "package installed; VM rebooting"
