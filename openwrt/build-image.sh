#!/bin/bash
# Builds the firmware image(s) of a device profile from the lock (chapter 11 §3; ENK-BUILD-007,
# 008, 010; ENK-RLS-002): the pinned Image Builder, the locked packages and no live feed, the
# enkrnett package given, the carried patches applied to the locked files, the from-image witness.
#
#   openwrt/build-image.sh PROFILE --apk ENKRNETT.apk [options]
#
#   --out DIR        where the images go (default openwrt/out/images)
#   --files DIR      an overlay copied into the image's root (the VM tests' ssh key and such)
#   --add "PKGS"     packages beyond the profile's (they must be in the lock, or use --live)
#   --drop "PKGS"    packages of the profile's set left out (the deliberately broken successor)
#   --live           the live feeds instead of the lock; never for a release
#   --twice          build a second time and compare: the result goes into the report
#   --version V      the version in the file names (default: the one in the package's name)
#
# Output, named per ENK-RLS-002: enkrnett-VERSION-BASELINE-PROFILE-sysupgrade.bin, the vendor
# kind of the profile when the Image Builder makes one (factory), the package manifest, and
# enkrnett-VERSION-BASELINE-PROFILE.build.json with sizes, hashes and the comparison.
#
# The mirror: each locked package is fetched by its path from ENKRNETT_MIRROR when set (the
# project's pinned mirror, RFC 013 §6), else from the release's download tree, and checked against
# the lock's SHA-256 either way; a mismatch stops the build. The Image Builder then sees the files
# in its packages/ directory and an empty repositories file, so it cannot reach a feed.
set -euo pipefail
. "$(dirname "$0")/lib.sh"

PROFILE=${1:?PROFILE}; shift
APK=""; OUT="$ROOT/openwrt/out/images"; FILES_IN=""; ADD=""; DROP=""; LIVE=0; TWICE=0; VERSION=""
while [ $# -gt 0 ]; do
  case "$1" in
    --apk) APK=$2; shift 2 ;;
    --out) OUT=$2; shift 2 ;;
    --files) FILES_IN=$2; shift 2 ;;
    --add) ADD=$2; shift 2 ;;
    --drop) DROP=$2; shift 2 ;;
    --live) LIVE=1; shift ;;
    --twice) TWICE=1; shift ;;
    --version) VERSION=$2; shift 2 ;;
    *) echo "usage: $0 PROFILE --apk ENKRNETT.apk [--out DIR] [--files DIR] [--add PKGS] [--drop PKGS] [--live] [--twice] [--version V]"; exit 2 ;;
  esac
done
[ -f "$APK" ] || { echo "--apk FILE is needed"; exit 2; }
PF=$(profile_file "$PROFILE")
TARGET=$(json_value "$PF" "['target']")
IB_PROFILE=$(json_value "$PF" "['image_builder_profile']")
VENDOR_KIND=$(json_value "$PF" "['install']['vendor_image_kind']")
FLASH_LIMIT=$(json_value "$PF" "['flash_image_limit_kib']")
BASELINE=$(lock_value "['openwrt']['baseline']")
DOWNLOADS=$(lock_value "['openwrt']['downloads']")
[ -n "$VERSION" ] || VERSION=$(version_of_apk "$APK")
[ -n "$VERSION" ] || { echo "the package name does not carry a version (enkrnett-VERSION-rN.apk); use --version"; exit 1; }
PKG_LOCK="$ROOT/openwrt/lock/packages/$PROFILE.json"
mkdir -p "$OUT"; OUT=$(cd "$OUT" && pwd)
NAME="enkrnett-$VERSION-$BASELINE-$PROFILE"
LOG="$OUT/$NAME.build.log"; : > "$LOG"
REPORT="$OUT/$NAME.build.json"

IB=$(image_builder "$TARGET")
echo "Image Builder: $(basename "$IB")"
[ -f "$IB/repositories.feeds" ] || cp "$IB/repositories" "$IB/repositories.feeds"

