#!/bin/bash
# Assembles what a release publishes beside its images (chapter 11 §6 item 6; ENK-BUILD-012):
# the images named per ENK-RLS-002, their hashes, the lock, the written source offer, the build
# report, and a draft of the release index for the signing host to sign. Nothing here signs
# anything: the key is on the signing host and never on CI (ENK-RLS-007).
#
#   openwrt/release-assemble.sh VERSION [--images DIR] [--out DIR] [--sequence N --notes-url URL]
#
#   VERSION      must equal the workspace's, the package's and the UI's version (checked)
#   --images     where openwrt/build-image.sh put the images and reports (default openwrt/out/images)
#   --out        the release directory (default openwrt/out/release/VERSION)
#   --sequence   with --notes-url: also write index.json, the next publication's index, with
#                size, hash, minimum free memory and duration per profile (ENK-RLS-006)
#
# Then, on the signing host (docs/src/contributors/releasing.md):
#   enkrnett-release sign release.sec IMAGE      for every .bin  -> IMAGE.sig   (ENK-RLS-004)
#   enkrnett-release sign release.sec index.json                 -> index.json.sig
set -euo pipefail
. "$(dirname "$0")/lib.sh"

VERSION=${1:?VERSION}; shift
IMAGES="$ROOT/openwrt/out/images"; OUT=""; SEQUENCE=""; NOTES_URL=""
while [ $# -gt 0 ]; do
  case "$1" in
    --images) IMAGES=$2; shift 2 ;;
    --out) OUT=$2; shift 2 ;;
    --sequence) SEQUENCE=$2; shift 2 ;;
    --notes-url) NOTES_URL=$2; shift 2 ;;
    *) echo "usage: $0 VERSION [--images DIR] [--out DIR] [--sequence N --notes-url URL]"; exit 2 ;;
  esac
done
[ -n "$OUT" ] || OUT="$ROOT/openwrt/out/release/$VERSION"
BASELINE=$(lock_value "['openwrt']['baseline']")
RELEASE=$(lock_value "['openwrt']['release']")
DOWNLOADS=$(lock_value "['openwrt']['downloads']")

echo "## the version in every place it is written"
fail=0
check_version() { if [ "$2" = "$VERSION" ]; then echo "  ok   $1: $2"; else echo "  FAIL $1: $2 (release $VERSION)"; fail=1; fi; }
check_version "Cargo.toml [workspace.package]" "$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)"
check_version "openwrt/package/enkrnett/Makefile PKG_VERSION" "$(sed -n 's/^PKG_VERSION:=\(.*\)/\1/p' "$ROOT/openwrt/package/enkrnett/Makefile")"
check_version "ui/package.json" "$(json_value "$ROOT/ui/package.json" "['version']")"
check_version "tests/browser/package.json" "$(json_value "$ROOT/tests/browser/package.json" "['version']")"
[ "$fail" = 0 ] || { echo "the versions disagree; nothing assembled"; exit 1; }

echo "## the images and their reports"
REPORTS=$(ls "$IMAGES"/enkrnett-"$VERSION"-"$BASELINE"-*.build.json 2>/dev/null || true)
[ -n "$REPORTS" ] || { echo "no images of $VERSION in $IMAGES (openwrt/build-image.sh PROFILE --apk ...)"; exit 1; }
rm -rf "$OUT"; mkdir -p "$OUT/lock/packages" "$OUT/lock/patches"
for r in $REPORTS; do
  [ "$(json_value "$r" "['packages_from']")" = "the lock (openwrt/lock/packages)" ] || { echo "$(basename "$r"): built from the live feeds, not from the lock (ENK-BUILD-008); nothing assembled"; exit 1; }
  python3 -c "import json,sys; [print(i['file']) for i in json.load(open(sys.argv[1]))['images']]" "$r" | while read -r f; do cp "$IMAGES/$f" "$OUT/"; echo "  $f"; done
  base=$(basename "$r" .build.json)
  [ -f "$IMAGES/$base.manifest" ] && cp "$IMAGES/$base.manifest" "$OUT/"
  cp "$r" "$OUT/"
done

echo "## the package for the expert install (ENK-INST-005)"
# The .apk the images were built with, named with its architecture; verified by the sums below.
for apk in "$ROOT"/openwrt/out/enkrnett-"$VERSION"-r*.apk; do
  [ -f "$apk" ] || continue
  arch=$(python3 -c "import json,glob,sys; print(next(json.load(open(f))['package_arch'] for f in glob.glob('$ROOT/openwrt/profiles/*/profile.json') if json.load(open(f))['target']!='x86/64'))")
  cp "$apk" "$OUT/$(basename "$apk" .apk)-$arch.apk"; echo "  $(basename "$apk" .apk)-$arch.apk"
done

