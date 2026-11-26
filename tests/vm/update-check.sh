#!/bin/bash
# The update check on the VM (chapter 11 §2–3: ENK-UPD-001, ENK-UPD-008, ENK-UPD-009,
# ENK-RLS-005, ENK-RLS-006; RFC 016 §9). The host serves a release index signed with a test key
# over HTTPS, with a test certificate authority installed on the VM for the duration; the daemon
# fetches it through uclient-fetch, verifies it against the key installed beside it, and offers
# the release. Then the refusals: a replayed older index, a tampered index, a stranger's
# signature, a document that is not an index, a plain http URL, an unreachable server. The VM is
# left as it was: no URL, no test authority, no test key, checks off.
#
#   tests/vm/update-check.sh VM_DIR     (after tests/vm/claim.sh: a CLAIMED device)
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
VM=${1:?VM_DIR}
PASSWORD="correct horse battery staple"
PORT=18086
HTTPS_PORT=18443
HOST_NAME=updates.enkrnett.test
TOOL="$ROOT/target/release/enkrnett-release"
[ -x "$TOOL" ] || ( cd "$ROOT" && cargo build --release -q -p enkrnett-release )
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)
W=$(mktemp -d)
SERVER=""; TUNNEL=""
cleanup() { [ -n "$SERVER" ] && kill "$SERVER" 2>/dev/null; [ -n "$TUNNEL" ] && kill "$TUNNEL" 2>/dev/null; rm -rf "$W"; return 0; }
trap cleanup EXIT
FAILS=0
pass() { echo "  ok   $*"; }
fail() { echo "  FAIL $*"; FAILS=$((FAILS + 1)); }
check() { if [ "$2" = "$3" ]; then pass "$1"; else fail "$1 (got '$2', expected '$3')"; fi; }
api() { local method=$1 path=$2 token=${3:-} body=${4:-}
  local args=(-s -o /tmp/enkrnett-u-body -w '%{http_code}' --connect-timeout 5 -m 60 --retry 8 --retry-connrefused --retry-all-errors --retry-delay 2 -X "$method" "http://127.0.0.1:$PORT/api/1/$path" -H "Host: 192.168.207.1")
  [ -n "$token" ] && args+=(-H "Enkrnett-Session: $token")
  [ "$method" = POST ] && args+=(-H 'Content-Type: application/json' -d "${body:-{\}}")
  curl "${args[@]}" 2>/dev/null || true; }
jq_() { python3 -c "import json,sys; d=json.load(open('/tmp/enkrnett-u-body')); print(eval('d'+sys.argv[1]))" "$1" 2>/dev/null || echo "?"; }

cd "$VM"
./vm.sh start >/dev/null || true
python3 vmsh.py wait 120 >/dev/null 2>&1 || true

