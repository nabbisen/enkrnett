#!/bin/bash
# Installing an update on the VM (chapter 11 §3: ENK-UPD-002 to 007, ENK-API-012). The host
# builds two successor releases from this source (tests/vm/successor.sh): 9.9.9, a complete
# image, and 9.9.10 without the VM's wireless driver, the deliberately broken successor of
# ENK-UPD-005. Both are signed with a test key and served with a signed index over HTTPS, as the
# update check test does. The owner's request installs 9.9.9: the daemon downloads and verifies
# the image, hands it to the root side, sysupgrade flashes it, and the device comes back as 9.9.9
# with the owner, the key, the settings and the identifier kept and nothing else (ENK-UPD-004);
# its first boot reports the update done. Then 9.9.10 over it: installed, its health check fails,
# the device says so and keeps serving the owner's settings. The refusals of the root side are
# exercised too (a stranger's signature: usign refuses what the daemon would also refuse).
# At the end the disk is put back as it was before the test, so that the chain continues on the
# same release (the VM's disk is a file on the host).
#
#   tests/vm/update-install.sh VM_DIR     (after tests/vm/claim.sh: a CLAIMED device)
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
VM=$(cd "${1:?VM_DIR}" && pwd)
PASSWORD="correct horse battery staple"
PORT=18087
HTTPS_PORT=18444
HOST_NAME=updates.enkrnett.test
TOOL="$ROOT/target/release/enkrnett-release"
[ -x "$TOOL" ] || ( cd "$ROOT" && cargo build --release -q -p enkrnett-release )
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)
OUT="$ROOT/openwrt/out/successor"
W=$(mktemp -d)
SERVER=""; TUNNEL=""
# Whatever happens, the disk goes back to how it was before the test (a successor may be running).
cleanup() {
  [ -n "$SERVER" ] && kill "$SERVER" 2>/dev/null || true
  [ -n "$TUNNEL" ] && kill "$TUNNEL" 2>/dev/null || true
  if [ -f "$VM/disk.img.before-update" ]; then
    echo "## the disk as it was before the test (cleanup)"
    ( cd "$VM" && ./vm.sh stop >/dev/null 2>&1; sleep 1; mv -f disk.img.before-update disk.img; rm -rf successor-files; ./vm.sh start >/dev/null 2>&1 && python3 vmsh.py wait 240 >/dev/null 2>&1 ) || true
  fi
  rm -rf "$W"; return 0; }
trap cleanup EXIT
FAILS=0
pass() { echo "  ok   $*"; }
fail() { echo "  FAIL $*"; FAILS=$((FAILS + 1)); }
check() { if [ "$2" = "$3" ]; then pass "$1"; else fail "$1 (got '$2', expected '$3')"; fi; }
api() { local method=$1 path=$2 token=${3:-} body=${4:-}
  local args=(-s -o /tmp/enkrnett-i-body -w '%{http_code}' --connect-timeout 5 -m 60 -X "$method" "http://127.0.0.1:$PORT/api/1/$path" -H "Host: 192.168.207.1")
  [ -n "$token" ] && args+=(-H "Enkrnett-Session: $token")
  [ "$method" = POST ] && args+=(-H 'Content-Type: application/json' -d "${body:-{\}}")
  curl "${args[@]}" 2>/dev/null || echo 000; }
jq_() { python3 -c "import json,sys; d=json.load(open('/tmp/enkrnett-i-body')); print(eval('d'+sys.argv[1]))" "$1" 2>/dev/null || echo "?"; }
# How many log lines match; 0 when none or when the VM does not answer.
vm_log_count() { local n; n=$(./vmssh "logread | grep -c '$1' || true" 2>/dev/null | tr -d '\r' | tail -1); echo "${n:-0}"; }

cd "$VM"
./vm.sh start >/dev/null || true
python3 vmsh.py wait 120 >/dev/null 2>&1 || true

echo "## the test key and authority; the two successor images, signed"
"$TOOL" keygen -c "enkrnett VM test" "$W/test" >/dev/null
"$TOOL" keygen -c "a stranger" "$W/stranger" >/dev/null
openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -keyout "$W/srv.key" -out "$W/srv.crt" -days 2 \
  -subj "/CN=$HOST_NAME" -addext "subjectAltName=DNS:$HOST_NAME,IP:10.0.2.2" >/dev/null 2>&1