# ----- the packages the Image Builder may use ---------------------------------------------------
rm -f "$IB"/packages/enkrnett-*.apk "$IB/packages/packages.adb"
if [ "$LIVE" = 1 ]; then
  echo "packages: the live feeds (not a release build)"
  cp "$IB/repositories.feeds" "$IB/repositories"
  # Only the archive's own packages stay beside ours.
  for f in "$IB"/packages/*.apk; do
    case "$(basename "$f")" in base-files-*|kernel-*|libc-*) ;; *) rm -f "$f" ;; esac
  done
else
  [ -f "$PKG_LOCK" ] || { echo "no package lock for $PROFILE: openwrt/lock-packages.sh $PROFILE --apk ... (ENK-BUILD-007)"; exit 1; }
  [ "$(json_value "$PKG_LOCK" "['openwrt_release']")" = "$(lock_value "['openwrt']['release']")" ] || { echo "the package lock is for another OpenWrt release than inputs.json"; exit 1; }
  [ "$(json_value "$PKG_LOCK" "['image_builder']")" = "$(basename "$IB")" ] || { echo "the package lock was made with another Image Builder than the lock names"; exit 1; }
  BASE="${ENKRNETT_MIRROR:-$DOWNLOADS}"
  echo "packages: $(python3 -c "import json,sys; print(len(json.load(open(sys.argv[1]))['packages']))" "$PKG_LOCK") locked files from $BASE, checked against the lock"
  MIRROR="$CACHE/mirror"
  for f in "$IB"/packages/*.apk; do
    case "$(basename "$f")" in base-files-*|kernel-*|libc-*) ;; *) rm -f "$f" ;; esac
  done
  # The archive's own three must be the ones the lock saw.
  python3 -c "import json,sys; [print(p['file']) for p in json.load(open(sys.argv[1]))['from_image_builder']]" "$PKG_LOCK" | while read -r f; do
    [ -f "$IB/packages/$f" ] || { echo "the Image Builder archive lacks $f, which the lock expects"; exit 1; }
  done
  python3 -c "import json,sys; [print(p['path'], p['sha256']) for p in json.load(open(sys.argv[1]))['packages']]" "$PKG_LOCK" | \
  while read -r path sha; do
    fetch_checked "$BASE/$path" "$sha" "$MIRROR/$path" || exit 1
    cp "$MIRROR/$path" "$IB/packages/"
  done
  : > "$IB/repositories"
fi
cp "$APK" "$IB/packages/"

# ----- the overlay: the carried patches, the witness, the caller's files -------------------------
FILES=$(mktemp -d); trap 'rm -rf "$FILES"' EXIT
[ -n "$FILES_IN" ] && cp -r "$FILES_IN/." "$FILES/"
# Each carried patch names the installed file it changes (its header's "Installed file:" line);
# the file is taken from the locked package, patched, and placed in the image (ENK-CFG-004).
APK_TOOL="$IB/staging_dir/host/bin/apk"
for p in "$ROOT"/openwrt/patches/*.patch; do
  [ -f "$p" ] || continue
  spec=$(sed -n 's/^# Installed file: \(\/[^ ]*\) (package \([^)]*\)).*/\1 \2/p' "$p")
  [ -n "$spec" ] || { echo "$(basename "$p") has no 'Installed file: /path (package NAME)' line"; exit 1; }
  file=${spec% *}; pkg=${spec#* }
  src=$(ls "$IB"/packages/"$pkg"-[0-9]*.apk 2>/dev/null | head -1 || true)
  if [ -z "$src" ] && [ "$LIVE" = 1 ]; then src=$(ls "$IB"/dl/"$pkg"-[0-9]*.apk 2>/dev/null | head -1 || true); fi
  [ -n "$src" ] || { echo "no package $pkg among the locked files, needed by $(basename "$p")"; exit 1; }
  X=$(mktemp -d)
  "$APK_TOOL" extract --allow-untrusted --destination "$X" "$src" >/dev/null 2>&1 || { echo "cannot extract $src"; exit 1; }
  [ -f "$X$file" ] || { echo "$pkg does not install $file (the patch header is wrong)"; exit 1; }
  mkdir -p "$FILES$(dirname "$file")"
  cp "$X$file" "$FILES$file"
  patch -s "$FILES$file" < "$p" || { echo "$(basename "$p") does not apply to $file of the locked $pkg: review its removal condition"; exit 1; }
  rm -rf "$X"
  echo "patched: $file ($(basename "$p"))"
done
# The witness of an image install, which the package never places (platform.md §11).
mkdir -p "$FILES/usr/share/enkrnett"
echo "$NAME" > "$FILES/usr/share/enkrnett/from-image"

# ----- the build ------------------------------------------------------------------------------
PACKAGES="$(ib_packages_arg "$PF") $ADD"
for d in $DROP; do PACKAGES=$(echo " $PACKAGES " | sed "s/ $d / /g"); done
echo "PROFILE=$IB_PROFILE PACKAGES=$PACKAGES" >> "$LOG"

build_once() { # OUTDIR -> files named per ENK-RLS-002 in OUTDIR
  local out=$1 img
  rm -rf "$IB/bin"
  ( cd "$IB" && make image PROFILE="$IB_PROFILE" PACKAGES="$PACKAGES" FILES="$FILES" ) >>"$LOG" 2>&1 \
    || { echo "image build failed (see $LOG)"; tail -30 "$LOG"; return 1; }
  mkdir -p "$out"
  if [ "$TARGET" = x86/64 ]; then
    img=$(find "$IB/bin" -name "*-$IB_PROFILE-ext4-combined.img.gz" | head -1)
  else
    img=$(find "$IB/bin" -name "*-$IB_PROFILE-squashfs-sysupgrade.bin" | head -1)
  fi
  [ -n "$img" ] || { echo "no sysupgrade image for $IB_PROFILE (over the flash limit? see $LOG)"; return 1; }
  cp "$img" "$out/$NAME-sysupgrade.bin"
  if [ "$VENDOR_KIND" != none ]; then
    img=$(find "$IB/bin" -name "*-$IB_PROFILE-squashfs-$VENDOR_KIND.bin" | head -1)
    [ -n "$img" ] && cp "$img" "$out/$NAME-$VENDOR_KIND.bin"
  fi
  local m; m=$(find "$IB/bin" -name "*-$IB_PROFILE.manifest" | head -1)
  [ -n "$m" ] && cp "$m" "$out/$NAME.manifest"
  return 0
}

echo "## building $NAME"
build_once "$OUT"
SIZE=$(stat -c %s "$OUT/$NAME-sysupgrade.bin")
[ "$SIZE" -le $((FLASH_LIMIT * 1024)) ] || echo "warning: the sysupgrade image ($SIZE bytes) is over the profile's flash limit ($FLASH_LIMIT KiB)"
COMPARED=null
if [ "$TWICE" = 1 ]; then
  echo "## building $NAME again (ENK-BUILD-010)"
  AGAIN=$(mktemp -d)
  build_once "$AGAIN"
  COMPARED=true
  for f in "$OUT/$NAME"-*.bin; do
    if cmp -s "$f" "$AGAIN/$(basename "$f")"; then echo "identical: $(basename "$f")"; else echo "DIFFERS: $(basename "$f")"; COMPARED=false; fi
  done
  rm -rf "$AGAIN"
fi

python3 - "$REPORT" "$OUT" "$NAME" "$PROFILE" "$VERSION" "$BASELINE" "$TARGET" "$(basename "$IB")" "$LIVE" "$COMPARED" "$FLASH_LIMIT" <<'EOF'
import glob, hashlib, json, os, sys
report, out, name, profile, version, baseline, target, ib, live, compared, limit = sys.argv[1:]
images = []
for f in sorted(glob.glob(f"{out}/{name}-*.bin")):
    b = open(f, "rb").read()
    images.append({"file": os.path.basename(f), "size": len(b), "sha256": hashlib.sha256(b).hexdigest(), "sha512": hashlib.sha512(b).hexdigest()})
doc = {"format": 1, "profile": profile, "version": version, "baseline": baseline, "target": target, "image_builder": ib,
       "packages_from": "live feeds (not a release build)" if live == "1" else "the lock (openwrt/lock/packages)",
       "flash_image_limit_kib": int(limit), "images": images,
       "built_twice_identical": None if compared == "null" else compared == "true"}
json.dump(doc, open(report, "w"), indent=1); open(report, "a").write("\n")
for i in images:
    print(f"image: {i['file']} ({i['size']} bytes) sha256 {i['sha256'][:16]}…")
print("report:", report)
EOF
[ "$COMPARED" != false ] || { echo "the two builds differ (ENK-BUILD-010: stated in the notes; not a failure)"; }
