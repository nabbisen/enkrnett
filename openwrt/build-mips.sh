#!/bin/bash
# Builds enkrnettd for OpenWrt (mipsel_24kc, dynamic musl) with the pinned SDK toolchain and the
# pinned Rust nightly, then measures the squashfs sizes against the budgets of RFC 010
# (ENK-FOOT-001 to 003; ENK-BUILD-011).
#
#   openwrt/build-mips.sh            build, measure, fail when over budget
#   openwrt/build-mips.sh --no-check build and measure only
#   openwrt/build-mips.sh --package  as the first form, then build the OpenWrt package (.apk)
#                                    inside the SDK from the prebuilt daemon and ui/dist
#
# Environment:
#   ENKRNETT_SDK_DIR  directory that holds (or will hold) the extracted SDK; default openwrt/.cache
#   RUST_NIGHTLY      overrides the pinned nightly (for local experiments only)
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
LOCK="$ROOT/openwrt/lock/inputs.json"
CACHE="${ENKRNETT_SDK_DIR:-$ROOT/openwrt/.cache}"
OUT="$ROOT/openwrt/out"
CHECK=1
PACKAGE=0
case "${1:-}" in
  --no-check) CHECK=0 ;;
  --package) PACKAGE=1 ;;
  "") ;;
  *) echo "usage: $0 [--no-check|--package]"; exit 2 ;;
esac

lock() { python3 -c "import json,sys; d=json.load(open('$LOCK')); print(eval('d'+sys.argv[1]))" "$1"; }

TARGET_KEY=$(lock "['openwrt']['daemon_build_target']")
SDK_URL=$(lock "['openwrt']['targets']['$TARGET_KEY']['sdk']['url']")
SDK_SHA=$(lock "['openwrt']['targets']['$TARGET_KEY']['sdk']['sha256']")
TC_GLOB=$(lock "['openwrt']['toolchain_dir_glob']")
NIGHTLY="${RUST_NIGHTLY:-$(lock "['rust']['nightly']")}"
RUST_TARGET=$(lock "['rust']['target']")
BUILD_STD=$(lock "['rust']['build_std']")
BUILD_STD_FEATURES=$(lock "['rust']['build_std_features']")
RUSTFLAGS_LOCK=$(lock "['rust']['rustflags']")
B_DAEMON=$(lock "['budgets_kib']['daemon']")
B_UI=$(lock "['budgets_kib']['ui']")
B_EXTRAS=$(lock "['budgets_kib']['extras']")

mkdir -p "$CACHE" "$OUT"
SDK_TAR="$CACHE/$(basename "$SDK_URL")"
SDK_DIR=$(ls -d "$CACHE"/openwrt-sdk-* 2>/dev/null | head -1 || true)
if [ -z "$SDK_DIR" ]; then
  if [ ! -f "$SDK_TAR" ]; then
    echo "fetching SDK: $SDK_URL"
    curl -fsSL -o "$SDK_TAR" "$SDK_URL"
  fi
  echo "$SDK_SHA  $SDK_TAR" | sha256sum -c - || { echo "SDK checksum mismatch (ENK-BUILD-007)"; exit 1; }
  tar -C "$CACHE" --zstd -xf "$SDK_TAR"
  SDK_DIR=$(ls -d "$CACHE"/openwrt-sdk-* | head -1)
fi
echo "SDK: $SDK_DIR"

export STAGING_DIR="$SDK_DIR/staging_dir"
TC=$(ls -d "$STAGING_DIR"/$TC_GLOB | head -1)/bin
export CARGO_TARGET_MIPSEL_UNKNOWN_LINUX_MUSL_LINKER="$TC/mipsel-openwrt-linux-musl-gcc"
export CC_mipsel_unknown_linux_musl="$TC/mipsel-openwrt-linux-musl-gcc"
export AR_mipsel_unknown_linux_musl="$TC/mipsel-openwrt-linux-musl-ar"

echo "toolchain: $NIGHTLY, $RUST_TARGET, -Zbuild-std=$BUILD_STD, features $BUILD_STD_FEATURES, RUSTFLAGS $RUSTFLAGS_LOCK"
RUSTFLAGS="$RUSTFLAGS_LOCK" cargo "+$NIGHTLY" build --release -p enkrnettd \
  --target "$RUST_TARGET" -Zbuild-std="$BUILD_STD" -Zbuild-std-features="$BUILD_STD_FEATURES" \
  --target-dir "$ROOT/target/mips"
BIN="$ROOT/target/mips/$RUST_TARGET/release/enkrnettd"
cp "$BIN" "$OUT/enkrnettd"
file "$OUT/enkrnettd" | sed 's/^/binary: /'

# ----- sizes ---------------------------------------------------------------------------------
readarray -t MKSQ_ARGS < <(python3 -c "import json; print('\n'.join(json.load(open('$LOCK'))['squashfs']['mksquashfs_args']))")

# The budget figure is the size of a squashfs holding exactly the measured tree. Without
# squashfs-tools the script estimates it from xz with OpenWrt's squashfs settings plus 1 KiB of
# metadata per file (M-2 found the two within about 1 KiB); CI has the real tool.
if command -v mksquashfs >/dev/null; then
  METHOD=squashfs