# The successor images carry the key and the authority, so that the second update can follow.
rm -rf "$VM/successor-files"; mkdir -p "$VM/successor-files/usr/share/enkrnett/keys" "$VM/successor-files/etc/ssl/certs"
cp "$W/test.pub" "$VM/successor-files/usr/share/enkrnett/keys/vm-test.pub"
cp "$W/srv.crt" "$VM/successor-files/etc/ssl/certs/enkrnett-vm-test.crt"
"$ROOT/tests/vm/successor.sh" "$VM" 9.9.9 "$OUT" | sed 's/^/    /'
"$ROOT/tests/vm/successor.sh" "$VM" 9.9.10 "$OUT" broken | sed 's/^/    /'
GOOD="$OUT/enkrnett-9.9.9-25.12-x86_64_generic-sysupgrade.bin"
BROKEN="$OUT/enkrnett-9.9.10-25.12-x86_64_generic-sysupgrade.bin"
mkdir -p "$W/www/a" "$W/www/b" "$W/www/stranger"
# Index a: 9.9.9 for this device from the installed version; b: 9.9.10 (the broken one) for 9.9.9.
cp "$GOOD" "$W/www/a/"; "$TOOL" sign "$W/test.sec" "$W/www/a/$(basename "$GOOD")" >/dev/null
"$TOOL" index --sequence 11 --out "$W/www/a/index.json" --release x86_64_generic 9.9.9 25.12 25.12 "$GOOD" 16384 https://example.org/releases/9.9.9 180 >/dev/null
"$TOOL" sign "$W/test.sec" "$W/www/a/index.json" >/dev/null
cp "$BROKEN" "$W/www/b/"; "$TOOL" sign "$W/test.sec" "$W/www/b/$(basename "$BROKEN")" >/dev/null
"$TOOL" index --sequence 12 --out "$W/www/b/index.json" --release x86_64_generic 9.9.10 25.12 25.12 "$BROKEN" 16384 https://example.org/releases/9.9.10 180 >/dev/null
"$TOOL" sign "$W/test.sec" "$W/www/b/index.json" >/dev/null
# stranger: the good index, but the image signed by someone else (the index itself is fine).
cp "$W/www/a/index.json" "$W/www/a/index.json.sig" "$W/www/stranger/"; cp "$GOOD" "$W/www/stranger/"
"$TOOL" sign "$W/stranger.sec" "$W/www/stranger/$(basename "$GOOD")" >/dev/null
pass "images signed: $(basename "$GOOD") ($(stat -c %s "$GOOD") bytes), $(basename "$BROKEN")"

echo "## an HTTPS server on the host"
python3 - "$W/www" "$HTTPS_PORT" "$W/srv.crt" "$W/srv.key" <<'PY' &
import http.server, os, ssl, sys
www, port, crt, key = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER); ctx.load_cert_chain(crt, key)
os.chdir(www)
class H(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *a): pass
srv = http.server.ThreadingHTTPServer(('127.0.0.1', port), H)
srv.socket = ctx.wrap_socket(srv.socket, server_side=True)
srv.serve_forever()
PY
SERVER=$!
sleep 1

echo "## the VM: the disk as it is now, the authority, the host name, the key"
./vmssh "sync" >/dev/null 2>&1 || true
cp --sparse=always "$VM/disk.img" "$VM/disk.img.before-update"
./vmscp "$W/srv.crt" /etc/ssl/certs/enkrnett-vm-test.crt >/dev/null
./vmssh "grep -q '$HOST_NAME' /etc/hosts || echo '10.0.2.2 $HOST_NAME' >> /etc/hosts; mkdir -p /usr/share/enkrnett/keys" >/dev/null
./vmscp "$W/test.pub" /usr/share/enkrnett/keys/vm-test.pub >/dev/null

tunnel() { [ -n "$TUNNEL" ] && { kill "$TUNNEL" 2>/dev/null || true; }; sleep 0.5
  ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -o ExitOnForwardFailure=yes -i id_ed25519 -p 2222 -N -L "$PORT:192.168.207.1:8080" root@127.0.0.1 &
  TUNNEL=$!; sleep 2; }