echo "## hashes"
( cd "$OUT" && sha256sum ./*.bin ./*.apk 2>/dev/null | sed 's#\./##' > SHA256SUMS && sha512sum ./*.bin ./*.apk 2>/dev/null | sed 's#\./##' > SHA512SUMS )
cat "$OUT/SHA256SUMS" | sed 's/^/  /'

echo "## the lock (ENK-BUILD-007)"
cp "$LOCK" "$OUT/lock/inputs.json"
cp "$ROOT"/openwrt/lock/packages/*.json "$OUT/lock/packages/"
cp "$ROOT/Cargo.lock" "$OUT/lock/Cargo.lock"
cp "$ROOT/ui/package-lock.json" "$OUT/lock/ui-package-lock.json"
for p in "$ROOT"/openwrt/patches/*.patch; do [ -f "$p" ] && cp "$p" "$OUT/lock/patches/"; done
( cd "$OUT/lock" && find . -type f ! -name SHA256SUMS | sort | xargs sha256sum | sed 's#\./##' > SHA256SUMS )

echo "## the build report (ENK-BUILD-009, 010)"
python3 - "$OUT" "$VERSION" "$BASELINE" "$RELEASE" <<'EOF' > "$OUT/build-report.md"
import glob, json, sys
out, version, baseline, release = sys.argv[1:]
print(f"# Build report: Enkrnett {version} on OpenWrt {baseline} ({release})\n")
print("Every image was built by `openwrt/build-image.sh` from `lock/` (the pinned Image Builder, the locked")
print("packages, no live feed: ENK-BUILD-007, 008). Where a profile was built twice, the two results were")
print("compared byte for byte (ENK-BUILD-010); the release notes state this table.\n")
print("| Profile | Image | Size | SHA-256 | Built twice |")
print("|---|---|---:|---|---|")
for r in sorted(glob.glob(f"{out}/*.build.json")):
    d = json.load(open(r))
    twice = {None: "not compared", True: "identical", False: "**differs**"}[d.get("built_twice_identical")]
    for i in d["images"]:
        print(f"| `{d['profile']}` | `{i['file']}` | {i['size']:,} | `{i['sha256']}` | {twice} |")
print("\nThe daemon and the UI bundle are built twice on every CI run and compared (ENK-BUILD-009):")
print("`.github/workflows/ci.yml`, jobs *mips* and *ui*.")
EOF
cat "$OUT/build-report.md" | sed -n 6,20p | sed 's/^/  /'

echo "## the source offer (ENK-BUILD-012)"
python3 - "$OUT" "$VERSION" "$BASELINE" "$RELEASE" "$DOWNLOADS" <<'EOF' > "$OUT/source-offer.md"
import glob, json, sys
out, version, baseline, release, downloads = sys.argv[1:]
locks = [json.load(open(f)) for f in sorted(glob.glob(f"{out}/lock/packages/*.json"))]
print(f"# Source offer for Enkrnett {version}\n")
print(f"The firmware images of Enkrnett {version} contain OpenWrt {release} and packages from its feeds,")
print("most of them under the GNU General Public License (version 2 or later) and other free software")
print("licenses, together with Enkrnett itself (Apache License 2.0). This is the written offer of the")
print("corresponding source code for every component whose license requires one.\n")
print("## OpenWrt\n")
revs = sorted({l["source"]["openwrt_revision"] for l in locks})
print(f"- Release: OpenWrt {release}, revision {', '.join(revs)} (`version.buildinfo` of the release).")
print(f"- Source: `https://git.openwrt.org/openwrt/openwrt.git`, tag `v{release}` (also on GitHub, `openwrt/openwrt`).")
feeds = sorted({f for l in locks for f in l["source"]["feeds"]})
print("- Package feeds at the commits the release was built from (`feeds.buildinfo`):")
for f in feeds:
    print(f"  - `{f}`")
print(f"- Binary packages as installed: listed per device in `lock/packages/*.json` with version and SHA-256;")
print(f"  the files were taken from `{downloads}/` (or the project's mirror of it).")
print("- Build configuration: `config.buildinfo` beside each target's images at the address above.\n")
print("## Carried patches\n")
patches = sorted(glob.glob(f"{out}/lock/patches/*.patch"))
if patches:
    print("Applied to the files of the locked packages at image build (`openwrt/build-image.sh`); each")
    print("header names its upstream commit and removal condition:\n")
    for p in patches:
        print(f"- `lock/patches/{p.split('/')[-1]}`")
else:
    print("None.")
print("\n## Enkrnett\n")
print(f"- Source: `https://github.com/nabbisen/enkrnett`, tag `{version}`; license Apache-2.0 (`LICENSE`, `NOTICE`).")
print(f"- Every other input (Image Builder archive, Rust toolchain, Cargo and npm locks) is in `lock/`;")
print("  `docs/src/contributors/releasing.md` says how the images are rebuilt from it.\n")
print("## How to obtain the source\n")
print("Everything above is public at the addresses given. Anyone who received these images and cannot")
print("reach those addresses may ask the project (the repository's issue tracker) for a copy on a physical")
print("medium at the cost of the medium and its shipping; the offer stands for three years from the")
print("publication of this release.")
EOF
sed -n 1,12p "$OUT/source-offer.md" | sed 's/^/  /'

if [ -n "$SEQUENCE" ] && [ -n "$NOTES_URL" ]; then
  echo "## the index to sign (ENK-RLS-006)"
  TOOL="$ROOT/target/release/enkrnett-release"
  [ -x "$TOOL" ] || ( cd "$ROOT" && cargo build --release -q -p enkrnett-release )
  ARGS=(index --sequence "$SEQUENCE" --out "$OUT/index.json")
  for r in $REPORTS; do
    profile=$(json_value "$r" "['profile']")
    pf=$(profile_file "$profile")
    ARGS+=(--release "$profile" "$VERSION" "$BASELINE" "$BASELINE" "$OUT/enkrnett-$VERSION-$BASELINE-$profile-sysupgrade.bin" \
           "$(json_value "$pf" "['update']['min_free_kib']")" "$NOTES_URL" "$(json_value "$pf" "['update']['install_seconds']")")
  done
  "$TOOL" "${ARGS[@]}"
  echo "  index.json written (sequence $SEQUENCE); an upgrade across baselines needs its from_baseline set by hand"
fi

echo "## assembled: $OUT"
ls -1 "$OUT" | sed 's/^/  /'
echo
echo "Next, on the signing host (never on CI): sign every .bin and index.json with the release key;"
echo "then publish this directory with the .sig files beside the index (docs/src/contributors/releasing.md)."
