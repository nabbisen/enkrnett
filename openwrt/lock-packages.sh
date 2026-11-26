#!/bin/bash
# Writes the package lock of a device profile (ENK-BUILD-007): every OpenWrt package the image of
# that profile installs, with its version, its path under the release's download tree and its
# SHA-256. The image build (openwrt/build-image.sh) installs exactly these files and nothing from
# a live feed (ENK-BUILD-008). Run it deliberately, when the baseline moves or a package is added
# to a profile; review the diff; commit it with the change.
#
#   openwrt/lock-packages.sh PROFILE --apk ENKRNETT.apk
#
#   PROFILE  a directory of openwrt/profiles
#   --apk    an enkrnett package of the profile's architecture, so that the Image Builder can
#            resolve its dependencies; the package itself is not locked (it is what the release
#            builds)
#
# How: the pinned Image Builder of the profile's target resolves the profile's package set
# (`make manifest`, which reads the live 25.12.5 feeds this once); each resolved package is then
# fetched from the feed that lists it and hashed. The three packages the Image Builder archive
# carries itself (base-files, kernel, libc) are covered by the archive's hash and listed apart.
set -euo pipefail
. "$(dirname "$0")/lib.sh"

PROFILE=${1:?PROFILE}; shift
APK=""
while [ $# -gt 0 ]; do
  case "$1" in
    --apk) APK=$2; shift 2 ;;
    *) echo "usage: $0 PROFILE --apk ENKRNETT.apk"; exit 2 ;;
  esac
done
[ -f "$APK" ] || { echo "--apk FILE is needed (openwrt/build-mips.sh --package, or tests/vm/package.sh for x86_64)"; exit 2; }
PF=$(profile_file "$PROFILE")
TARGET=$(json_value "$PF" "['target']")
IB_PROFILE=$(json_value "$PF" "['image_builder_profile']")
ARCH=$(json_value "$PF" "['package_arch']")
RELEASE=$(lock_value "['openwrt']['release']")
DOWNLOADS=$(lock_value "['openwrt']['downloads']")
OUT="$ROOT/openwrt/lock/packages/$PROFILE.json"

