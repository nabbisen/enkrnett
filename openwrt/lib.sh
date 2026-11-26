# Shared by the openwrt/ scripts and tests/vm: the lock, the pinned archives, the device profiles.
# Source it; it defines functions and the ROOT, LOCK and CACHE variables, nothing else.
#
#   ROOT   the repository
#   LOCK   openwrt/lock/inputs.json
#   CACHE  openwrt/.cache (ENKRNETT_CACHE_DIR overrides it): sdk/, ib/<target>/, mirror/
#
# Every archive is fetched by the URL the lock names and checked against the lock's SHA-256 before
# it is extracted (ENK-BUILD-007); a mismatch stops the caller.

ROOT=${ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}
LOCK="$ROOT/openwrt/lock/inputs.json"
CACHE="${ENKRNETT_CACHE_DIR:-$ROOT/openwrt/.cache}"

# lock_value "['openwrt']['release']"
lock_value() { python3 -c "import json,sys; d=json.load(open(sys.argv[1])); print(eval('d'+sys.argv[2]))" "$LOCK" "$1"; }

# json_value FILE "['a']['b']"
json_value() { python3 -c "import json,sys; d=json.load(open(sys.argv[1])); print(eval('d'+sys.argv[2]))" "$1" "$2"; }

# json_list FILE "['a']['list']"  -> one item per line
json_list() { python3 -c "import json,sys; d=json.load(open(sys.argv[1])); [print(x) for x in eval('d'+sys.argv[2])]" "$1" "$2"; }

# target_dir ramips/mt7621 -> ramips-mt7621
target_dir() { echo "${1//\//-}"; }

# fetch_checked URL SHA256 DEST: fetches unless DEST already matches; a mismatch removes DEST and fails.
fetch_checked() {
  local url=$1 sha=$2 dest=$3
  if [ -f "$dest" ] && echo "$sha  $dest" | sha256sum -c --quiet - 2>/dev/null; then return 0; fi
  mkdir -p "$(dirname "$dest")"
  curl -fsSL -o "$dest.part" "$url" || { rm -f "$dest.part"; echo "fetch failed: $url" >&2; return 1; }
  if ! echo "$sha  $dest.part" | sha256sum -c --quiet - 2>/dev/null; then
    rm -f "$dest.part"; echo "checksum mismatch (ENK-BUILD-007): $url" >&2; return 1
  fi
  mv -f "$dest.part" "$dest"
}

# ensure_archive URL SHA256 DIR GLOB -> the extracted directory under DIR (fetched and extracted on first use)
ensure_archive() {
  local url=$1 sha=$2 dir=$3 glob=$4 found
  found=$(ls -d "$dir"/$glob/ 2>/dev/null | head -1 || true)
  if [ -z "$found" ]; then
    local tar="$dir/$(basename "$url")"
    echo "fetching $(basename "$url")" >&2
    fetch_checked "$url" "$sha" "$tar" || return 1
    tar -C "$dir" --zstd -xf "$tar"
    found=$(ls -d "$dir"/$glob/ | head -1)
  fi
  echo "${found%/}"
}

# image_builder TARGET -> the Image Builder directory for the target (e.g. ramips/mt7621), from the lock
image_builder() {
  local t=$1 dir="$CACHE/ib/$(target_dir "$1")"
  mkdir -p "$dir"
  ensure_archive "$(lock_value "['openwrt']['targets']['$t']['image_builder']['url']")" \
    "$(lock_value "['openwrt']['targets']['$t']['image_builder']['sha256']")" "$dir" 'openwrt-imagebuilder-*'
}

# sdk TARGET -> the SDK directory for the target, from the lock
sdk() {
  local t=$1 dir="$CACHE/sdk/$(target_dir "$1")"
  mkdir -p "$dir"
  ensure_archive "$(lock_value "['openwrt']['targets']['$t']['sdk']['url']")" \
    "$(lock_value "['openwrt']['targets']['$t']['sdk']['sha256']")" "$dir" 'openwrt-sdk-*'
}

# profile_file ID -> openwrt/profiles/ID/profile.json (fails when there is none)
profile_file() {
  local f="$ROOT/openwrt/profiles/$1/profile.json"
  [ -f "$f" ] || { echo "no device profile $1 (openwrt/profiles/)" >&2; return 1; }
  echo "$f"
}

# ib_packages_arg PROFILE_FILE -> the PACKAGES= value the profile asks for: "-removed ... added ..."
ib_packages_arg() {
  { json_list "$1" "['image']['packages_remove']" | sed 's/^/-/'; json_list "$1" "['image']['packages_add']"; } | tr '\n' ' '
}

# version_of_apk FILE -> the version in enkrnett-VERSION-rN.apk
version_of_apk() { basename "$1" | sed -n 's/^enkrnett-\(.*\)-r[0-9]*\.apk$/\1/p'; }