tunnel
T=""
# The daemon answers once its start-up health check is through (up to 60 s on a broken release).
login() { local s i; for i in $(seq 1 18); do s=$(api POST login '' "{\"password\":\"$PASSWORD\"}"); [ "$s" = 200 ] && break; sleep 5; done; [ "$s" = 200 ] || fail "login ($s)"; T=$(jq_ "['token']"); }
set_url() {
  local n0; n0=$(vm_log_count 'enkrnettd started')
  ./vmssh "uci set enkrnett.main.update_index='$1'; uci commit enkrnett; /etc/init.d/enkrnett restart" >/dev/null 2>&1
  for i in $(seq 1 30); do [ "$(vm_log_count 'enkrnettd started')" -gt "$n0" ] && break; sleep 2; done
  sleep 1; login
}
# Waits for the VM to go down and come back after a sysupgrade; then a new tunnel and a login.
wait_reboot() {
  local i
  for i in $(seq 1 90); do ./vmssh true >/dev/null 2>&1 || break; sleep 2; done
  python3 vmsh.py wait 300 >/dev/null 2>&1 || true
  for i in $(seq 1 60); do ./vmssh true >/dev/null 2>&1 && break; sleep 3; done
  sleep 20   # the daemon's start-up health check and the networks
  tunnel; login
}

echo "## the owner installs 9.9.9 (ENK-UPD-002, 003)"
set_url "https://$HOST_NAME:$HTTPS_PORT/a/index.json"
GUEST_NAME=$(api GET status "$T" >/dev/null; jq_ "['settings']['guest']['name']")
DEVICE_ID=$(api GET device "$T" >/dev/null; jq_ "['device_id']")
api POST update/check "$T" '{"enable_checks": true}' >/dev/null
check "the release offered" "$(jq_ "['available']['version']")" 9.9.9
check "ENK-UPD-006 no undo promised" "$(jq_ "['can_undo']")" False
S=$(api POST update/install "$T" '{}'); check "ENK-UPD-002 install accepted (202)" "$S" 202
STATE=$(jq_ "['state']"); [ "$STATE" = downloading ] || [ "$STATE" = verifying ] || [ "$STATE" = installing ] && pass "state $STATE" || fail "state after install: $STATE"
# ENK-API-012 while it runs.
S=$(api POST update/check "$T" '{}'); check "ENK-API-012 a POST meanwhile: 409" "$S" 409
check "update_in_progress" "$(jq_ "['error']['code']")" update_in_progress
# Follow the shape until the device goes away.
for i in $(seq 1 60); do S=$(api GET update "$T"); [ "$S" = 200 ] || break; STATE=$(jq_ "['state']"); [ "$STATE" = installing ] && break; sleep 1; done
check "the shape reached installing" "$STATE" installing
# The marker is on flash before the hand-over (ENK-UPD-005). The root side's own words go to the
# log, which lives in RAM and is gone with the restart; that the device comes back as 9.9.9 is the
# proof that it verified and flashed.
./vmssh "cat /etc/enkrnett/update.json" > "$W/marker" 2>/dev/null || true
check "the update marker says installing" "$(python3 -c "import json,sys; print(json.load(open(sys.argv[1]))['stage'])" "$W/marker" 2>/dev/null || echo '?')" installing
OLD_TOKEN=$T
wait_reboot

echo "## the device came back as 9.9.9 (ENK-UPD-004, 005)"
api GET device "$T" >/dev/null
check "ENK-UPD-005 the new release runs" "$(jq_ "['version']['enkrnett']")" 9.9.9
check "ENK-UPD-004 the device identifier kept" "$(jq_ "['device_id']")" "$DEVICE_ID"
check "installed from an image now" "$(jq_ "['installed_as']")" image
api GET update "$T" >/dev/null
check "ENK-UPD-005 the update reported done" "$(jq_ "['last_result']['result']")" done
check "no reason" "$(jq_ "['last_result']['reason']")" None
check "state idle" "$(jq_ "['state']")" idle
check "installed version 9.9.9" "$(jq_ "['installed']['version']")" 9.9.9
api GET status "$T" >/dev/null
check "ENK-UPD-004 the settings kept (guest name)" "$(jq_ "['settings']['guest']['name']")" "$GUEST_NAME"
check "the owner's password still logs in (login above)" "$([ -n "$T" ] && echo yes)" yes
S=$(api GET update "$OLD_TOKEN"); check "ENK-UPD-004 the old session was not carried (401)" "$S" 401
./vmssh "ls /etc/enkrnett | sort | tr '\n' ' '" > "$W/ls" 2>/dev/null || true
check "ENK-UPD-004 only the state and the marker on flash" "$(tr -d '\r' < "$W/ls" | sed 's/ *$//')" "state.json update.json"
[ "$(vm_log_count 'update_done version=9.9.9')" -ge 1 ] && pass "ENK-API-029 update_done logged" || fail "no update_done in the log"
./vmssh "ls /tmp/enkrnett-update 2>/dev/null | wc -l" > "$W/n" 2>/dev/null || echo 0 > "$W/n"; check "nothing left in RAM" "$(tr -d '\r ' < "$W/n")" 0
check "the test key carried in the image" "$(./vmssh 'ls /usr/share/enkrnett/keys' 2>/dev/null | tr -d '\r')" vm-test.pub