else
  METHOD=xz-estimate
  echo "note: mksquashfs not found; sizes are xz estimates (install squashfs-tools for the real figure)"
fi

squashfs_kib() { # dir -> KiB of a squashfs holding exactly that tree
  local bytes
  if [ "$METHOD" = squashfs ]; then
    local img; img=$(mktemp)
    mksquashfs "$1" "$img" "${MKSQ_ARGS[@]}" -quiet >/dev/null 2>&1
    bytes=$(stat -c %s "$img"); rm -f "$img"
  else
    bytes=0
    while IFS= read -r -d '' f; do
      local c; c=$(xz --format=raw --lzma2=preset=9e,lc=0,lp=2,pb=2,dict=256KiB -c "$f" | wc -c)
      bytes=$(( bytes + c + 1024 ))
    done < <(find "$1" -type f -print0)
  fi
  echo $(( (bytes + 1023) / 1024 ))
}

T=$(mktemp -d)
mkdir -p "$T/daemon/usr/sbin" "$T/ui" "$T/extras"
cp "$OUT/enkrnettd" "$T/daemon/usr/sbin/"
DAEMON_KIB=$(squashfs_kib "$T/daemon")
UI_KIB="-"
if [ -d "$ROOT/ui/dist" ]; then
  cp -r "$ROOT/ui/dist/." "$T/ui/"
  UI_KIB=$(squashfs_kib "$T/ui")
fi
cp -r "$ROOT/openwrt/package/enkrnett/files/." "$T/extras/"
EXTRAS_KIB=$(squashfs_kib "$T/extras")
rm -rf "$T"

RAW=$(stat -c %s "$OUT/enkrnettd")
echo "method: $METHOD"
printf '%-28s %10s %10s\n' item kib budget_kib
printf '%-28s %10s %10s\n' "daemon (squashfs)" "$DAEMON_KIB" "$B_DAEMON"
printf '%-28s %10s %10s\n' "ui/dist (squashfs)" "$UI_KIB" "$B_UI"
printf '%-28s %10s %10s\n' "extras (squashfs)" "$EXTRAS_KIB" "$B_EXTRAS"
printf '%-28s %10s\n' "daemon raw bytes" "$RAW"
python3 - "$OUT/sizes.json" "$DAEMON_KIB" "$UI_KIB" "$EXTRAS_KIB" "$RAW" "$NIGHTLY" "$METHOD" <<'EOF'
import json, sys
out, d, u, e, raw, nightly, method = sys.argv[1:]
json.dump({"method": method, "daemon_squashfs_kib": int(d), "ui_squashfs_kib": None if u == "-" else int(u),
           "extras_squashfs_kib": int(e), "daemon_raw_bytes": int(raw), "rust": nightly}, open(out, "w"), indent=1)
EOF
echo "sizes written to $OUT/sizes.json"

if [ "$CHECK" = 1 ]; then
  fail=0
  [ "$DAEMON_KIB" -le "$B_DAEMON" ] || { echo "ENK-FOOT-001: daemon $DAEMON_KIB KiB > $B_DAEMON KiB"; fail=1; }
  if [ "$UI_KIB" != "-" ]; then [ "$UI_KIB" -le "$B_UI" ] || { echo "ENK-FOOT-002: ui $UI_KIB KiB > $B_UI KiB"; fail=1; }; fi
  [ "$EXTRAS_KIB" -le "$B_EXTRAS" ] || { echo "ENK-FOOT-003: extras $EXTRAS_KIB KiB > $B_EXTRAS KiB"; fail=1; }
  [ "$fail" = 0 ] && echo "budgets met" || exit 1
fi

# ----- OpenWrt package (optional) --------------------------------------------------------------
if [ "$PACKAGE" = 1 ]; then
  [ -d "$ROOT/ui/dist" ] || { echo "ui/dist is needed for the package (build the UI first)"; exit 1; }
  PRE="$OUT/prebuilt"
  rm -rf "$PRE"; mkdir -p "$PRE/ui"
  cp "$OUT/enkrnettd" "$PRE/"
  cp -r "$ROOT/ui/dist/." "$PRE/ui/"
  mkdir -p "$PRE/profiles"
  for d in "$ROOT"/openwrt/profiles/*/; do cp "$d/profile.json" "$PRE/profiles/$(basename "$d").json"; done
  rm -rf "$SDK_DIR/package/enkrnett"
  cp -r "$ROOT/openwrt/package/enkrnett" "$SDK_DIR/package/enkrnett"
  # STAGING_DIR is exported above for the toolchain wrapper; the SDK's make derives its host
  # directory from it and must see its own value instead.
  ( cd "$SDK_DIR" && env -u STAGING_DIR sh -c '{ [ -f .config ] || make defconfig >/dev/null; } && make package/enkrnett/compile ENKRNETT_PREBUILT="$0"' "$PRE" >"$OUT/package.log" 2>&1 ) \
    || { echo "package build failed (see $OUT/package.log)"; tail -20 "$OUT/package.log"; exit 1; }
  APK=$(find "$SDK_DIR/bin/packages" -name 'enkrnett-*.apk' | head -1)
  cp "$APK" "$OUT/"
  echo "package: $OUT/$(basename "$APK") ($(stat -c %s "$APK") bytes)"
fi