IB=$(image_builder "$TARGET")
echo "Image Builder: $(basename "$IB")"
# The archive's own feeds file is kept once as repositories.feeds; the build rewrites repositories.
[ -f "$IB/repositories.feeds" ] || cp "$IB/repositories" "$IB/repositories.feeds"
cp "$IB/repositories.feeds" "$IB/repositories"
# The archive's own packages: whatever was in packages/ before anything was added.
IB_OWN=$(ls "$IB/packages"/*.apk | xargs -n1 basename | grep -v '^enkrnett-' | sort)
rm -f "$IB"/packages/enkrnett-*.apk "$IB/packages/packages.adb"
cp "$APK" "$IB/packages/"

echo "## resolving the package set of $PROFILE ($IB_PROFILE on $TARGET)"
PACKAGES=$(ib_packages_arg "$PF")
MANIFEST=$(mktemp)
( cd "$IB" && make -s manifest PROFILE="$IB_PROFILE" PACKAGES="$PACKAGES" 2>"$MANIFEST.err" ) > "$MANIFEST" \
  || { echo "manifest failed:"; tail -20 "$MANIFEST.err"; exit 1; }
echo "$(wc -l < "$MANIFEST") packages resolved"

echo "## the feeds' indexes"
W=$(mktemp -d); trap 'rm -rf "$W" "$MANIFEST" "$MANIFEST.err"' EXIT
i=0
while read -r repo; do
  [ -n "$repo" ] || continue
  i=$((i + 1))
  curl -fsSL -o "$W/feed$i.json" "$(dirname "$repo")/index.json"
  echo "$(dirname "$repo")" > "$W/feed$i.url"
done < "$IB/repositories.feeds"

echo "## fetching and hashing"
VERSION_INFO=$(curl -fsSL "$DOWNLOADS/targets/$TARGET/version.buildinfo")
FEEDS_INFO=$(curl -fsSL "$DOWNLOADS/targets/$TARGET/feeds.buildinfo")
MIRROR="$CACHE/mirror"
python3 - "$MANIFEST" "$W" "$DOWNLOADS" "$MIRROR" "$OUT" "$PROFILE" "$TARGET" "$ARCH" "$RELEASE" "$IB_OWN" "$VERSION_INFO" "$FEEDS_INFO" "$(basename "$IB")" <<'EOF'
import glob, hashlib, json, os, subprocess, sys
manifest, w, downloads, mirror, out, profile, target, arch, release, ib_own, version_info, feeds_info, ib_name = sys.argv[1:]
own = {}
for f in ib_own.split():
    # name-version.apk, the version starting at the first "-" followed by a digit
    stem = f[:-4]
    for i, c in enumerate(stem):
        if c == '-' and i + 1 < len(stem) and stem[i + 1].isdigit():
            own[stem[:i]] = stem[i + 1:]
            break
feeds = []
for u in sorted(glob.glob(f"{w}/feed*.url")):
    url = open(u).read().strip()
    idx = json.load(open(u[:-4] + ".json"))
    feeds.append((url, idx.get("packages", {})))
packages, from_ib, missing = [], [], []
for line in open(manifest):
    parts = line.split(" - ")
    if len(parts) != 2:
        continue
    name, version = parts[0].strip(), parts[1].strip()
    if name == "enkrnett":
        continue
    if name in own:
        from_ib.append({"name": name, "version": own[name], "file": f"{name}-{own[name]}.apk"})
        continue
    # The feed's index lists a library under its plain name; the file carries the ABI version
    # (libubox20260213-VERSION.apk is listed as libubox).
    plain = name.rstrip("0123456789")
    feed = next((url for url, pk in feeds if pk.get(name) == version), None) \
        or next((url for url, pk in feeds if pk.get(plain) == version), None)
    if feed is None:
        missing.append(f"{name} {version}")
        continue
    rel = feed[len(downloads) + 1:] if feed.startswith(downloads + "/") else feed
    path = f"{rel}/{name}-{version}.apk"
    dest = os.path.join(mirror, path)
    if not os.path.isfile(dest):
        os.makedirs(os.path.dirname(dest), exist_ok=True)
        subprocess.run(["curl", "-fsSL", "-o", dest + ".part", f"{downloads}/{path}"], check=True)
        os.replace(dest + ".part", dest)
    sha = hashlib.sha256(open(dest, "rb").read()).hexdigest()
    packages.append({"name": name, "version": version, "path": path, "sha256": sha})
if missing:
    sys.exit("not found in any feed: " + ", ".join(missing))
packages.sort(key=lambda p: p["name"])
from_ib.sort(key=lambda p: p["name"])
doc = {
    "format": 1,
    "comment": "The OpenWrt packages the image of this profile installs (ENK-BUILD-007): version, path under openwrt.downloads of inputs.json, SHA-256. openwrt/build-image.sh fetches each by its path (from ENKRNETT_MIRROR when set, the project's pinned mirror of RFC 013 §6), checks the hash, and gives the Image Builder these files and no feed (ENK-BUILD-008). Written by openwrt/lock-packages.sh; the project's own package, enkrnett, is built from the lock's toolchain and is not listed.",
    "profile": profile,
    "target": target,
    "package_arch": arch,
    "openwrt_release": release,
    "source": {
        "comment": "OpenWrt's own record of what built these packages (version.buildinfo and feeds.buildinfo of the target): the tag's revision and the feed commits, for the source offer of ENK-BUILD-012.",
        "openwrt_revision": version_info.strip(),
        "feeds": [l for l in feeds_info.strip().splitlines() if l.strip()],
    },
    "image_builder": ib_name,
    "from_image_builder": from_ib,
    "packages": packages,
}
json.dump(doc, open(out, "w"), indent=1, ensure_ascii=False)
open(out, "a").write("\n")
print(f"{len(packages)} packages locked, {len(from_ib)} from the Image Builder archive")
print(f"lock written to {out}")
EOF