echo "## the root side refuses an image the project did not sign (ENK-UPD-003, twice over)"
# The daemon's own refusal of a stranger's signature is in the Rust tests (tests/install.rs); here
# the root side is exercised directly: a stranger-signed copy of the image staged by hand and the
# event fired as the daemon would fire it. usign must refuse it, and nothing is flashed.
./vmscp "$W/www/stranger/$(basename "$GOOD")" /tmp/enkrnett-update-stranger.bin >/dev/null
./vmscp "$W/www/stranger/$(basename "$GOOD").sig" /tmp/enkrnett-update-stranger.sig >/dev/null
./vmssh 'rm -rf /tmp/enkrnett-update; mkdir -p /tmp/enkrnett-update && mv /tmp/enkrnett-update-stranger.bin /tmp/enkrnett-update/image && mv /tmp/enkrnett-update-stranger.sig /tmp/enkrnett-update/image.sig && echo vmtest-stranger > /tmp/enkrnett-update/request && ubus call service event "{\"type\":\"enkrnett.install\",\"data\":{}}"; sleep 5; cat /var/run/enkrnett-install/result' > "$W/verdict" 2>/dev/null || true
check "usign refuses the stranger's signature: refused bad_signature" "$(tr -d '\r\n' < "$W/verdict")" "vmtest-stranger refused bad_signature"
check "the device still runs 9.9.9" "$(api GET device "$T" >/dev/null; jq_ "['version']['enkrnett']")" 9.9.9
./vmssh 'rm -rf /tmp/enkrnett-update' >/dev/null 2>&1 || true

echo "## a deliberately broken successor: 9.9.10 without the wireless driver (ENK-UPD-005)"
set_url "https://$HOST_NAME:$HTTPS_PORT/b/index.json"
api POST update/check "$T" '{}' >/dev/null
check "9.9.10 offered" "$(jq_ "['available']['version']")" 9.9.10
S=$(api POST update/install "$T" '{}'); check "install accepted" "$S" 202
for i in $(seq 1 60); do S=$(api GET update "$T"); [ "$S" = 200 ] || break; STATE=$(jq_ "['state']"); [ "$STATE" = installing ] && break; sleep 1; done
wait_reboot
api GET device "$T" >/dev/null
check "9.9.10 runs" "$(jq_ "['version']['enkrnett']")" 9.9.10
api GET update "$T" >/dev/null
check "ENK-UPD-005 the update reported failed" "$(jq_ "['last_result']['result']")" failed
check "health_check_failed" "$(jq_ "['last_result']['reason']")" health_check_failed
api GET status "$T" >/dev/null
check "the owner's settings still served (guest name)" "$(jq_ "['settings']['guest']['name']")" "$GUEST_NAME"
check "the status screen carries the outcome" "$(jq_ "['update']['last_result']['reason']")" health_check_failed
[ "$(vm_log_count 'update_failed version=9.9.10 reason=health_check_failed')" -ge 1 ] && pass "ENK-API-029 update_failed logged" || fail "no update_failed in the log"

echo "## the disk as it was before the test"
api POST logout "$T" >/dev/null 2>&1 || true
./vm.sh stop >/dev/null 2>&1 || true; sleep 1
mv -f "$VM/disk.img.before-update" "$VM/disk.img"
rm -rf "$VM/successor-files"
{ ./vm.sh start >/dev/null && python3 vmsh.py wait 240 >/dev/null 2>&1; } || fail "the VM did not come back on its old disk"
for i in $(seq 1 40); do ./vmssh true >/dev/null 2>&1 && break; sleep 3; done
tunnel; login
api GET device "$T" >/dev/null; check "the old release again ($VERSION)" "$(jq_ "['version']['enkrnett']")" "$VERSION"
api POST update/check "$T" '{"enable_checks": false}' >/dev/null 2>&1 || true
./vmssh "rm -f /etc/ssl/certs/enkrnett-vm-test.crt /usr/share/enkrnett/keys/vm-test.pub; sed -i '/$HOST_NAME/d' /etc/hosts; uci -q delete enkrnett.main.update_index; uci commit enkrnett; /etc/init.d/enkrnett restart" >/dev/null 2>&1 || true
api POST logout "$T" >/dev/null 2>&1 || true
echo; if [ "$FAILS" = 0 ]; then echo "update install on the VM: every check passed"; else echo "update install on the VM: $FAILS check(s) failed"; exit 1; fi