echo "## a signed index and its variants, made on the host"
"$TOOL" keygen -c "enkrnett VM test" "$W/test" >/dev/null
"$TOOL" keygen -c "a stranger" "$W/stranger" >/dev/null
IMG="$W/enkrnett-9.9.9-25.12-x86_64_generic-sysupgrade.bin"; head -c 1048576 /dev/zero > "$IMG"
mkdir -p "$W/www/good" "$W/www/stale" "$W/www/tampered" "$W/www/stranger" "$W/www/bad"
release=(--release x86_64_generic 9.9.9 25.12 25.12 "$IMG" 24576 https://example.org/releases/9.9.9 180)
"$TOOL" index --sequence 5 --out "$W/www/good/index.json" "${release[@]}" >/dev/null
"$TOOL" sign "$W/test.sec" "$W/www/good/index.json" >/dev/null
"$TOOL" index --sequence 1 --out "$W/www/stale/index.json" "${release[@]}" >/dev/null
"$TOOL" sign "$W/test.sec" "$W/www/stale/index.json" >/dev/null
sed 's/"sequence": 5/"sequence": 6/' "$W/www/good/index.json" > "$W/www/tampered/index.json"; cp "$W/www/good/index.json.sig" "$W/www/tampered/"
cp "$W/www/good/index.json" "$W/www/stranger/"; "$TOOL" sign "$W/stranger.sec" "$W/www/stranger/index.json" >/dev/null
printf '{"format": 1}\n' > "$W/www/bad/index.json"; "$TOOL" sign "$W/test.sec" "$W/www/bad/index.json" >/dev/null
"$TOOL" verify "$W/www/good/index.json" --key "$W/test.pub" >/dev/null && pass "the tool verifies what it signed" || fail "the tool does not verify its own signature"
"$TOOL" verify "$W/www/stranger/index.json" --key "$W/test.pub" >/dev/null 2>&1 && fail "a stranger's signature verified on the host" || pass "a stranger's signature is refused on the host"

echo "## an HTTPS server on the host with a test authority"
openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -keyout "$W/srv.key" -out "$W/srv.crt" -days 2 \
  -subj "/CN=$HOST_NAME" -addext "subjectAltName=DNS:$HOST_NAME,IP:10.0.2.2" >/dev/null 2>&1
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

echo "## the test authority, the host name, and the test key on the VM"
./vmscp "$W/srv.crt" /etc/ssl/certs/enkrnett-vm-test.crt >/dev/null
./vmssh "grep -q '$HOST_NAME' /etc/hosts || echo '10.0.2.2 $HOST_NAME' >> /etc/hosts; mkdir -p /usr/share/enkrnett/keys" >/dev/null
./vmscp "$W/test.pub" /usr/share/enkrnett/keys/vm-test.pub >/dev/null
./vmssh "uclient-fetch -q -O - https://$HOST_NAME:$HTTPS_PORT/good/index.json 2>&1 | head -c 40" > "$W/probe" 2>&1 || true
grep -q '"format"' "$W/probe" && pass "uclient-fetch reaches the host over HTTPS with the test authority" || fail "uclient-fetch over HTTPS: $(tr -d '\n' < "$W/probe")"

ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -i id_ed25519 -p 2222 -N -L "$PORT:192.168.207.1:8080" root@127.0.0.1 &
TUNNEL=$!; sleep 2
T=""
login() { local s; s=$(api POST login '' "{\"password\":\"$PASSWORD\"}"); [ "$s" = 200 ] || { sleep 5; s=$(api POST login '' "{\"password\":\"$PASSWORD\"}"); }; [ "$s" = 200 ] || fail "login ($s)"; T=$(jq_ "['token']"); }
# The URL reaches the daemon through the init script (its config file is root's); a restart
# re-reads it. Waits for the new start line in the log.
set_url() {
  local n0; n0=$(./vmssh "logread | grep -c 'enkrnettd started'" 2>/dev/null || echo 0)
  if [ -n "$1" ]; then ./vmssh "uci set enkrnett.main.update_index='$1'; uci commit enkrnett; /etc/init.d/enkrnett restart" >/dev/null 2>&1
  else ./vmssh "uci -q delete enkrnett.main.update_index; uci commit enkrnett; /etc/init.d/enkrnett restart" >/dev/null 2>&1; fi
  for i in $(seq 1 30); do [ "$(./vmssh "logread | grep -c 'enkrnettd started'" 2>/dev/null || echo 0)" -gt "$n0" ] && break; sleep 2; done
  sleep 1; login
}

echo "## a package install does not look for updates until the owner says so (ENK-UPD-008)"
set_url "https://$HOST_NAME:$HTTPS_PORT/good/index.json"
api GET update "$T" >/dev/null
check "checks are off on a package install" "$(jq_ "['checks_enabled']")" False
check "the installed version is the daemon's ($VERSION)" "$(jq_ "['installed']['version']")" "$VERSION"
check "ENK-RLS-003 the baseline" "$(jq_ "['installed']['baseline']")" 25.12
api POST update/check "$T" '{}' >/dev/null
check "no check while the switch is off" "$(jq_ "['last_check']")" None
S=$(api POST update/check "$T" '{"enable_checks": true}'); check "the switch and the first check in one request" "$S" 200
check "checks on" "$(jq_ "['checks_enabled']")" True
check "ENK-UPD-001 the index fetched over HTTPS through uclient-fetch and verified" "$(jq_ "['last_check']['result']")" ok
check "ENK-RLS-006 the release offered" "$(jq_ "['available']['version']")" 9.9.9
check "its size in KiB" "$(jq_ "['available']['size_kib']")" 1024
check "same baseline" "$(jq_ "['available']['changes_baseline']")" False
check "state idle" "$(jq_ "['state']")" idle
./vmssh "cat /etc/enkrnett/state.json" > "$W/state" 2>/dev/null || true
check "ENK-UPD-009 the sequence remembered on flash" "$(python3 -c "import json,sys; print(json.load(open(sys.argv[1]))['index_sequence'])" "$W/state" 2>/dev/null || echo '?')" 5
./vmssh "logread | grep -c 'index_verified key='" > "$W/n" 2>/dev/null || echo 0 > "$W/n"; [ "$(cat "$W/n")" -ge 1 ] && pass "index_verified logged with the key number" || fail "no index_verified in the log"
./vmssh "logread | grep -c 'update_checked by=request result=ok'" > "$W/n" 2>/dev/null || echo 0 > "$W/n"; [ "$(cat "$W/n")" -ge 1 ] && pass "ENK-API-029 update_checked logged" || fail "no update_checked in the log"

echo "## refusals (ENK-UPD-003 for the index, ENK-UPD-009; threat 43)"
set_url "https://$HOST_NAME:$HTTPS_PORT/stale/index.json"; api POST update/check "$T" '{}' >/dev/null
check "a replayed older index: stale_index" "$(jq_ "['last_check']['result']")" stale_index
set_url "https://$HOST_NAME:$HTTPS_PORT/tampered/index.json"; api POST update/check "$T" '{}' >/dev/null
check "a changed index under the old signature: bad_signature" "$(jq_ "['last_check']['result']")" bad_signature
set_url "https://$HOST_NAME:$HTTPS_PORT/stranger/index.json"; api POST update/check "$T" '{}' >/dev/null
check "a stranger's signature: bad_signature" "$(jq_ "['last_check']['result']")" bad_signature
set_url "https://$HOST_NAME:$HTTPS_PORT/bad/index.json"; api POST update/check "$T" '{}' >/dev/null
check "a signed document that is not an index: bad_index" "$(jq_ "['last_check']['result']")" bad_index
check "nothing offered after the refusals" "$(jq_ "['available']")" None
set_url "http://$HOST_NAME:$HTTPS_PORT/good/index.json"; api POST update/check "$T" '{}' >/dev/null
check "a plain http URL is not tried: unreachable" "$(jq_ "['last_check']['result']")" unreachable
./vmssh "logread | grep -c 'not an https URL'" > "$W/n" 2>/dev/null || echo 0 > "$W/n"; [ "$(cat "$W/n")" -ge 1 ] && pass "the refusal of http is in the log" || fail "no 'not an https URL' in the log"
kill "$SERVER" 2>/dev/null || true; SERVER=""; sleep 1
set_url "https://$HOST_NAME:$HTTPS_PORT/good/index.json"; api POST update/check "$T" '{}' >/dev/null
check "the server gone: unreachable" "$(jq_ "['last_check']['result']")" unreachable

echo "## the VM as it was"
api POST update/check "$T" '{"enable_checks": false}' >/dev/null; check "checks off again" "$(jq_ "['checks_enabled']")" False
api POST logout "$T" >/dev/null || true
./vmssh "rm -f /etc/ssl/certs/enkrnett-vm-test.crt /usr/share/enkrnett/keys/vm-test.pub; sed -i '/$HOST_NAME/d' /etc/hosts" >/dev/null 2>&1 || true
set_url ""
api POST logout "$T" >/dev/null || true
echo; if [ "$FAILS" = 0 ]; then echo "update check on the VM: every check passed"; else echo "update check on the VM: $FAILS check(s) failed"; exit 1; fi
